use crate::controller_signals::ControllerSignal;
use chrono::TimeZone;
use redis::{from_redis_value, streams::StreamKey, AsyncCommands, Value};
use std::{collections::HashMap, hash::BuildHasher};
use tokio::sync::mpsc;

pub enum ConnectorEvent {
    Post { message: String },
}

pub async fn output_connector(mut rx: tokio::sync::mpsc::Receiver<ConnectorEvent>) {
    eprintln!("Output thread begins.");
    let mut con = create_redis_connection().await;
    eprintln!("Start output");
    while let Some(event) = rx.recv().await {
        match event {
            ConnectorEvent::Post { message } => {
                let _: () = con
                    .xadd("42", "*", &[("Master", message.as_str())])
                    .await
                    .unwrap();
            }
        }
    }
}

pub async fn input_connector(tx: mpsc::Sender<ControllerSignal>) {
    eprintln!("Input thread begins.");
    let mut con = create_redis_connection().await;
    eprintln!("Start input");
    loop {
        let opts = redis::streams::StreamReadOptions::default()
            .count(10)
            .block(100);
        let result: Result<redis::streams::StreamReadReply, _> =
            con.xread_options(&[42], &["$"], &opts).await;
        match result {
            Ok(result) if !result.keys.is_empty() => {
                for key in result.keys {
                    eprintln!("{:?}", key);
                    process_input_key(tx.clone(), key).await;
                }
            }
            Err(e) => {
                let _ = tx
                    .send(ControllerSignal::Info {
                        message: format!("ERROR: {:?}", e),
                    })
                    .await;
            }
            _ => {}
        }
    }
}

async fn create_redis_connection() -> redis::aio::Connection {
    let client = redis::Client::open("redis://127.0.0.1/")
        .map_err(|e| eprintln!("Failed open client: {:?}", e))
        .unwrap();
    client
        .get_tokio_connection()
        .await
        .map_err(|e| eprintln!("Failed get connection: {:?}", e))
        .unwrap()
}

async fn process_input_key(tx: mpsc::Sender<ControllerSignal>, key: StreamKey) {
    for redis::streams::StreamId { id, map } in key.ids {
        processs_input_id(tx.clone(), &id, map).await
    }
}

async fn processs_input_id<S: BuildHasher>(
    tx: mpsc::Sender<ControllerSignal>,
    id: &str,
    map: HashMap<String, Value, S>,
) {
    for (from, message) in map {
        let _ = tx.send(make_incoming_message(id, from, message)).await;
    }
}

fn make_incoming_message(id: &str, from: String, message: Value) -> ControllerSignal {
    ControllerSignal::IncomingMessage {
        from,
        message: format!(
            "{}. {:?}",
            make_timestamp_string(id),
            from_redis_value::<String>(&message)
                .unwrap_or_else(|e| format!("{:?} ({:?})", message, e))
        ),
    }
}

fn make_timestamp_string(id: &str) -> String {
    if let Some((timestamp, _)) = id.split_once('-') {
        format!(
            "{}",
            chrono::Local
                .timestamp_millis_opt(timestamp.parse().unwrap())
                .unwrap()
                .format("%d/%m/%Y %H:%M")
        )
    } else {
        String::new()
    }
}
