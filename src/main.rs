mod task;

mod config;
use config::Conf;

type Error = Box<dyn std::error::Error>;

const CONFIGURATION: &str = "/home/tet/project/taskmaster/example.toml";

fn main() -> Result<(), Error> {
	let _config = Conf::new(CONFIGURATION.to_string())?;
	println!("Hello, world!");
	Ok(())
}
