#[derive(Debug)]
pub enum Message {
    SoundEnded,
    Downloaded,
    Quit,
    Next,
    Previous,
    Toggle,
    NoMessage
}

impl Message {
    pub fn encode(&self) -> i32 {
        match self {
            Message::SoundEnded => 6,
            Message::Downloaded => 5,
            Message::Quit => 4,
            Message::Next => 3,
            Message::Toggle => 2,
            Message::Previous => 1,
            Message::NoMessage => 0,
        }
    }

    pub fn decode(number: i32) -> Message {
        match number {
            6 => Message::SoundEnded,
            5 => Message::Downloaded,
            4 => Message::Quit,
            3 => Message::Next,
            2 => Message::Toggle,
            1 => Message::Previous,
            0 => Message::NoMessage,
            s => (panic!("Not a valid message code: {}!", s))
        }
    }
}