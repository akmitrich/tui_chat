use tokio::sync::mpsc;

use redis::AsyncCommands;

use crate::controller_signals::ControllerSignal;

pub enum ConnectorEvent {
    Post { message: String },
}

pub async fn output_connector(mut rx: tokio::sync::mpsc::Receiver<ConnectorEvent>) {
    eprintln!("Output thread begins.");
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con = client.get_async_connection().await.unwrap();
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
    let client = redis::Client::open("redis://127.0.0.1/")
        .map_err(|e| eprintln!("Failed open client: {:?}", e))
        .unwrap();
    let mut con = client
        .get_tokio_connection()
        .await
        .map_err(|e| eprintln!("Failed get connection: {:?}", e))
        .unwrap();
    eprintln!("Start input");
    loop {
        let opts = redis::streams::StreamReadOptions::default()
            .count(1)
            .block(100);
        let result: Result<redis::streams::StreamReadReply, _> =
            con.xread_options(&[42], &["$"], &opts).await;
        match result {
            Ok(result) if !result.keys.is_empty() => {
                let _ = tx
                    .send(ControllerSignal::IncomingMessage {
                        from: "Redis".to_owned(),
                        message: format!("{:?}", result.keys.first().unwrap()),
                    })
                    .await;
                eprintln!("{:?}", result);
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
