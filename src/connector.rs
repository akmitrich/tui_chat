use std::sync::mpsc;

use crate::controller_signals::ControllerSignal;

pub enum ConnectorEvent {
    Post { message: String },
}

pub async fn connector(rx: mpsc::Receiver<ConnectorEvent>, tx: mpsc::Sender<ControllerSignal>) {
    loop {
        match rx.recv() {
            Ok(event) => match event {
                ConnectorEvent::Post { message } => {
                    let _ = tx.send(ControllerSignal::IncomingMessage {
                        from: "Me".to_owned(),
                        message,
                    });
                }
            },
            Err(_e) => break,
        }
    }
}
