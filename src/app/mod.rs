use std::sync::mpsc;

use tokio::runtime::Runtime;

use crate::{
    connector::{connector, ConnectorEvent},
    controller_signals::ControllerSignal,
    ui::Ui,
};

pub struct App {
    ui: Ui,
    rx: mpsc::Receiver<ControllerSignal>,
    _async_runtime: Runtime,
    async_tx: mpsc::Sender<ConnectorEvent>,
}

impl App {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let (async_tx, async_rx) = mpsc::channel();
        let async_runtime = Runtime::new().expect("Failed to start asynchronous runtime.");
        async_runtime.spawn(connector(async_rx, tx.clone()));
        Self {
            ui: Ui::new(tx),
            rx,
            _async_runtime: async_runtime,
            async_tx,
        }
    }

    pub fn go(&mut self) {
        self.ui.init_view();
        self.run();
    }
}

impl App {
    fn run(&mut self) {
        loop {
            self.process_signals();
            self.ui.step_next();
            if self.ui.stopped() {
                break;
            }
        }
    }

    fn process_signals(&mut self) {
        while let Ok(signal) = self.rx.try_recv() {
            match signal {
                ControllerSignal::Submit => self.ui.submit(),
                ControllerSignal::IncomingMessage { from, message } => {
                    self.ui.append(&from, &message)
                }
                ControllerSignal::OutgoingMessage { message } => {
                    let _ = self.async_tx.send(ConnectorEvent::Post { message });
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
