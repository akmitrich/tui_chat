use redis::JsonCommands;

use crate::{connector::write_to_stream, session::Session};

pub fn extract_one_string_from_array(v: &serde_json::Value) -> Option<String> {
    v.as_array()
        .and_then(|v| v.first().and_then(|s| s.as_str()))
        .map(ToOwned::to_owned)
}

pub fn blocking_get_from_session(
    con: &mut redis::Connection,
    session_id: &str,
    path: &str,
) -> Option<serde_json::Value> {
    con.json_get(session_id, path)
        .ok()
        .and_then(|s: String| serde_json::from_str::<serde_json::Value>(&s).ok())
}

pub fn blocking_update_session_timestamp(con: &mut redis::Connection, session_id: &str) {
    let now = chrono::Local::now();
    let _: redis::RedisResult<()> = con.json_set(
        session_id,
        "$.timestamp",
        &serde_json::json!(now.timestamp_millis()),
    );
}

pub async fn user_output_into_session(
    con: &mut redis::aio::MultiplexedConnection,
    session: &Session,
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
        write_to_stream(
            con,
            &session.chat_id,
            &[(session.robot.as_str(), msg.as_str())],
        )
        .await;
    }
}
