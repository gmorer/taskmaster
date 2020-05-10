mod task;
use libc::c_void;
use libc::write;

mod config;
use config::Conf;

type Error = Box<dyn std::error::Error>;

const CONFIGURATION: &str = "/home/tet/project/taskmaster/example.toml";
use libc::c_char;
use libc::execve;
use libc::fork;
use libc::pid_t;
use libc::size_t;
use libc::strlen;
use libc::INT_MAX;
use std::ffi::CString;
use std::ptr;

fn exec_child(path: &str, argv: Vec<String>) -> pid_t {
    let converted_path = CString::new(path).unwrap();
    let v:Vec<*const c_char> = argv.into_iter().map(|string| CString::new(string).unwrap().as_ptr()).collect();
    v.append(ptr::null() as *const char);
    unsafe {
        execve(converted_path.as_ptr(), v, ptr::null());
    }
    return 0;
}

fn main() -> Result<(), Error> {
	let config = Conf::new(CONFIGURATION.to_string())?;
	dbg!(config);
	Ok(())
}
