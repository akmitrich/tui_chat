use crate::controller_signals::ControllerSignal;
use chrono::TimeZone;
use redis::{
    from_redis_value,
    streams::{StreamKey, StreamRangeReply},
    AsyncCommands,
};
use std::{collections::HashMap, hash::BuildHasher};
use tokio::sync::mpsc;

pub enum ConnectorEvent {
    Post { message: String },
}

pub async fn read_from_stream(
    con: &mut redis::aio::MultiplexedConnection,
    chat_id: &str,
    last_id: &str,
) -> redis::RedisResult<Vec<StreamKey>> {
    let opts = redis::streams::StreamReadOptions::default()
        .count(10)
        .block(0);
    let result: redis::streams::StreamReadReply = con
        .xread_options(&[chat_id.to_owned()], &[last_id], &opts)
        .await?;
    Ok(result.keys)
}

pub async fn write_to_stream(
    con: &mut redis::aio::MultiplexedConnection,
    chat_id: &str,
    items: &[(&str, &str)],
) {
    let _: redis::RedisResult<()> = con.xadd(chat_id, "*", items).await;
}

pub async fn output_connector(
    username: String,
    chat_id: String,
    mut rx: tokio::sync::mpsc::Receiver<ConnectorEvent>,
) {
    eprintln!("Output thread begins.");
    let mut con = create_async_redis_connection().await;
    eprintln!("Start output");
    while let Some(event) = rx.recv().await {
        match event {
            ConnectorEvent::Post { message } => {
                write_to_stream(&mut con, &chat_id, &[(username.as_str(), message.as_str())]).await;
            }
        }
    }
}

pub async fn input_connector(chat_id: String, tx: mpsc::Sender<ControllerSignal>) {
    eprintln!("Input thread begins.");
    let mut con = create_async_redis_connection().await;
    eprintln!("Start input");
    let mut last_id = "$".to_owned();

    read_old_messages(&mut con, &chat_id, tx.clone()).await;

    loop {
        match read_from_stream(&mut con, &chat_id, &last_id).await {
            Ok(result) if !result.is_empty() => {
                eprintln!("From stream {:?}", result);
                for key in result {
                    if let Some(result) = process_input_key(tx.clone(), key).await {
                        last_id = result;
                    }
                }
            }
            Err(e) => {
                let _ = tx
                    .send(ControllerSignal::Info {
                        message: format!("REDIS ERROR: {:?}", e),
                    })
                    .await;
            }
            _ => {}
        }
    }
}

pub async fn create_async_redis_connection() -> redis::aio::MultiplexedConnection {
    let client = redis::Client::open("redis://127.0.0.1/")
        .map_err(|e| eprintln!("Failed open client: {:?}", e))
        .unwrap();
    client
        .get_multiplexed_tokio_connection()
        .await
        .map_err(|e| eprintln!("Failed get connection: {:?}", e))
        .unwrap()
}

pub fn create_blocking_redis_connection() -> redis::RedisResult<redis::Connection> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    client.get_connection()
}

async fn read_old_messages(
    con: &mut redis::aio::MultiplexedConnection,
    chat_id: &str,
    tx: mpsc::Sender<ControllerSignal>,
) {
    let prev: Option<StreamRangeReply> = con.xrange_all(chat_id).await.unwrap_or_default();
    if let Some(reply) = prev {
        for stream_id in reply.ids {
            eprintln!("Prev: {:?}", stream_id);
            process_input_id(tx.clone(), &stream_id.id, stream_id.map).await;
        }
    }
}

async fn process_input_key(tx: mpsc::Sender<ControllerSignal>, key: StreamKey) -> Option<String> {
    let mut last_id = None;
    for redis::streams::StreamId { id, map } in key.ids {
        process_input_id(tx.clone(), &id, map).await;
        last_id = Some(id);
    }
    last_id
}

async fn process_input_id<S: BuildHasher>(
    tx: mpsc::Sender<ControllerSignal>,
    id: &str,
    map: HashMap<String, redis::Value, S>,
) {
    for (from, message) in map {
        let _ = tx.send(make_incoming_message(id, from, message)).await;
    }
}

fn make_incoming_message(id: &str, from: String, message: redis::Value) -> ControllerSignal {
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
                .format("%d/%m/%Y %H:%M:%S")
        )
    } else {
        String::new()
    }
}
