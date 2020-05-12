use libc::c_char;
use libc::execve;
use libc::fork;
use libc::INT_MAX;
use std::ffi::CString;
use std::path::PathBuf;

#[allow(dead_code)]
pub struct RunnigTask {
    // TODO
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
    pub binary: String,
    pub args: Vec<String>,
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
    pub env: Vec<(String, String)>,
}

impl TaskConf {
    pub fn exec(&self) {
        let name = CString::new(self.binary.clone()).unwrap();
        let args: Vec<*const c_char> = self
            .args
            .iter()
            .map(|string| CString::new(string.clone()).unwrap().as_ptr())
            .collect();
        println!("Value of name: {:?}", name);
        unsafe {
            match fork() {
                1..=INT_MAX => {
                    execve(
                        name.as_ptr(),
                        std::ptr::null(),
                        std::ptr::null(),
                    );
                }
                _ => println!("Parent is fine"),
            }
        }
    }
}

