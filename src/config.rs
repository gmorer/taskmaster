use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;
use crate::task::{ TaskConf, RunningTask, Autorestart };
use crate::Error;
use crate::parser;
use crate::task;

#[derive(Debug)]
pub struct Conf {
    pub conf_id: usize, // change on every new tasks (+1)
    pub address: String,
    pub tasks: Vec<TaskConf>, // Mutex or RWlock ?
    pub runnings: Vec<RunningTask>,
}

fn find_dead(runnings: &mut Vec<task::RunningTask>) -> Option<(ExitStatus, usize, &mut RunningTask)> {
    for (index, task) in &mut runnings.iter_mut().enumerate() {
        match task.child.try_wait() {
            Ok(Some(status)) => { return Some((status, index, task)) },
            Ok(None) => { /* not hti sone */ },
            Err(e) => eprintln!("child.try_wait() error: {}", e)
        }
    }
    eprintln!("THe dead is not mine");
    None
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
        let found = find_dead(&mut self.runnings);
        if found.is_none() {
            return ;
        }
        let (status, task_index, task) = found.unwrap();
        let conf = self.tasks.iter_mut().find(|conf| conf.id == task.conf_id).expect("this task do not have a conf");
        let running_time = task.start_time.elapsed().as_secs();
        let restart = if let Some(code) = status.code() {
            println!("{} stopped in {}s with exitcode: {}", task.name, running_time, code);
            conf.autorestart == Autorestart::Always || (!status.success() && conf.autorestart == Autorestart::Unexpected)
        } else if let Some(signal) = status.signal() {
            println!("{} stopped in {}s with signal: {}", task.name, running_time, signal);
            conf.autorestart == Autorestart::Always || conf.autorestart == Autorestart::Unexpected
        } else {
            eprintln!("{} stopped in {}s with neither a signal neither an exitcode", task.name, running_time);
            false
        };

        self.runnings.remove(task_index);
        if restart {
            self.runnings.push(conf.run())
        }
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
