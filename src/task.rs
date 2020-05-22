use std::path::PathBuf;
use libc::{c_char, execve, fork, INT_MAX};
use std::time::SystemTime;

#[allow(dead_code)]
#[derive(Debug)]
pub struct RunningTask {
    name: String,
    time: SystemTime,
    pid: i32
}

// name should be task name + identifier
impl RunningTask {
	pub fn new(name: &str, pid: i32) -> Self {
		RunningTask {
                    name: name.to_string(),
                    time: SystemTime::now(),
                    pid
                }
	}
}

// TODO: signal enum

#[derive(Copy, Clone, Debug)]
pub enum Autorestart {
	True,
	False,
	Unexpected,
}

impl Into<Autorestart> for String {
	fn into(self) -> Autorestart {
		match self.to_lowercase().as_ref() {
			"true" => Autorestart::True,
			"false" => Autorestart::False,
			"unexpected" => Autorestart::Unexpected,
			_ => Autorestart::False, // TODO: log the error
		}
	}
}

#[derive(Debug)]
pub struct TaskConf {
	pub name: String,
	pub binary: Vec<c_char>,
	pub args: Option<Vec<Vec<c_char>>>,
	pub numproc: u32,
	pub umask: u32,
	pub workingdir: Option<PathBuf>, // env::set_current_dir
	pub autostart: bool,
	pub autorestart: Autorestart,
	pub exitcodes: Vec<i32>,
	pub startretries: u32,
	pub startime: u32,
	pub stopsignal: u32, // or a signal
	pub stoptime: u32,
	pub stdout: PathBuf,
	pub stderr: PathBuf,
	pub env: Vec<String>,
}

impl TaskConf {
	pub fn run(&self) -> i32 { // TODO: handle errors
		match unsafe { fork() } {
			pid if pid > 0 && pid < INT_MAX => pid,
			_ => {
				let mut args = match &self.args {
					Some(a) => a.iter().map(|e| e.as_ptr()).collect(),
					None => Vec::new(),
				};
				args.push(std::ptr::null());
				unsafe { execve(self.binary.as_ptr() as *const i8, args.as_ptr() as *const *const i8, std::ptr::null()); }
				0 // Will not be executed
			}
		}
	}
}
