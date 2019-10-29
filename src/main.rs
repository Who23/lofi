// TODO: do proper error handling
// TODO: use proper paths
// TODO: Refactor/clean code
// TODO: Add comments
// TODO: Get name/artist from title
// TODO: Download .wav so duration is accessible
// TODO: Add ability to adjust volume
// FIXME: Process::exit causes no destructors, so it will not shutdown cleanly.
// Propogate Error and extract main function.
// TODO: Daemonize by writing .service (systemctl) and .plist (launchctl)
// files. Also listen on socket for commands. 
    // https://doc.rust-lang.org/1.12.1/std/env/fn.current_dir.html
    // https://doc.rust-lang.org/std/net/struct.UdpSocket.html
    // https://doc.rust-lang.org/std/net/struct.TcpStream.html

use std::fs::File;
use std::io::{self, BufReader, Write};

use rodio::Sink;

use termion::raw::IntoRawMode;
use termion::event::Key;
use termion::input::TermRead;

use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::Sender;

use std::process::{self, Command, Stdio};
use std::env::{self, Args};

fn main() {
    let config = Config::new(env::args());

    let device = rodio::default_output_device().unwrap();
    let mut sink = Sink::new(&device);

    let file = File::open("./music/playing.mp3").unwrap();
    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();

    sink.append(source);

    let mut state = State {
        is_playing: true,
        at_playing_song: true,
        can_skip: true
    };


    let (tx, rx) = mpsc::channel();
    let tx1 = mpsc::Sender::clone(&tx);     // for the cycle_songs thread

    // set up user input thread
    thread::spawn(move || {
        let stdin = io::stdin();
        let mut stdout = io::stdout().into_raw_mode().unwrap();

        for c in stdin.keys() {
            match c.unwrap() {
                Key::Ctrl('c') => break,
                Key::Char('k') => tx.send("toggle").unwrap(),
                Key::Char('j') => tx.send("previous").unwrap(),
                Key::Char('l') => tx.send("next").unwrap(),
                _ => {},
            }
            stdout.flush().unwrap();
        }

        tx.send("kill").unwrap();
    });

    show_tui(&state);
    loop {
        if let Ok(data) = rx.try_recv() {
            if data == "kill" {
                break;
            } else if data == "toggle" {
                if sink.is_paused() {
                    sink.play();
                    state.is_playing = true;
                } else {
                    sink.pause();
                    state.is_playing = false;
                }
                show_tui(&state);
            } else if data == "previous" {
                if state.at_playing_song {
                    sink = add_music(sink, String::from("./music/prev.mp3"), true);
                    state.at_playing_song = false;
                    show_tui(&state);
                }
            } else if data == "next" {
                if state.at_playing_song && state.can_skip {
                    sink = add_music(sink, String::from("./music/next.mp3"), true);
                    state.can_skip = false;
                    cycle_songs(&tx1);
                    show_tui(&state);
                } else {
                    sink = add_music(sink, String::from("./music/playing.mp3"), true);
                    state.at_playing_song = true;
                    show_tui(&state);
                }
            } else if data == "cycle_finished" {
                state.can_skip = true;
                show_tui(&state);
            }
        }

        if sink.empty() {
            // if the music finishes without skipping
            sink = add_music(sink, String::from("./music/next.mp3"), false);
            state.can_skip = false;
            cycle_songs(&tx1);
        }
    }
    println!("\n\n")
}


fn add_music(sink: Sink, file_path: String, reset: bool) -> Sink {
    let file = File::open(file_path).unwrap();
    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();

    if reset {
        // if a sink is stopped, it cannot be restarted
        // a new sink needs to be created in order to be usable
        // a little annoying, but it's the cleanest solution
        sink.stop();

        let device = rodio::default_output_device().unwrap();
        let new_sink = Sink::new(&device);
        new_sink.append(source);

        new_sink
    } else {
        sink.append(source);

        sink
    }
    
}

fn cycle_songs(tx: &Sender<&'static str>) {
    let tx1 = mpsc::Sender::clone(tx);
    thread::spawn(move || {
        Command::new("./src/cycle_songs.sh")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

        tx1.send("cycle_finished").unwrap();
    });
}

fn show_tui(state: &State) {

    let prev_symbol = if state.at_playing_song { "\u{f04a}" } else { " " };
    let play_pause_symbol = if state.is_playing { "\u{f04c}" } else { "\u{f04b}" };
    let next_symbol = if state.can_skip { "\u{f04e}" } else { " " };

    //print tui
    print!("\n\r");
    print!("{}\t\t{}\t\t{}\n\r", prev_symbol, play_pause_symbol, next_symbol);

    // move cursor back up
    print!("\u{001b}[2A");
}

struct State {
    is_playing: bool,
    at_playing_song: bool,  // are we at playing.mp3 or prev.mp3
    can_skip: bool,         // so that we cannot skip while next.mp3 downloads
}

#[derive(Debug)]
struct Config {
    daemon : bool,
    message : Message,
}

impl Config {
    fn new(mut args: Args) -> Config {

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

#[derive(Debug)]
enum Message {
    Next,
    Previous,
    Toggle,
    NoMessage
}

impl Message {
    fn encode(&self) -> i32 {
        match self {
            Message::Next => 3,
            Message::Toggle => 2,
            Message::Previous => 1,
            Message::NoMessage => 0,
        }
    }

    fn decode(number: i32) -> Message {
        match number {
            3 => Message::Next,
            2 => Message::Toggle,
            1 => Message::Previous,
            0 => Message::NoMessage,
            s => (panic!("Not a valid message code: {}!", s))
        }
    }
}