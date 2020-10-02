use std::time::Instant;

use crate::task::{ TaskConf, RunningTask, Autorestart };
use crate::Error;
use crate::parser;

#[derive(Debug)]
pub struct Conf {
    pub conf_id: usize, // change on every new tasks (+1)
    pub port: u32,
    pub tasks: Vec<TaskConf>, // Mutex or RWlock ?
    pub runnings: Vec<RunningTask>,
}

// TODO: wrapper around status for less unsafe in safe code

fn triger_unexpected(status: libc::c_int, exitcode: &Vec<i32>) -> bool {
    if unsafe { libc::WIFEXITED(status) } {
        exitcode.iter().any(|code| code == unsafe { &libc::WEXITSTATUS(status) })
    } else {
        false
    }
}

impl Conf {
    pub fn new(path: String) -> Result<Conf, Error> {
        parser::parse_config(path)
    }

    pub fn autostart(&mut self) {
        for task in self.tasks.iter_mut() {
            if task.autostart == true {
                self.runnings.push(task.run(0))
            }
        }
    }

    pub fn dead_task(&mut self) {
        let mut status: libc::c_int = 5;
        let pid = unsafe { libc::waitpid(-1, &mut status, libc::WNOHANG | libc::WUNTRACED | libc::WCONTINUED) };
        println!("status: {:?}", status);
        let (task_index, task) = self.runnings.iter().enumerate().find(|task| task.1.pid == pid).expect("A child die but not mine");
        if unsafe { libc::WIFEXITED(status) } {
            println!("{} Terminated with exit code {}", task.name, unsafe { libc::WEXITSTATUS(status) })
        } else if unsafe { libc::WIFSIGNALED(status) } {
            println!("{} Terminated with signal {}", task.name, unsafe { libc::WTERMSIG(status) })
        } else {
            // Stop or continue signals
            return;
        }
        let conf = self.tasks.iter_mut().find(|conf| conf.id == task.conf_id).expect("this task do not have a conf");
        let running_time = Instant::now() - task.start_time;
        if running_time.as_secs() < conf.startime as u64 {
            println!("{} aborted during the startup", task.name);
            if conf.startretries > 0 && task.respawn_no >= conf.startretries {
                println!("{} aborted due to too mutch start retries", task.name);
            } else if conf.startretries > 0 {
                println!("Restarting {}", conf.name);
                let respawn = task.respawn_no + 1;
                self.runnings.push(conf.run(respawn))
            }
        } else {
            match conf.autorestart {
                Autorestart::Always => {
                    println!("Restarting {}", conf.name);
                    self.runnings.push(conf.run(0));
                }
                Autorestart::Unexpected => {
                    if triger_unexpected(status, &conf.exitcodes) {
                        println!("{} unexpected crash, will be relaunch", conf.name);
                        self.runnings.push(conf.run(0));
                    }
                }
                _ => { /* Do nothing the task just died */ }
            }
        }
        self.runnings.remove(task_index);
    }
}
