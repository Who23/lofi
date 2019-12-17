// TODO: do proper error handling
// TODO: use proper paths
// TODO: Add comments
// TODO: Get name/artist from title
// TODO: Download .wav so duration is accessible
// TODO: Daemonize by writing .service (systemctl) and .plist (launchctl) files.
// FIXME: fix crackly volume beyond 1.4 
// FIXME: When preving right after nexting, next button disappears even though you can still skip fine
// FIXME: Volume displays one higher than it is - or mutes at -1?

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
