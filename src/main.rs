// TODO: do proper error handling
// TODO: use proper paths
// TODO: Refactor/clean code
// TODO: Add comments
// TODO: Get name/artist from title
// TODO: Make sure you cannot skip when next.mp3 is downloading

use std::fs::File;
use std::io::{self, BufReader, Write};

use rodio::Sink;

use termion::raw::IntoRawMode;
use termion::event::Key;
use termion::input::TermRead;

use std::thread;
use std::sync::mpsc;
use std::process::{Command, Stdio};

fn main() {
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
                if state.at_playing_song {
                    sink = add_music(sink, String::from("./music/next.mp3"), true);
                    cycle_songs();
                    show_tui(&state);
                } else {
                    sink = add_music(sink, String::from("./music/playing.mp3"), true);
                    state.at_playing_song = true;
                    show_tui(&state);
                }
            }
        }

        if sink.empty() {
            // if the music finishes without skipping
            sink = add_music(sink, String::from("./music/next.mp3"), false);
            cycle_songs();
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

fn cycle_songs() {
    Command::new("./src/cycle_songs.sh")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
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