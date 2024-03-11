use crate::controller_signals::ControllerSignal;
use cursive::{
    view::{Nameable, Resizable},
    views::{Dialog, EditView, LinearLayout, TextArea, TextView},
    View,
};
use tokio::sync::mpsc;

pub const MAIN_ID: &str = "main";
pub const VIEW_ID: &str = "view";
pub const EDIT_ID: &str = "edit";

pub fn create_main_view(tx: mpsc::Sender<ControllerSignal>) -> impl View {
    let tx_submit = tx.clone();
    let tx_quit = tx.clone();
    Dialog::around(create_main_layout(tx.clone()))
        .button("Submit", move |_| {
            let _ = tx_submit.blocking_send(ControllerSignal::Submit);
        })
        .button("Disconnect", move |_| {
            let _ = tx_quit.blocking_send(ControllerSignal::Quit);
        })
        .title("Main View")
        .with_name(MAIN_ID)
}

fn create_main_layout(tx: mpsc::Sender<ControllerSignal>) -> LinearLayout {
    let view = TextArea::new().disabled();
    let edit = EditView::new().on_submit(move |_, _| {
        let _ = tx.blocking_send(ControllerSignal::Submit);
    });
    LinearLayout::vertical()
        .child(view.with_name(VIEW_ID).full_height())
        .child(TextView::new("Введите сообщение:"))
        .child(edit.with_name(EDIT_ID).full_width())
}
