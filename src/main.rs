use std::thread;
use std::sync::{ Arc };

mod task;
mod parser;
mod signals;
use signals::{ create_sigset, signal_handler };

mod config;
use config::Conf;

type Error = Box<dyn std::error::Error>;

const CONFIGURATION: &str = "/Users/cedricmpassi/Programming/42/taskmaster/ls.toml";

fn main() -> Result<(), Error> {
	let config = Arc::new(Conf::new(CONFIGURATION.to_string())?);
	config.autostart();

	let sigset = create_sigset();
	thread::spawn(move || {
		let conf = Arc::clone(&config);
		signal_handler(&sigset, &conf);
	});
	// signals
	// from_client
	loop { thread::sleep(std::time::Duration::from_secs(5));}
}
