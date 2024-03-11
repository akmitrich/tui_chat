fn main() {
    if let Some(session_id) = std::env::args().nth(1) {
        let app = tui_chat::app::App::new();
        app.go(&session_id);
    } else {
        eprintln!("\nUsage: widget SESSION_ID\n");
    }
}
