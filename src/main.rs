// TODO: do proper error handling
// TODO: use proper paths
// TODO: Refactor/clean code
// TODO: Add comments
// TODO: Get name/artist from title
// TODO: Download .wav so duration is accessible
// TODO: Add ability to adjust volume
// FIXME: Process::exit causes no destructors, so it will not shutdown cleanly.
// TODO: Propogate Error and extract main function.
// FIXME: Make sure files are all present when calling (next.mp3, playing.mp3, etc.)
// FIXME: When preving right after nexting, next button disappears even though you can still skip fine
// TODO: Daemonize by writing .service (systemctl) and .plist (launchctl) files.

use std::env;
use lofi::Config;

fn main() {
    let config = Config::new(env::args());

    lofi::run(config);
}