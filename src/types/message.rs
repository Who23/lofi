#[derive(Debug)]
pub enum Message {
    SoundEnded,
    Downloaded,
    VolUp,
    VolDown,
    Quit,
    Next,
    Previous,
    Toggle,
    NoMessage
}

impl Message {
    pub fn encode(&self) -> i32 {
        match self {
            Message::SoundEnded => 8,
            Message::Downloaded => 7,
            Message::VolUp => 6,
            Message::VolDown => 5,
            Message::Quit => 4,
            Message::Next => 3,
            Message::Toggle => 2,
            Message::Previous => 1,
            Message::NoMessage => 0,
        }
    }

    pub fn decode(number: i32) -> Message {
        match number {
            8 => Message::SoundEnded,
            7 => Message::Downloaded,
            6 => Message::VolUp,
            5 => Message::VolDown,
            4 => Message::Quit,
            3 => Message::Next,
            2 => Message::Toggle,
            1 => Message::Previous,
            0 => Message::NoMessage,
            s => (panic!("Not a valid message code: {}!", s))
        }
    }
}
