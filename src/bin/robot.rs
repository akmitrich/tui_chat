use redis::JsonAsyncCommands;
use tui_chat::{interpret::Command, session::Session};

#[tokio::main]
async fn main() {
    let session_id = std::env::args().nth(1).unwrap();
    let mut con = tui_chat::connector::create_async_redis_connection().await;
    let session: redis::RedisResult<_> = con
        .json_get(session_id, "$")
        .await
        .map(|s: String| serde_json::from_str::<Vec<tui_chat::session::Session>>(&s).unwrap());
    match session {
        Ok(s) => serve(&mut con, s.first().unwrap()).await,
        Err(_) => todo!(),
    }
}

async fn serve(con: &mut redis::aio::MultiplexedConnection, session: &Session) {
    let chat_client = reqwest::Client::new();
    let mut context = serde_json::json!({});
    loop {
        eprintln!("Send: {:#?}", context);
        match chat_client
            .post(format!(
                "http://127.0.0.1:8000/api/v1/scripts/{}",
                session.script
            ))
            .json(&context)
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<serde_json::Value>().await {
                    Ok(mut interpreted) => {
                        let output = match interpreted["user_output"].take() {
                            serde_json::Value::Array(a) => a,
                            serde_json::Value::Object(o) => o.values().cloned().collect(),
                            v => vec![v],
                        };
                        for msg in output
                            .into_iter()
                            .filter_map(|v| v.as_str().map(ToOwned::to_owned))
                        {
                            tui_chat::connector::write_to_stream(
                                con,
                                &session.chat_id,
                                &[(session.robot.as_str(), msg.as_str())],
                            )
                            .await;
                        }

                        context["context"] = interpreted["context"].take();
                        match dbg!(Command::from(interpreted["command"].as_str().unwrap())) {
                            Command::Wait => {
                                context["user_input"] = serde_json::Value::Array(vec![]);
                                if let Some(serde_json::Value::Array(inp)) =
                                    context.get_mut("user_input")
                                {
                                    while inp.is_empty() {
                                        match tui_chat::connector::read_from_stream(
                                            con,
                                            &session.chat_id,
                                        )
                                        .await
                                        {
                                            Ok(keys) => {
                                                for key in keys {
                                                    for id in key.ids {
                                                        for (i, j) in id.map {
                                                            if i == session.username {
                                                                if let Ok(x) =
                                                                    redis::from_owned_redis_value(j)
                                                                {
                                                                    inp.push(
                                                                        serde_json::Value::String(
                                                                            x,
                                                                        ),
                                                                    );
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
                                if context["context"]["prev"].as_str().unwrap() != "12" {
                                    break;
                                }
                            }
                            Command::Finish => {}
                            Command::Pause => {}
                            Command::Noop => {}
                        }
                    }
                    Err(e) => eprintln!("Failed to receive JSON-response: {:?}", e),
                }
            }
            Err(_e) => todo!(),
            Ok(resp) => {
                eprintln!("ERROR: {:#?}", resp.text().await);
                break;
            }
        };
    }
    eprintln!("Final: {:#?}", context);
}
