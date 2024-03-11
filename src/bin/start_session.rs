use redis::JsonAsyncCommands;
use serde_json::json;

#[tokio::main]
async fn main() {
    let mut con = tui_chat::connector::create_redis_connection().await;
    let session = json!({
        "chat_id": "77",
        "username": "Customer",
        "robot": "Robot",
        "operator": "Operator",
    });
    let _: () = con.json_set("session_key", "$", &session).await.unwrap();
}
