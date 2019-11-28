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
use std::io::{self, BufReader, Write};

use crate::sink::LofiSink;
use crate::types::{Message, State};

// so that main.rs can get the config
pub use crate::types::Config;

pub fn run(config: Config) {
    if let Message::NoMessage = config.message {
        play_music(config);
    } else {
        send_message(config.message);
    }
}

pub fn play_music(config: Config) {
    let device = rodio::default_output_device().unwrap();
    let mut sink = LofiSink::new(&device);
    let file = File::open("./music/playing.mp3").unwrap();
    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
    
    sink.append(source);

    let mut state = State {
        is_playing: true,
        at_playing_song: true,
        can_skip: true,
        volume: 1.0
    };


    let (tx, rx) = mpsc::channel();

    // for the cycle_songs thread, and the message_on_end thread
    let tx1 = mpsc::Sender::clone(&tx);
    let tx2 = mpsc::Sender::clone(&tx);

    // set up user input thread
    spawn_input(tx, &config);

    // send a message when the track ends
    sink.message_on_end(&tx2);

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
                        sink = add_music(sink, String::from("./music/prev.mp3"), true);

                        if !state.is_playing { sink.pause(); }
                        state.at_playing_song = false;
                        show_tui(&state);


                        // this rids the rx of the next value, which would always be a SoundEnded
                        // message. It would cycle the songs, so we prevent this.
                        // FIXME: Needs to be be at the end of this block or the tui, state, etc, don't update??
                        rx.recv().unwrap();
                    }
                },
                Message::Next => {
            
                    if state.at_playing_song && state.can_skip {
                        sink = add_music(sink, String::from("./music/next.mp3"), true);
                        
                        if !state.is_playing { sink.pause(); }
                        state.can_skip = false;
                        cycle_songs(&tx1);
                        show_tui(&state);

                        // this rids the rx of the next value, which would always be a SoundEnded
                        // message. It would cycle the songs, so we prevent this.
                        // FIXME: Needs to be be at the end of this block or the tui, state, etc, don't update??
                        rx.recv().unwrap();
                        
                    } else {
                        sink = add_music(sink, String::from("./music/playing.mp3"), true);
                        if !state.is_playing { sink.pause(); }
                        state.at_playing_song = true;
                        show_tui(&state);
                    }
                },
                Message::VolDown => {
                    // obviously, there is no sound at 0.
                    if state.volume > 0.0 {
                        state.volume -= 0.1;
                        sink.set_volume(state.volume);
                        show_tui(&state);
                    }
                }
                Message::VolUp => {
                    // the sound gets very crackly at 1.5
                    if state.volume < 1.4 {
                        state.volume += 0.1;
                        sink.set_volume(state.volume);
                        show_tui(&state);
                    }
                }
                Message::Downloaded => {
                    state.can_skip = true;
                    show_tui(&state);
                }
                Message::SoundEnded => {
                    // if the music finishes without skipping
                    // this is also triggered if the user skips/prevs the track
                    // through the cycle_songs function.

                    state.can_skip = false;
                    sink = add_music(sink, String::from("./music/next.mp3"), false);
                    
                    cycle_songs(&tx1);
                    sink.message_on_end(&tx2);
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

fn add_music(sink: LofiSink, file_path: String, reset: bool) -> LofiSink {
    let file = File::open(file_path).unwrap();
    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();

    if reset {
        // if a sink is stopped, it cannot be restarted
        // a new sink needs to be created in order to be usable
        // a little annoying, but it's the cleanest solution
        sink.stop();

        let device = rodio::default_output_device().unwrap();
        let new_sink = LofiSink::new(&device);
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
    let vol_slider = format!("{}{}", "#".repeat((state.volume * 10.0) as usize), "-".repeat(14 - ((state.volume * 10.0) as usize)));

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
