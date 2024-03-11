use redis::JsonAsyncCommands;
use serde_json::json;

#[tokio::main]
async fn main() {
    let mut con = tui_chat::connector::create_redis_connection().await;
    let now = std::time::SystemTime::now();
    let ts = now.duration_since(std::time::UNIX_EPOCH).unwrap();
    let session = json!({
        "chat_id": std::env::args().nth(2).unwrap_or_else(|| "42".to_owned()),
        "username": "Customer",
        "robot": "Robot",
        "operator": "Operator",
        "context": {},
        "timestamp": ts.as_millis(),
    });
    let _: () = con
        .json_set(
            std::env::args()
                .nth(1)
                .unwrap_or_else(|| "session_key".to_owned()),
            "$",
            &session,
        )
        .await
        .unwrap();
}
