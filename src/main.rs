mod task;
use libc::c_void;
use libc::write;

mod config;
use config::Conf;

type Error = Box<dyn std::error::Error>;

const CONFIGURATION: &str = "/home/tet/project/taskmaster/example.toml";

fn main() -> Result<(), Error> {
    let _config = Conf::new(CONFIGURATION.to_string())?;
    println!("Hello, world!");

    let arg: &[u8] = b"hello";
    let arg = arg.as_ptr() as *const c_void;
    unsafe {
        write(1, arg, 4);
    }
    Ok(())
}
