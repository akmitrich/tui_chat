use redis::JsonAsyncCommands;
use tui_chat::interpret::Command;

#[tokio::main]
async fn main() {
    let session_id = std::env::args().nth(1).unwrap();
    let mut con = tui_chat::connector::create_async_redis_connection().await;
    serve(&mut con, &session_id).await;
}

async fn serve(con: &mut redis::aio::MultiplexedConnection, session_id: &str) {
    let session: redis::RedisResult<_> = con
        .json_get(session_id, "$")
        .await
        .map(|s: String| serde_json::from_str::<Vec<tui_chat::session::Session>>(&s).unwrap());
    let mut sessions = session.unwrap();
    let session = sessions.first_mut().unwrap();
    let chat_client = reqwest::Client::new();

    let mut keep_going = true;
    while keep_going {
        eprintln!("Send: {:#?}", session.context);
        match interpret(&chat_client, session).await {
            Ok(resp) if resp.status().is_success() => match on_success(resp, con, session).await {
                Some(proceed) => keep_going = proceed,
                None => break,
            },
            Err(_e) => todo!(),
            Ok(bad_resp) => {
                eprintln!("ERROR: {:#?}", bad_resp.text().await);
                break;
            }
        };
        session.update_to_redis(con, session_id).await;
    }
    eprintln!("Final: {:#?}", session.context);
}

fn interpret(
    chat_client: &reqwest::Client,
    session: &tui_chat::session::Session,
) -> impl std::future::Future<Output = reqwest::Result<reqwest::Response>> {
    chat_client
        .post(format!(
            "http://127.0.0.1:8000/api/v1/scripts/{}",
            session.script
        ))
        .json(&session.context)
        .send()
}

async fn on_success(
    resp: reqwest::Response,
    con: &mut redis::aio::MultiplexedConnection,
    session: &mut tui_chat::session::Session,
) -> Option<bool> {
    match resp.json::<serde_json::Value>().await {
        Ok(mut interpreted) => {
            eprintln!("Received: {:#?}", interpreted);
            session
                .send_user_output_to_redis(con, interpreted["user_output"].take())
                .await;

            session.context["context"] = interpreted["context"].take();
            match Command::from(interpreted["command"].as_str().unwrap()) {
                Command::Wait => {
                    wait_for_user_input(con, session).await;
                    Some(true)
                }
                Command::Finish => {
                    session.context = serde_json::json!({});
                    Some(false)
                }
                Command::Pause => Some(true),
                Command::Operator => {
                    eprintln!(
                        "Need operator in chat {:?}. {:?}",
                        session.chat_id,
                        session.context["operator_message"].as_str()
                    );
                    Some(false)
                }
                Command::Noop => {
                    eprint!("NOOP after command: {:?}.", interpreted["command"]);
                    None
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to receive JSON-response: {:?}", e);
            None
        }
    }
}

async fn wait_for_user_input(
    con: &mut redis::aio::MultiplexedConnection,
    session: &mut tui_chat::session::Session,
) {
    let mut user_input = vec![];

    while user_input.is_empty() {
        match tui_chat::connector::read_from_stream(con, &session.chat_id, &session.stream_id).await
        {
            Ok(keys) => {
                for key in keys {
                    for id in key.ids {
                        session.stream_id = id.id;
                        for (source, msg) in id.map {
                            if source == session.username {
                                if let Ok(text) = redis::from_owned_redis_value(msg) {
                                    user_input.push(serde_json::Value::String(text));
                                }
                            }
                        }
                    }
                }
            }
            Err(_) => todo!(),
        }
    }
    session.context["user_input"] = serde_json::Value::Array(user_input);
}
