use std::process;
use std::env::Args;

use crate::types::Message;

#[derive(Debug)]
pub struct Config {
    pub daemon : bool,
    pub message : Message,
}

impl Config {
    pub fn new(mut args: Args) -> Config {

        let mut config = Config {
            daemon : false,
            message : Message::NoMessage,
        };

        // As far as I know, can't use a for loop here as args would
        // be borrowed. Matches arguments till iterator is exhausted.

        // Start from index two, as one is useless
        args.next();
        loop {
            let arg = args.next();

            if arg == None { break }

            match arg.unwrap().as_ref() {
                "-d" => (config.daemon = true),
                "-m" => (config.message = {
                    if let Some(message_arg) = args.next() { 
                        match message_arg.as_ref() {
                            "next" => Message::Next,
                            "previous" => Message::Previous,
                            "toggle" => Message::Toggle,
                            "quit" => Message::Quit,
                            _ => {
                                eprintln!("Config Parser Failed: Invalid Message Given!");
                                process::exit(1);
                            },
                        }
                    } else {
                        eprintln!("Config Parser Failed: No Message Given!");
                        process::exit(1);
                    }
                }),
                other_flag => {
                    eprintln!("Config Parser Failed: Unrecognized Flag: {}!", other_flag);
                    process::exit(1);
                },

            }
        }

        config
    }
}