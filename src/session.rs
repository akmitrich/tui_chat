use serde_json::json;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Session {
    chat_id: String,
    started: i64,
    script: String,
    username: String,
    robot: String,
    operator: String,
    timestamp: i64,
    context: serde_json::Value,
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
}
