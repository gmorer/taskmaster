use std::path::PathBuf;
use libc::{
    c_char,
    execve,
    fork,
    INT_MAX,
    SIG_DFL,
    signal,
    umask,
    dup2,
    STDOUT_FILENO,
    STDERR_FILENO
};
use std::time::Instant;
use std::fs::OpenOptions;
use std::os::unix::io::IntoRawFd;

#[allow(dead_code)]
#[derive(Debug)]
pub struct RunningTask {
    pub name: String,
    pub pid: i32,
    pub conf_id: usize,
    pub start_time: Instant,
    pub respawn_no: u32,
    // TODO
}

// name should be task name + identifier
impl RunningTask {
    pub fn new(conf_id: usize, pid: i32, respawn_no: u32, name: String) -> Self {
        println!("new task, pid: {}", pid);
        RunningTask {
            name,
            pid,
            conf_id,
            start_time: Instant::now(),
            respawn_no
        }
    }
}

// TODO: signal enum

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Autorestart {
    Always,
    Never,
    Unexpected,
}

impl Into<Autorestart> for String {
    fn into(self) -> Autorestart {
        match self.to_lowercase().as_ref() {
            "always" => Autorestart::Always,
            "never" => Autorestart::Never,
            "unexpected" => Autorestart::Unexpected,
            _ => Autorestart::Never, // TODO: log the error
        }
    }
}

#[derive(Debug)]
pub struct TaskConf {
    pub id: usize,
    pub name: String,
    pub binary: Vec<c_char>,
    pub args: Vec<Vec<c_char>>,
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
    pub stdout: Option<PathBuf>,
    pub stderr: Option<PathBuf>,
    pub env: Vec<String>,
    pub index: u32,
}

const NB_SIGNALS: i32 = 39;

unsafe fn reset_signals() {
    for n in 1..NB_SIGNALS {
        signal(n, SIG_DFL);
    }
}

fn redirect(fd: i32, path: &Option<PathBuf>) -> Result<(), std::io::Error>{
    if let Some(path) = path.as_ref() {
        let file = OpenOptions::new().append(true).create(true).open(path.as_path())?; // TODO more explicit error
        unsafe { dup2(file.into_raw_fd(), fd); }
        Ok(())
    }
    else { Ok(()) }
}


impl TaskConf {
    pub fn run(&mut self, respawn_no: u32) -> RunningTask { // TODO: handle errors
        let pid = unsafe { fork() };
        if pid > 0 && pid < INT_MAX {
            self.index += 1;
            return RunningTask::new(self.id, pid, respawn_no, format!("{}_{}", self.name, self.index))
        } else {
            if let Err(e) = self.run_child() {
                println!("{}", e);
            }
            std::process::exit(0);
        }
    }

    fn run_child(&self) -> Result<(), std::io::Error> {
        let mut args: Vec<*const c_char> = self.args.iter().map(|e| e.as_ptr()).collect();
        args.push(std::ptr::null() as *const c_char);
        redirect(STDOUT_FILENO, &self.stdout)?;
        redirect(STDERR_FILENO, &self.stderr)?;
        if let Some(workingdir) = &self.workingdir {
            std::env::set_current_dir(workingdir)?;
        }
        unsafe {
            reset_signals();
            umask(self.umask);
            execve(self.binary.as_ptr() as *const i8, self.args.as_ptr() as *const *const i8, std::ptr::null());
        }
        Ok(())
    }
}
