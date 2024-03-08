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
    output_tx: mpsc::Sender<ConnectorEvent>,
}

impl App {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(1024);
        let (output_tx, output_rx) = mpsc::channel(1024);
        let async_runtime = Runtime::new().expect("Failed to start asynchronous runtime.");
        async_runtime.handle().spawn(output_connector(output_rx));
        async_runtime.handle().spawn(input_connector(tx.clone()));
        Self {
            ui: Ui::new(tx),
            rx,
            async_runtime,
            output_tx,
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
                ControllerSignal::Submit => self.ui.submit(),
                ControllerSignal::IncomingMessage { from, message } => {
                    eprintln!("Incoming Message. {:?} -> {}", from, message);
                    self.ui.append(&from, &message)
                }
                ControllerSignal::OutgoingMessage { message } => {
                    let _ = self
                        .output_tx
                        .blocking_send(ConnectorEvent::Post { message });
                }
                ControllerSignal::Info { message } => self.ui.present_info(&message),
                ControllerSignal::Quit => self.ui.stop(),
            }
        }
        if let Err(mpsc::error::TryRecvError::Disconnected) = self.rx.try_recv() {
            eprintln!("Application crashed!");
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
