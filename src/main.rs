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
    let sink = Sink::new(&device);

    let file = File::open("./music/playing.mp3").unwrap();
    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();

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
            }
            else if data == "toggle" {
                if sink.is_paused() {
                    sink.play();
                } else {
                    sink.pause();
                }
            }
        }

        if sink.empty() {
            println!("\rPlaying next track...");
        
            let file = File::open("./music/next.mp3").unwrap();
            let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
            sink.append(source);

            Command::new("./src/cycle_songs.sh")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap();
        }

        thread::sleep(Duration::from_millis(500));
    }
}
