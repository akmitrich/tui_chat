use redis::JsonAsyncCommands;
use tui_chat::session::Session;

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
    let chat_client = 
}
