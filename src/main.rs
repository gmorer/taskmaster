mod task;
mod config;
mod exec;
use config::Conf;

type Error = Box<dyn std::error::Error>;

const CONFIGURATION: &str = "/home/tet/project/taskmaster/example.toml";

fn main() -> Result<(), Error> {
	let config: Conf = config::Conf::new(CONFIGURATION.to_string())?;
	config.autostart();
	Ok(())
}
