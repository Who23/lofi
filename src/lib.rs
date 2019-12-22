mod sink;
mod types;

use termion::raw::IntoRawMode;
use termion::event::Key;
use termion::input::TermRead;

use std::net::UdpSocket;
use std::process::{Command, Stdio};

use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::Sender;

use std::fs::File;
use std::path::Path;
use std::io::{self, BufReader, Write};

use crate::sink::LofiSink;
use crate::types::{Message, State};

// so that main.rs can get the config
pub use crate::types::Config;

pub fn run(config: Config) {
    if let Message::NoMessage = config.message {
        // calls download.sh. See file for details.
        Command::new("./src/download.sh")
            .arg(&config.playlist)
            .status()
            .unwrap();

        play_music(config);
    } else {
        send_message(config.message);
    }
}

pub fn play_music(config: Config) {

    let (tx, rx) = mpsc::channel();

    // for the cycle_songs thread, the message_on_end thread, and in the Message::Previous block
    let tx1 = mpsc::Sender::clone(&tx);
    let tx2 = mpsc::Sender::clone(&tx);
    let tx3 = mpsc::Sender::clone(&tx);


    let device = rodio::default_output_device().unwrap();
    let mut sink = LofiSink::new(&device, tx2);
    let file = File::open("./music/playing.mp3").unwrap();
    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
    
    sink.append(source);

    let mut state = State {
        is_playing: true,
        at_playing_song: true,
        can_skip: true,
        volume: 10
    };

    // set up user input thread
    spawn_input(tx, &config);

    // send a message when the track ends
    sink.message_on_end();

    show_tui(&state);
    loop {
        if let Ok(data) = rx.recv() {
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

                        sink = add_music(sink, String::from("./music/prev.mp3"), true, &state);

                        state.at_playing_song = false;
                        state.can_skip = true;
                        show_tui(&state);

                        // consume a next message that follows so that it does not download a song.
                        // TODO: this is very hacky, another way to do this?
                        if let Ok(data) = rx.recv() {
                            match data {
                                Message::Next => {},
                                message => { tx3.send(message).unwrap(); }
                            }
                        }

                    }
                },
                Message::Next => {
                    if state.at_playing_song && state.can_skip {

                        // this triggers Message::SoundEnded when we cut the sound, cycling songs.
                        sink = add_music(sink, String::from("./music/next.mp3"), true, &state);
                        cycle_songs(&tx1, &config.playlist);
                        
                        state.can_skip = false;
                        show_tui(&state);

                        // consume a next message that follows so that it does not download a song.
                        // TODO: this is very hacky, another way to do this?
                        if let Ok(data) = rx.recv() {
                            match data {
                                Message::Next => {},
                                message => { tx3.send(message).unwrap(); }
                            }
                        }
                        
                    } else if !state.at_playing_song {
                        sink = add_music(sink, String::from("./music/playing.mp3"), true, &state);
                        state.at_playing_song = true;

                        // sometimes next.mp3 is downloading so needs to check if it exists
                        if Path::new("./music/next.mp3").exists() {
                            state.can_skip = true;
                        } else {
                            state.can_skip = false;
                        }

                        show_tui(&state);

                        // consume a next message that follows so that it does not download a song.
                        // TODO: this is very hacky, another way to do this?
                        if let Ok(data) = rx.recv() {
                            match data {
                                Message::Next => {},
                                message => { tx3.send(message).unwrap(); }
                            }
                        }

                    }
                },
                Message::VolDown => {
                    // obviously, there is no sound at 0.
                    if state.volume > 0 {
                        state.volume -= 1;
                        sink.set_volume(state.volume as f32 / 10.0);
                        show_tui(&state);
                    }
                }
                Message::VolUp => {
                    // the sound gets very crackly at 1.5
                    if state.volume < 14 {
                        state.volume += 1;
                        sink.set_volume(state.volume as f32 / 10.0);
                        show_tui(&state);
                    }
                }
                Message::Downloaded => {
                    state.can_skip = true;
                    show_tui(&state);
                }
                _ => {},
            }
        }
    }
    println!("\n\n");
}

pub fn send_message(message: Message) {
    let socket = UdpSocket::bind("127.0.0.1:34255").expect("couldn't bind to address");
    socket.connect("127.0.0.1:31165").expect("connect function failed");


    socket.send(&[message.encode() as u8]).expect("couldn't send message");
}

fn add_music(sink: LofiSink, file_path: String, reset: bool, state: &State) -> LofiSink {

    let file = File::open(file_path).unwrap();
    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();

    let new_sink = if reset {
        // if a sink is stopped, it cannot be restarted
        // a new sink needs to be created in order to be usable
        // a little annoying, but it's the cleanest solution
        sink.stop();

        let device = rodio::default_output_device().unwrap();
        let new_sink = LofiSink::new(&device, mpsc::Sender::clone(&sink.message_tx));
        new_sink.append(source);

        new_sink
    } else {
        sink.append(source);

        sink
    };

    // make sure state persists through tracks
    new_sink.message_on_end();
    if !state.is_playing { new_sink.pause(); }
    new_sink.set_volume(state.volume as f32 / 10.0);



    new_sink
    
}

// how to do this with generics?
fn cycle_songs(tx: &Sender<Message>, playlist: &String) {
    let tx1 = mpsc::Sender::clone(tx);
    let playlist_duplicate = playlist.clone();
    thread::spawn(move || {

        Command::new("./src/cycle_songs.sh")
        .arg(playlist_duplicate)
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
    let vol_slider = format!("{}{}", "#".repeat(state.volume as usize), "-".repeat(14 - state.volume as usize));

    //print tui
    print!("\n\r");
    print!("{}        {}        {}\n\r", prev_symbol, play_pause_symbol, next_symbol);
    print!(" \u{fa80}{}\u{fa7d}\n\r", vol_slider);

    // move cursor back up
    print!("\r\u{001b}[3A");
}

// how to do this with generics?
fn spawn_input(tx: Sender<Message>, config: &Config) {
    if config.daemon {
        // get input using messages sent to open socket

        thread::spawn(move || {
            let socket = UdpSocket::bind("127.0.0.1:31165").expect("couldn't bind to address");
            let mut buf = [0];
            loop {
                let (_, _) = socket.recv_from(&mut buf)
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
                    Key::Char('=') => tx.send(Message::VolUp).unwrap(),
                    Key::Char('-') => tx.send(Message::VolDown).unwrap(),
                    _ => {},
                }
                stdout.flush().unwrap();
            }

            tx.send(Message::Quit).unwrap();
        });
    }
}
