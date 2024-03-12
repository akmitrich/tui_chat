#[derive(Debug)]
pub enum Command {
    Wait,
    Finish,
    Pause,
    Noop,
}

impl From<&str> for Command {
    fn from(value: &str) -> Self {
        match value {
            "Wait" => Self::Wait,
            "Finish" => Self::Finish,
            "Pause" => Self::Pause,
            _ => Self::Noop,
        }
    }
}
