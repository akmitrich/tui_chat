use std::rc::Rc;

use crate::controller_signals::ControllerSignal;
use cursive::{
    view::{Nameable, Resizable},
    views::{Dialog, EditView, LinearLayout, TextView},
    Cursive,
};
use tokio::sync::mpsc;

const NAME_ID: &str = "name";
const CHAT_ID: &str = "chat_id";

pub fn create_intro_dialog(tx: mpsc::Sender<ControllerSignal>) -> Dialog {
    let entry_name = entry("Enter your name:\t", NAME_ID, tx.clone());
    let entry_chat = entry("Enter chat ID:\t", CHAT_ID, tx.clone());
    let layout = LinearLayout::vertical().child(entry_name).child(entry_chat);
    Dialog::around(layout).button("OK", move |siv| start(siv, tx.clone()))
}

fn start(siv: &mut Cursive, tx: mpsc::Sender<ControllerSignal>) {
    let username = siv
        .call_on_name(NAME_ID, |view: &mut EditView| view.get_content())
        .and_then(take_content);
    let chat_id = siv
        .call_on_name(CHAT_ID, |view: &mut EditView| view.get_content())
        .and_then(take_content);
    siv.pop_layer();
    let _ = tx.blocking_send(ControllerSignal::Intro { username, chat_id });
}

fn entry(title: &str, name: &str, tx: mpsc::Sender<ControllerSignal>) -> LinearLayout {
    LinearLayout::horizontal()
        .child(TextView::new(title))
        .child(
            EditView::new()
                .on_submit(move |siv, _| start(siv, tx.clone()))
                .with_name(name)
                .fixed_width(40),
        )
}

fn take_content(s: Rc<String>) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.as_str().to_owned())
    }
}
