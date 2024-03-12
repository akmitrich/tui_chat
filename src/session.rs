use redis::JsonAsyncCommands as _;
use serde_json::json;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Session {
    pub chat_id: String,
    pub started: i64,
    pub script: String,
    pub username: String,
    pub robot: String,
    pub operator: String,
    pub timestamp: i64,
    pub context: serde_json::Value,
}

impl Session {
    pub fn new(script: &str) -> Self {
        let ts = chrono::Local::now().timestamp_millis();
        Self {
            chat_id: format!("{}", uuid::Uuid::new_v4()),
            started: ts,
            script: script.to_owned(),
            username: "Customer".to_owned(),
            robot: "Robot".to_owned(),
            operator: "Operator".to_owned(),
            timestamp: ts,
            context: json!({}),
        }
    }

    pub async fn update(&self, con: &mut redis::aio::MultiplexedConnection, session_id: &str) {
        let now = chrono::Local::now();
        let _: redis::RedisResult<()> = con
            .json_set(
                session_id,
                "$.timestamp",
                &serde_json::json!(now.timestamp_millis()),
            )
            .await;
        let _: redis::RedisResult<()> = con.json_set(session_id, "$.context", &self.context).await;
    }
}
