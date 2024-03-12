use redis::JsonAsyncCommands;

#[tokio::main]
async fn main() {
    let mut con = tui_chat::connector::create_async_redis_connection().await;
    let session = tui_chat::session::Session::new(&std::env::args().nth(1).unwrap());
    let _: () = con
        .json_set(format!("{}", uuid::Uuid::new_v4()), "$", &session)
        .await
        .unwrap();
}
