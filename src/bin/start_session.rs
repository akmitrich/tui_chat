use redis::JsonAsyncCommands;
use serde_json::json;

#[tokio::main]
async fn main() {
    let mut con = tui_chat::connector::create_redis_connection().await;
    let now = std::time::SystemTime::now();
    let ts = now.duration_since(std::time::UNIX_EPOCH).unwrap();
    let _: () = con
        .json_set("small_talk", "$.timestamp", &json!(ts.as_millis()))
        .await
        .unwrap();
}
