mod task;
use libc::c_void;
use libc::write;

mod config;
use config::Conf;

type Error = Box<dyn std::error::Error>;

const CONFIGURATION: &str = "/home/tet/project/taskmaster/example.toml";

fn main() -> Result<(), Error> {
	let config = Conf::new(CONFIGURATION.to_string())?;
	dbg!(config);
	Ok(())
}
