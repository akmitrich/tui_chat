pub enum ControllerSignal {
    IncomingMessage {
        from: String,
        message: String,
    },
    Info {
        message: String,
    },
    Intro {
        username: Option<String>,
        chat_id: Option<String>,
    },
    OutgoingMessage {
        message: String,
    },
    Submit,
    Quit,
}
