mod ui;

use crate::{
    connector::{
        create_blocking_redis_connection, input_connector, output_connector, ConnectorEvent,
    },
    controller_signals::ControllerSignal,
    utils,
};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

pub struct App {
    ui: ui::Ui,
    async_runtime: Runtime,
    rx: mpsc::Receiver<ControllerSignal>,
    tx: mpsc::Sender<ControllerSignal>,
    output_tx: Option<mpsc::Sender<ConnectorEvent>>,
}

impl App {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(1024);
        let async_runtime = Runtime::new().expect("Failed to start asynchronous runtime.");
        Self {
            ui: ui::Ui::new(tx.clone()),
            async_runtime,
            rx,
            tx,
            output_tx: None,
        }
    }

    pub fn go(mut self, session_id: &str) -> Option<()> {
        self.ui.init_view();
        let mut con = create_blocking_redis_connection().ok()?;
        self.init_session(&mut con, session_id)?;
        self.run();
        utils::blocking_update_session_timestamp(&mut con, session_id);
        Some(())
    }
}

impl App {
    fn run(mut self) {
        loop {
            self.process_signals();
            self.ui.step_next();
            if self.ui.stopped() {
                break;
            }
        }
        // We have to move self into shutdown_timeout(...)
        // That is why it is difficult to impl Drop for App
        self.async_runtime
            .shutdown_timeout(std::time::Duration::from_millis(200));
    }

    fn process_signals(&mut self) {
        while let Ok(signal) = self.rx.try_recv() {
            match signal {
                ControllerSignal::IncomingMessage { from, message } => {
                    self.ui.append(&from, &message)
                }
                ControllerSignal::Info { message } => self.ui.present_info(&message),
                ControllerSignal::ConnectTo { username, chat_id } => {
                    if self.output_tx.is_none() {
                        self.connect_to(
                            username.as_deref().unwrap_or("NONAME"),
                            chat_id.as_deref().unwrap_or("42"),
                        );
                    } else {
                        let _ = self.tx.blocking_send(ControllerSignal::Info {
                            message: "RUNTIME ERROR:\ntrying to connect when already connected."
                                .to_owned(),
                        });
                    }
                }
                ControllerSignal::OutgoingMessage { message } => {
                    if let Some(output_tx) = self.output_tx.as_ref() {
                        let _ = output_tx.blocking_send(ConnectorEvent::Post { message });
                    }
                }
                ControllerSignal::Submit => self.ui.submit(),
                ControllerSignal::Quit => self.ui.stop(),
            }
        }
        if let Err(mpsc::error::TryRecvError::Disconnected) = self.rx.try_recv() {
            eprintln!("Application crashed!");
        }
    }

    fn init_session(&mut self, con: &mut redis::Connection, session_id: &str) -> Option<()> {
        let usernames = utils::blocking_get_from_session(con, session_id, "$.username")?;
        let chat_ids = utils::blocking_get_from_session(con, session_id, "$.chat_id")?;

        self.tx
            .blocking_send(ControllerSignal::ConnectTo {
                username: utils::extract_one_string_from_array(&usernames),
                chat_id: utils::extract_one_string_from_array(&chat_ids),
            })
            .ok()
    }

    fn connect_to(&mut self, username: &str, chat_id: &str) {
        self.ui.change_title(&format!("{} @ {}", username, chat_id));
        let (tx, output_rx) = mpsc::channel(1024);
        self.output_tx = Some(tx);
        self.async_runtime.handle().spawn(output_connector(
            username.to_owned(),
            chat_id.to_owned(),
            output_rx,
        ));
        self.async_runtime
            .handle()
            .spawn(input_connector(chat_id.to_owned(), self.tx.clone()));
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
