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
        session.update(con, session_id).await;
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
            tui_chat::utils::user_output_into_session(
                con,
                session,
                interpreted["user_output"].take(),
            )
            .await;

            session.context["context"] = interpreted["context"].take();
            match Command::from(interpreted["command"].as_str().unwrap()) {
                Command::Wait => {
                    session.context["user_input"] = serde_json::Value::Array(vec![]);
                    if let Some(serde_json::Value::Array(inp)) =
                        session.context.get_mut("user_input")
                    {
                        wait_for_user_input(
                            con,
                            session.chat_id.as_str(),
                            session.username.as_str(),
                            inp,
                        )
                        .await;
                    }
                    Some(true)
                }
                Command::Finish => Some(false),
                Command::Pause => Some(true),
                Command::Operator => Some(false),
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
    chat_id: &str,
    username: &str,
    inp: &mut Vec<serde_json::Value>,
) {
    while inp.is_empty() {
        match tui_chat::connector::read_from_stream(con, chat_id).await {
            Ok(keys) => {
                for key in keys {
                    for id in key.ids {
                        for (i, j) in id.map {
                            if i == username {
                                if let Ok(x) = redis::from_owned_redis_value(j) {
                                    inp.push(serde_json::Value::String(x));
                                }
                            }
                        }
                    }
                }
            }
            Err(_) => todo!(),
        }
    }
}
