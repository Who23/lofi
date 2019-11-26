// TODO: do proper error handling
// TODO: use proper paths
// TODO: Refactor/clean code
// TODO: Add comments
// TODO: Get name/artist from title
// TODO: Download .wav so duration is accessible
// TODO: Add ability to adjust volume
// TODO: Propogate Error and extract main function.
// TODO: Daemonize by writing .service (systemctl) and .plist (launchctl) files.
// TODO: Read from config file
// TODO: Custom youtube playlist
// FIXME: Process::exit causes no destructors, so it will not shutdown cleanly.
// FIXME: Make sure files are all present when calling (next.mp3, playing.mp3, etc.)
// FIXME: When preving right after nexting, next button disappears even though you can still skip fine

use std::env;
use std::process;
use lofi::Config;

fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Error!: {}", err);
        process::exit(1);
    });

    lofi::run(config);
}
