use crate::task::{ TaskConf, RunningTask };
use crate::Error;
use crate::parser;

#[derive(Debug)]
pub struct Conf {
    pub conf_id: usize, // change on every new tasks (+1)
	pub address: String,
	// Tasks in a hashmap with task_id as key?
    pub tasks: Vec<TaskConf>, // Mutex or RWlock ?
    pub runnings: Vec<RunningTask>,
}

impl Conf {
    pub fn new(path: String) -> Result<Conf, Error> {
        parser::parse_config(path)
    }

    pub fn autostart(&mut self) {
        for task in self.tasks.iter_mut() {
            if task.autostart == true {
                self.runnings.push(task.run())
            }
        }
    }

    pub fn dead_task(&mut self) {
		let mut index = 0;
		let mut added_tasks = vec!();

		// need to do that cause std::vec::Vec::retain_mut does not exist
		while index < self.runnings.len() {
			let task = &mut self.runnings[index];
			let status = match task.is_dead() {
				Some(status) => status,
				None => { index += 1; continue }
			};
			let conf = match self.tasks.iter_mut().find(|conf| conf.id == task.conf_id) {
				Some(conf) => conf,
				None => {
					eprintln!("{} just died and do not have a conf", task.name);
					self.runnings.remove(index);
					continue ;
				}
			};
			let running_time = task.start_time.elapsed().as_secs();
			if conf.should_restart(status) {
				println!("{} stopped in {}s with status: {:?}, restarting...", task.name, running_time, status);
				added_tasks.push(conf.run())
			} else {
				println!("{} stopped in {}s with status: {:?}", task.name, running_time, status);
			}
			self.runnings.remove(index);
		}
		self.runnings.append(&mut added_tasks);
	}
	
	pub fn ls(&self) -> String {
		if self.runnings.len() == 0 {
			return "nothing running".to_string();
		}
		let now = std::time::Instant::now();
		self.runnings.iter().map(|task| {
			format!("{} running since {} seconds.", task.name, now.duration_since(task.start_time).as_secs())
		}).collect::<Vec<String>>().join("\n")
	}

	pub fn conf(&self) -> String {
		if self.tasks.len() == 0 {
			return "empty_conf".to_string();
		}
		self.tasks.iter().map(|task| {
			format!("{}: {} {:?}", task.name, task.binary, task.args)
		}).collect::<Vec<String>>().join("\n")
	}

	pub fn start(&mut self, name: &str) -> String {
		if let Some(conf) = self.tasks.iter_mut().find(|task| task.name == name) {
			self.runnings.push(conf.run());
			format!("{} launched", name)
		} else {
			format!("Cannot find the {} task in the conf", name)
		}
	}

	pub fn stop(&mut self, name: &str) -> String {
		if let Some(task) = self.runnings.iter_mut().find(|task| task.name == name) {
			if let Err(e) = task.child.kill() {
				format!("Error while killing {}: {}", name, e)
			} else {
				format!("stopping {} ...", name)
			}		
		} else {
			format!("Cannot find {} in the running tasks", name)
		}
	}

	pub fn stop_all(&mut self, name: &str) -> String {
		if let Some(conf) = self.tasks.iter().find(|conf| conf.name == name) {
			let response = self.runnings.iter_mut()
				.filter(|running| running.conf_id == conf.id)
				.map(|running| {
					running.child.kill().ok();
					println!("killing {}", running.name);
					format!("Killing {} ...", running.name)
				}).collect::<Vec<String>>().join("\n");
			if response.len() == 0 {
				format!("No {} running task", name)
			} else {
				response
			}
		} else {
			format!("Cannot find any conf with the name {}", name)
		}
	}
}
