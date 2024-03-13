use redis::JsonAsyncCommands;
use serde_json::json;

use crate::connector::write_to_stream;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Session {
    pub chat_id: String,
    pub started: i64,
    pub script: String,
    pub username: String,
    pub robot: String,
    pub operator: String,
    pub stream_id: String,
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
            stream_id: "$".to_owned(),
            context: json!({}),
        }
    }

    pub async fn update_to_redis(
        &mut self,
        con: &mut redis::aio::MultiplexedConnection,
        session_id: &str,
    ) {
        let _: redis::RedisResult<()> = con.json_set(session_id, "$", &self).await;
    }

    pub async fn send_user_output_to_redis(
        &mut self,
        con: &mut redis::aio::MultiplexedConnection,
        user_output: serde_json::Value,
    ) {
        let output = match user_output {
            serde_json::Value::Array(a) => a,
            serde_json::Value::Object(o) => o.values().cloned().collect(),
            v => vec![v],
        };
        for msg in output
            .into_iter()
            .filter_map(|v| v.as_str().map(ToOwned::to_owned))
        {
            write_to_stream(con, &self.chat_id, &[(self.robot.as_str(), msg.as_str())]).await;
        }
    }
}
