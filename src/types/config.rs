use std::env::{Args, self};
use std::collections::HashMap;
use std::fs;


use crate::types::Message;

#[derive(Debug)]
pub struct Config {
    pub daemon : bool,
    pub message : Message,
    pub playlist : String
}

impl Config {
    pub fn new(mut args: Args) -> Result<Config, &'static str> {

        // used for storiing playlist aliases
        let mut aliases = HashMap::new();

        let mut config = Config {
            daemon : false,
            message : Message::NoMessage,
            playlist : String::from("PL6NdkXsPL07KiewBDpJC1dFvxEubnNOp1")
        };

        // get config items from config file
        let mut config_path = env::home_dir().ok_or_else(|| "Could not get home directory")?;
        config_path.push(".config");
        config_path.push("lofi");
        config_path.push("config");

        if config_path.exists() {
            let config_file = fs::read_to_string(config_path).unwrap();

            let config_file = config_file.lines()
                                        .map(|line| line.replace(" ", ""));

            for line in config_file {
                let line : Vec<&str> = line.split('=').collect();
                match line[0] {
                    "playlist" => { config.playlist = String::from(line[1]); },

                    // alias name key : playlist id
                    "alias" => { aliases.insert(line[1].to_owned(), line[2].to_owned()); }
                    _ => (),
                }
            }
        }

        // As far as I know, can't use a for loop here as args would
        // be borrowed. Matches arguments till iterator is exhausted.

        // Start from index two, as one is useless
        args.next();
        loop {
            let arg = args.next();

            if arg == None { break }

            match arg.unwrap().as_ref() {
                "-d" => (config.daemon = true),
                "-p" => (config.playlist = args.next().unwrap()),
                "-m" => (config.message = {
                    if let Some(message_arg) = args.next() { 
                        match message_arg.as_ref() {
                            "next" => Message::Next,
                            "previous" => Message::Previous,
                            "toggle" => Message::Toggle,
                            "volup" => Message::VolUp,
                            "voldown" => Message::VolDown,
                            "quit" => Message::Quit,
                            _ => {
                                return Err("Config Parser Failed: Invalid Message Given!")
                            },
                        }
                    } else {
                        return Err("Config Parser Failed: No Message Given!")
                    }
                }),
                _ => {
                    return Err("Config Parser Failed: Unrecognized Flag:")
                },

            }
        }

        // if the given playlist is an alias, substitute it for the playlist id
        if aliases.contains_key(&config.playlist) {
            config.playlist = aliases.get(&config.playlist).unwrap().to_owned();
        }

        Ok(config)
    }
}
