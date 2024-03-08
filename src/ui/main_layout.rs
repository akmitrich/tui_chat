use crate::controller_signals::ControllerSignal;
use cursive::{
    view::{Nameable, Resizable},
    views::{EditView, LinearLayout, TextArea, TextView},
};
use tokio::sync::mpsc;

pub const VIEW_ID: &str = "view";
pub const EDIT_ID: &str = "edit";

pub fn create_main_layout(tx: mpsc::Sender<ControllerSignal>) -> LinearLayout {
    let view = TextArea::new().disabled();
    let edit = EditView::new().on_submit(move |_, _| {
        let _ = tx.blocking_send(ControllerSignal::Submit);
    });
    LinearLayout::vertical()
        .child(view.with_name(VIEW_ID).full_height())
        .child(TextView::new("Введите сообщение:"))
        .child(edit.with_name(EDIT_ID).full_width())
}
