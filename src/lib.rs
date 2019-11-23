use std::fs::File;
use std::io::{self, BufReader, Write};

use rodio::Sink;

use termion::raw::IntoRawMode;
use termion::event::Key;
use termion::input::TermRead;

use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::Sender;

use std::env::Args;
use std::net::UdpSocket;
use std::process::{self, Command, Stdio};

pub fn run(config: Config) {
    if let Message::NoMessage = config.message {
        play_music(config);
    } else {
        send_message(config.message);
    }
}

fn play_music(config: Config) {
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

    // for the cycle_songs thread
    let tx1 = mpsc::Sender::clone(&tx);

    // set up user input thread
    spawn_input(tx, &config);

    show_tui(&state);
    loop {
        if let Ok(data) = rx.try_recv() {
            match data {
                Message::Quit => {
                    break;
                },
                Message::Toggle => {
                    if sink.is_paused() {
                        sink.play();
                        state.is_playing = true;
                    } else {
                        sink.pause();
                        state.is_playing = false;
                    }
                    show_tui(&state);
                },
                Message::Previous => {
                    if state.at_playing_song {
                        sink = add_music(sink, String::from("./music/prev.mp3"), true);
                        if !state.is_playing { sink.pause(); }
                        state.at_playing_song = false;
                        show_tui(&state);
                    }
                },
                Message::Next => {
                    if state.at_playing_song && state.can_skip {
                        sink = add_music(sink, String::from("./music/next.mp3"), true);
                        if !state.is_playing { sink.pause(); }
                        state.can_skip = false;
                        cycle_songs(&tx1);
                        show_tui(&state);
                    } else {
                        sink = add_music(sink, String::from("./music/playing.mp3"), true);
                        if !state.is_playing { sink.pause(); }
                        state.at_playing_song = true;
                        show_tui(&state);
                    }
                },
                Message::Downloaded => {
                    state.can_skip = true;
                    show_tui(&state);
                }
                _ => {},
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

fn send_message(message: Message) {
    let socket = UdpSocket::bind("127.0.0.1:34255").expect("couldn't bind to address");
    socket.connect("127.0.0.1:31165").expect("connect function failed");


    socket.send(&[message.encode() as u8]).expect("couldn't send message");
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

// how to do this with generics?
fn cycle_songs(tx: &Sender<Message>) {
    let tx1 = mpsc::Sender::clone(tx);
    thread::spawn(move || {
        Command::new("./src/cycle_songs.sh")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

        tx1.send(Message::Downloaded).unwrap();
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

// how to do this with generics?
fn spawn_input(tx: Sender<Message>, config: &Config) {
    if config.daemon {
        // get input using messages sent to open socket

        thread::spawn(move || {
            let socket = UdpSocket::bind("127.0.0.1:31165").expect("couldn't bind to address");
            let mut buf = [0];
            loop {
                let (number_of_bytes, src_addr) = socket.recv_from(&mut buf)
                                                        .expect("Didn't receive data");
                tx.send(Message::decode(buf[0] as i32)).unwrap(); 
            }
        });

    } else {
        // get input from terminal
        
        thread::spawn(move || {
            let stdin = io::stdin();
            let mut stdout = io::stdout().into_raw_mode().unwrap();

            for c in stdin.keys() {
                match c.unwrap() {
                    Key::Ctrl('c') => break,
                    Key::Char('k') => tx.send(Message::Toggle).unwrap(),
                    Key::Char('j') => tx.send(Message::Previous).unwrap(),
                    Key::Char('l') => tx.send(Message::Next).unwrap(),
                    _ => {},
                }
                stdout.flush().unwrap();
            }

            tx.send(Message::Quit).unwrap();
        });
    }
}

struct State {
    is_playing: bool,
    at_playing_song: bool,  // are we at playing.mp3 or prev.mp3
    can_skip: bool,         // so that we cannot skip while next.mp3 downloads
}

#[derive(Debug)]
pub struct Config {
    daemon : bool,
    message : Message,
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

#[derive(Debug)]
enum Message {
    Downloaded,
    Quit,
    Next,
    Previous,
    Toggle,
    NoMessage
}

impl Message {
    fn encode(&self) -> i32 {
        match self {
            Message::Downloaded => 5,
            Message::Quit => 4,
            Message::Next => 3,
            Message::Toggle => 2,
            Message::Previous => 1,
            Message::NoMessage => 0,
        }
    }

    fn decode(number: i32) -> Message {
        match number {
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
