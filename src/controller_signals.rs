pub enum ControllerSignal {
    Submit,
    IncomingMessage { from: String, message: String },
    OutgoingMessage { message: String },
    Info { message: String },
    Quit,
}
