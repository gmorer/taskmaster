mod task;
mod config;
mod exec;
use config::Conf;

type Error = Box<dyn std::error::Error>;

const CONFIGURATION: &str = "/Users/cedricmpassi/Programming/42/taskmaster/ls.toml";

fn main() -> Result<(), Error> {
	let config = Conf::new(CONFIGURATION.to_string())?;
	dbg!(config);
	Ok(())
}
