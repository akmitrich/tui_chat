use crate::{
    connector::{input_connector, output_connector, ConnectorEvent},
    controller_signals::ControllerSignal,
    ui::Ui,
};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

pub struct App {
    ui: Ui,
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
            ui: Ui::new(tx.clone()),
            async_runtime,
            rx,
            tx,
            output_tx: None,
        }
    }

    pub fn go(mut self) {
        self.ui.init_view();
        self.run();
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
        self.async_runtime
            .shutdown_timeout(std::time::Duration::from_millis(200));
    }

    fn process_signals(&mut self) {
        while let Ok(signal) = self.rx.try_recv() {
            match signal {
                ControllerSignal::IncomingMessage { from, message } => {
                    eprintln!("Incoming Message. {:?} -> {}", from, message);
                    self.ui.append(&from, &message)
                }
                ControllerSignal::Info { message } => self.ui.present_info(&message),
                ControllerSignal::Intro { username, chat_id } => {
                    self.connect_to(
                        username.as_deref().unwrap_or("NONAME"),
                        chat_id.as_deref().unwrap_or("42"),
                    );
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
