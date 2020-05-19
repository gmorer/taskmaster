use std::sync::Mutex;

use crate::task::{ TaskConf, RunningTask };
use crate::Error;
use crate::parser;

#[derive(Debug)]
pub struct Conf {
	pub port: u32,
	pub tasks: Mutex<Vec<TaskConf>>, // Mutex or RWlock ?
	pub runnings: Mutex<Vec<RunningTask>>
}

impl Conf {
	pub fn new(path: String) -> Result<Conf, Error> {
		parser::parse_config(path)
	}

	pub fn autostart(&self) {
		let tasks = self.tasks.lock().unwrap();
		let mut runnings = self.runnings.lock().unwrap();
		for task in tasks.iter() {
			if task.autostart == true {
				let pid = task.run(); // TODO: handle errors
				runnings.push(RunningTask::new(&task.name, &task.name, pid));
			}
		}
	}

	pub fn dead_task(&self, pid: i32) {
		let _tasks = self.tasks.lock().unwrap();
		let mut _runnings = self.runnings.lock().unwrap();
		println!("child died, pid: {}", pid);
	}
}
