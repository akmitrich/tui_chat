use redis::JsonCommands;

pub fn extract_one_string_from_array(v: &serde_json::Value) -> Option<String> {
    v.as_array()
        .and_then(|v| v.first().and_then(|s| s.as_str()))
        .map(ToOwned::to_owned)
}

pub fn blocking_get_json(
    con: &mut redis::Connection,
    session_id: &str,
    path: &str,
) -> Option<serde_json::Value> {
    con.json_get(session_id, path)
        .ok()
        .and_then(|s: String| serde_json::from_str::<serde_json::Value>(&s).ok())
}

pub fn blocking_update_session_timestamp(con: &mut redis::Connection, session_id: &str) {
    let now = std::time::SystemTime::now();
    if let Ok(ts) = now.duration_since(std::time::UNIX_EPOCH) {
        let _: redis::RedisResult<()> = con.json_set(
            session_id,
            "$.timestamp",
            &serde_json::json!(ts.as_millis()),
        );
    }
}
