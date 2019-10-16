//TODO: do proper error handling
//TODO: use proper paths
//TODO: add next/previous buttons
//TODO: Refactor/clean code
//TODO: Add comments

use std::fs::File;
use std::io::{self, BufReader, Write};

use rodio::Sink;
use rodio::source::Source;

use termion::raw::IntoRawMode;
use termion::event::Key;
use termion::input::TermRead;

use std::thread;
use std::sync::mpsc;
use std::time::Duration;
use std::process::{Command, Stdio};

fn main() {
    let device = rodio::default_output_device().unwrap();
    let mut sink = Sink::new(&device);

    let file = File::open("./music/playing.mp3").unwrap();
    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();

    // are we at playing.mp3 or prev.mp3
    let mut at_playing_song = true;

    println!("\rplaying song.mp3...");

    sink.append(source);

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

    loop {
        if let Ok(data) = rx.try_recv() {
            if data == "kill" {
                break;
            } else if data == "toggle" {
                println!("\rtoggling");
                if sink.is_paused() {
                    sink.play();
                } else {
                    sink.pause();
                }
            } else if data == "previous" {
                println!("\rgoin previous");
                sink = add_music(sink, String::from("./music/prev.mp3"), true);
            } else if data == "next" {
                println!("\rgoin next");
                sink = add_music(sink, String::from("./music/next.mp3"), true);
            }
        }

        if sink.empty() {
            println!("playing next track...");
            sink = add_music(sink, String::from("./music/next.mp3"), false);
            

            // Command::new("./src/cycle_songs.sh")
            //     .stdout(Stdio::null())
            //     .stderr(Stdio::null())
            //     .spawn()
            //     .unwrap()
            //     .wait()
            //     .unwrap();
        }
    }
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