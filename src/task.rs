use std::path::PathBuf;
use std::process::{ Child, Command };
use std::time::Instant;
use std::fs::OpenOptions;

#[allow(dead_code)]
#[derive(Debug)]
pub struct RunningTask {
    pub name: String,
    pub child: Child,
    pub conf_id: usize,
    pub start_time: Instant,
    // TODO
}

// name should be task name + identifier
impl RunningTask {
    pub fn new(conf_id: usize, child: Child, name: String) -> Self {
        println!("new task, pid: {}", child.id());
        RunningTask {
            name,
            child,
            conf_id,
            start_time: Instant::now(),
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
    pub binary: String,
    pub args: Vec<String>,
    pub numproc: u32,
    pub workingdir: Option<String>, // env::set_current_dir
    pub autostart: bool,
    pub autorestart: Autorestart,
    pub exitcodes: Vec<i32>,
    pub stopsignal: u32, // or a signal
    pub stoptime: u32,
    pub stdout: Option<PathBuf>,
    pub stderr: Option<PathBuf>,
    pub env: Vec<(String, String)>,
    pub index: u32,
}

impl TaskConf {
    pub fn run(&mut self) -> RunningTask {
        /*
            Umask:
             .pre_exec(|| { umask(self.umask) })
        */
        let mut spawner = Command::new(&self.binary);
        spawner.args(&self.args);
        spawner.envs(self.env.clone());
        if let Some(workingdir) = &self.workingdir {
            spawner.current_dir(workingdir);
        }
        if let Some(stdout) = &self.stdout {
            spawner.stdout(OpenOptions::new().append(true).create(true).open(stdout.as_path()).expect("TODO handke that"));
        }
        if let Some(stderr) = &self.stderr {
            spawner.stderr(OpenOptions::new().append(true).create(true).open(stderr.as_path()).expect("TODO handle that too"));
        }
		let child = spawner.spawn().expect("child died :(");
		self.index += 1;
        RunningTask::new(self.id, child, format!("{}_{}", self.name, self.index))
    }
}
