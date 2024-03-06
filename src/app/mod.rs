use std::sync::mpsc;

use tokio::runtime::Runtime;

use crate::{
    connector::{input_connector, output_connector, ConnectorEvent},
    controller_signals::ControllerSignal,
    ui::Ui,
};

pub struct App {
    ui: Ui,
    tx: mpsc::Sender<ControllerSignal>,
    rx: mpsc::Receiver<ControllerSignal>,
    async_runtime: Runtime,
    async_tx: tokio::sync::mpsc::Sender<ConnectorEvent>,
    async_rx: Option<tokio::sync::mpsc::Receiver<ConnectorEvent>>,
}

impl App {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let (async_tx, async_rx) = tokio::sync::mpsc::channel(1024);
        let async_runtime = Runtime::new().expect("Failed to start asynchronous runtime.");
        Self {
            ui: Ui::new(tx.clone()),
            tx,
            rx,
            async_runtime,
            async_tx,
            async_rx: Some(async_rx),
        }
    }

    pub fn go(mut self) {
        self.ui.init_view();
        self.run();
    }
}

impl App {
    fn run(mut self) {
        if let Some(async_rx) = self.async_rx.take() {
            self.async_runtime
                .handle()
                .spawn(output_connector(async_rx));
        }
        self.async_runtime
            .handle()
            .spawn(input_connector(self.tx.clone()));
        loop {
            self.process_signals();
            self.ui.step_next();
            if self.ui.stopped() {
                break;
            }
        }
        self.async_runtime
            .shutdown_timeout(std::time::Duration::from_millis(300));
    }

    fn process_signals(&mut self) {
        while let Ok(signal) = self.rx.try_recv() {
            match signal {
                ControllerSignal::Submit => self.ui.submit(),
                ControllerSignal::IncomingMessage { from, message } => {
                    eprintln!("{:?} -> {}", from, message);
                    self.ui.append(&from, &message)
                }
                ControllerSignal::OutgoingMessage { message } => {
                    let tx = self.async_tx.clone();
                    self.async_runtime.handle().spawn(async move {
                        let _ = tx.send(ConnectorEvent::Post { message }).await;
                    });
                }
                ControllerSignal::Info { message } => self.ui.present_info(&message),
                ControllerSignal::Quit => self.ui.stop(),
            }
        }
        if let Err(mpsc::TryRecvError::Disconnected) = self.rx.try_recv() {
            eprintln!("Application crashed!");
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
