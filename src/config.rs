use std::fs;
use std::path::PathBuf;
use std::convert::From;
use serde_derive::Deserialize;
use crate::Error;
use crate::task::TaskConf;

fn convert_env(env: Vec<String>) -> Vec<(String, String)> {
	env.iter()
	.filter(|e| {
		// Remove entry with multiple or without '='
		match e.find('=') {
			Some(r) => Some(r) == e.rfind('='),
			None => false
		}
	})
	.map(|e| {
		// Create tuple from string=string
		let mut res = e.split('='); // length should be 2 with the test before
		(
			res.next().expect("i fk up").to_string(),
			res.next().expect("i fk up").to_string()
		)
	}).collect()
}

#[derive(Deserialize, Debug)]
struct LitteralConf {
	port: Option<u32>,
	tasks: Vec<LitteralTasks>
}

impl Into<Conf> for LitteralConf {
	fn into(self) -> Conf {
		Conf {
			port: self.port.unwrap_or(6060),
			tasks: self.tasks.iter().map(From::from).collect()
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
enum MaybeArray<T> {
	Alone(T),
	Multiple(Vec<T>)
}

impl<T> Into<Vec<T>> for MaybeArray<T> {
	fn into(self) -> Vec<T> {
		match self {
			MaybeArray::Alone(n) => vec!(n),
			MaybeArray::Multiple(n) => n
		}
	}
}

fn default_env() -> MaybeArray<String> { MaybeArray::Multiple(vec!()) }
fn default_alone_zero() -> MaybeArray<i32> { MaybeArray::Alone(0) }
fn default_autorestart() -> String { "false".to_string() }
fn default_term() -> String { "TERM".to_string() }
fn default_false() -> bool { false }
fn default_umask() -> u32 { 777 }
fn default_five() -> u32 { 5 }
fn default_one() -> u32 { 1 }

#[derive(Deserialize, Debug)]
struct LitteralTasks {
	cmd: String,
	name: Option<String>,
	#[serde(default = "default_one")]
	numproc: u32,
	#[serde(default = "default_umask")]
	umask: u32,
	workingdir: Option<String>,
	#[serde(default = "default_false")]
	autostart: bool,
	#[serde(default = "default_autorestart")]
	autorestart: String,
	#[serde(default = "default_alone_zero")]
	exitcodes: MaybeArray<i32>,
	#[serde(default = "default_five")]
	startretries: u32,
	#[serde(default = "default_one")]
	startime: u32,
	#[serde(default = "default_term")]
	stopsignal: String,
	#[serde(default = "default_one")]
	stoptime: u32,
	stdout: Option<String>,
	stderr: Option<String>,
	#[serde(default = "default_env")]
	env: MaybeArray<String>
}

#[derive(Debug)]
pub struct Conf {
	port: u32,
	tasks: Vec<TaskConf>
}

impl From<&LitteralTasks> for TaskConf {
	fn from(w: &LitteralTasks) -> TaskConf {
		let cmds: Vec<&str> = w.cmd.split_whitespace().collect();
		let name: String = w.name.clone().unwrap_or(cmds[0].to_string());
		TaskConf {
			binary: cmds[0].to_string(),
			args: cmds.iter().skip(1).map(|e| e.to_string()).collect(),
			numproc: w.numproc,
			umask: w.umask,
			workingdir: w.workingdir.as_ref().and_then(|e| Some(PathBuf::from(e))),
			autostart: w.autostart,
			autorestart: w.autorestart.clone().into(),
			exitcodes: w.exitcodes.clone().into(),
			startretries: w.startretries,
			startime: w.startime,
			stopsignal: 9, // TODO: parse str into u32 or signal enum
			stoptime: w.stoptime,
			stdout: PathBuf::from(w.stdout.clone().unwrap_or(format!("/tmp/{}.stdout", name))),
			stderr: PathBuf::from(w.stderr.clone().unwrap_or(format!("/tmp/{}.stderr", name))),
			env: convert_env(w.env.clone().into()),
			name: name
		}
	}
}

impl Conf {
	pub fn new(path: String) -> Result<(), Error> {
		let file = fs::read_to_string(path)?;
		let conf: Conf = toml::from_str::<LitteralConf>(&file)?.into();
		dbg!(conf);
		Ok(())
	}
}

#[cfg(test)]
mod config_tests {
	use super::*;
	#[test]
	fn convert_env_test() {
		// fn convert_env(env: Vec<String>) -> Vec<(String, String)> {
		assert_eq!(
			convert_env(vec!("FOO=BAR".to_string())),
			vec!(("FOO".to_string(), "BAR".to_string()))
		);
		assert_eq!(
			convert_env(vec!("FOO=BAR".to_string(), "BAOBAB".to_string())),
			vec!(("FOO".to_string(), "BAR".to_string()))
		);
		assert_eq!(
			convert_env(vec!("FOO=BAR".to_string(), "BA=OBA=B".to_string())),
			vec!(("FOO".to_string(), "BAR".to_string()))
		);
		assert_eq!(
			convert_env(vec!("FOO=BAR".to_string(), "BA=OBA=B".to_string(), "SPONGE=BOB".to_string())),
			vec!(("FOO".to_string(), "BAR".to_string()), ("SPONGE".to_string(), "BOB".to_string()))
		);
	}
}