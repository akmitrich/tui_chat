mod main;

use self::main::{EDIT_ID, MAIN_ID, VIEW_ID};
use crate::controller_signals::ControllerSignal;
use cursive::{
    event::Event,
    views::{Dialog, EditView, TextArea, TextView},
    Cursive, CursiveRunner,
};
use tokio::sync::mpsc;

pub struct Ui {
    runner: CursiveRunner<Cursive>,
    tx: mpsc::Sender<ControllerSignal>,
}

impl Ui {
    pub fn new(tx: mpsc::Sender<ControllerSignal>) -> Self {
        let ncurses =
            cursive::backends::curses::n::Backend::init().expect("Failed to init ncurses backend.");
        let runner = CursiveRunner::new(Cursive::default(), ncurses);

        Self { runner, tx }
    }

    pub fn init_view(&mut self) {
        let tx_ctrl_q = self.tx.clone();
        self.runner
            .add_global_callback(Event::CtrlChar('q'), move |_| {
                let _ = tx_ctrl_q.blocking_send(ControllerSignal::Quit);
            });

        self.runner
            .add_layer(main::create_main_view(self.tx.clone()));

        self.runner.refresh();
    }

    pub fn step_next(&mut self) {
        if !self.stopped() {
            self.runner.step();
            self.runner.refresh();
        }
    }

    pub fn stopped(&self) -> bool {
        !self.runner.is_running()
    }

    pub fn stop(&mut self) {
        self.runner.quit();
    }

    pub fn change_title(&mut self, title: &str) {
        self.runner
            .call_on_name(MAIN_ID, |view: &mut Dialog| view.set_title(title));
        self.runner.set_window_title(title);
    }

    pub fn submit(&mut self) {
        let message = self.take_message();
        if message.is_empty() {
            let _ = self.tx.blocking_send(ControllerSignal::Info {
                message: "You are trying to send an empty message to the chat.\nThis is forbidden."
                    .to_owned(),
            });
        } else {
            let _ = self
                .tx
                .blocking_send(ControllerSignal::OutgoingMessage { message });
        }
    }

    pub fn append(&mut self, from: &str, message: &str) {
        self.add_line_to_chat(&format!("[{}] -> {}", from, message));
    }

    pub fn present_info(&mut self, message: &str) {
        self.runner
            .add_layer(Dialog::around(TextView::new(message)).button("OK", |siv| {
                siv.pop_layer();
            }))
    }
}

impl Ui {
    fn take_message(&mut self) -> String {
        let content = self
            .runner
            .call_on_name(EDIT_ID, |view: &mut EditView| view.get_content())
            .unwrap()
            .as_str()
            .to_owned();
        self.runner
            .call_on_name(EDIT_ID, |view: &mut EditView| view.set_content(""));
        content
    }

    fn add_line_to_chat(&mut self, line: &str) {
        self.runner
            .call_on_name(VIEW_ID, |view: &mut TextArea| {
                let content = view.get_content();
                view.set_content(&format!("{}\n{}", content, line))
            })
            .unwrap();
    }
}
