use std::fs::File;
use std::io::{self, BufReader, Write};

use rodio::Sink;

use termion::raw::IntoRawMode;
use termion::event::Key;
use termion::input::TermRead;

fn main() {
    let device = rodio::default_output_device().unwrap();
    let sink = Sink::new(&device);

    let file = File::open("song.mp3").unwrap();
    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();
    println!("playing song.mp3...");

    sink.append(source);

    let stdin = io::stdin();
    let mut stdout = io::stdout().into_raw_mode().unwrap();


    for c in stdin.keys() {
        match c.unwrap() {
            Key::Ctrl('c') => break,
            Key::Char('k') => { if sink.is_paused() { sink.play(); } else { sink.pause(); } }
            _ => {}
        }
        stdout.flush().unwrap();
    }
}
