use crate::task::TaskConf;
use crate::Error;
use serde_derive::Deserialize;
use std::convert::From;
use std::fs;
use std::path::PathBuf;

fn parse_cmd(entry: &String) -> Vec<String> {
	let mut res = vec!();
	let mut actual_string = String::new();
	let mut dquote = false;
	let mut squote = false;
	let mut bslash = false;
	let mut space = false;

	for c in entry.chars() {
		match c {
			x if x == '\\' && !squote && !bslash => bslash = true,
			x if x == '\\' && !dquote && !squote && bslash => { bslash = false ; actual_string.push('\\'); },
			x if x == '\'' && !squote && !dquote && !bslash => squote = true,
			x if x == '\'' && squote => squote = false,
			x if x == '"' && dquote && !bslash => dquote = false,
			x if x == '"' && !dquote && !bslash && !squote => dquote = true,
			x if !x.is_whitespace() && space => { space = false ; res.push(actual_string) ; actual_string = x.to_string(); }
			x if x.is_whitespace() && !dquote && !squote && !bslash => space = true,
			x if x.is_whitespace() && bslash => { bslash = false ; actual_string.push(x); }
			x if (x == '\'' || x == '"' || x == '\\') && bslash => { bslash = false ; actual_string.push(c); }
			_ => { space = false ; actual_string.push(c); }
		}
	}
	if !actual_string.is_empty() {
		res.push(actual_string);
	}

	if dquote || squote || bslash {
		// TODO: log or exit because invalid command
	}
	res
}

#[derive(Deserialize, Debug)]
struct EnvVar {
	key: String,
	value: String
}

impl EnvVar {
	fn to_string(self) -> String {
		format!("{}={}", self.key, self.value)
	}
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum MaybeArray<T> {
    Alone(T),
    Multiple(Vec<T>),
}

fn to_vec<T>(src: MaybeArray<T>) -> Vec<T> {
	match src {
		MaybeArray::Alone(n) => vec!(n),
		MaybeArray::Multiple(n) => n
	}
}

fn default_env() -> MaybeArray<EnvVar> { MaybeArray::Multiple(vec!()) }
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
	env: MaybeArray<EnvVar>
}

impl LitteralTasks {
	fn parse(self) -> TaskConf {
		let args = parse_cmd(&self.cmd);
		let binary: String = args[0].clone();
		let args = if args.len() == 0 { None } else { Some(args)};
		let name = self.name.unwrap_or(binary.clone());
		let stdout = PathBuf::from(self.stdout.unwrap_or(format!("/tmp/{}.stdout", name)));
		let stderr =  PathBuf::from(self.stderr.unwrap_or(format!("/tmp/{}.stderr", name)));
		TaskConf {
			name,
			binary,
			args,
			numproc: self.numproc,
			umask: self.umask,
			workingdir: self.workingdir.as_ref().and_then(|e| Some(PathBuf::from(e))),
			autostart: self.autostart,
			autorestart: self.autorestart.into(),
			exitcodes: to_vec(self.exitcodes),
			startretries: self.startretries,
			startime: self.startime,
			stopsignal: 9, // TODO: parse str into u32 or signal enum
			stoptime: self.stoptime,
			stdout,
			stderr,
			env: to_vec(self.env).into_iter().map(EnvVar::to_string).collect(),
		}
	}
}

#[derive(Deserialize, Debug)]
struct LitteralConf {
	port: Option<u32>,
	tasks: Vec<LitteralTasks>
}

impl LitteralConf {
	fn parse(self) -> Conf {
		Conf {
			port: self.port.unwrap_or(6060),
			tasks: self.tasks.into_iter().map(LitteralTasks::parse).collect()
		}
	}
}

#[derive(Debug)]
pub struct Conf {
	port: u32,
	tasks: Vec<TaskConf>
}

impl Conf {
    pub fn new(path: String) -> Result<Conf, Error> {
        let file = fs::read_to_string(path)?;
        let conf: Conf = toml::from_str::<LitteralConf>(&file)?.into();
        Ok(dbg!(conf))
    }

    pub fn autostart(self) {
        for task in self.tasks.iter() {
            if task.autostart == true {
                task.run();
            }
        }
    }
}

#[cfg(test)]
mod config_tests {
	use super::*;

	#[test]
	fn parse_cmd_test() {
		let tests: Vec<(&str, Vec<&str>)> = vec!(
			(r#"/bin/ls 'lol'"#, vec!("/bin/ls", "lol")),
			(r#"/bin/ls\ mdr"#, vec!("/bin/ls mdr")),
			(r#"/bin/ls '"lol'"#, vec!("/bin/ls", "\"lol")),
			(r#""'"\'"""#, vec!(r#"''"#)),
			// TODO: more tests
		);
		for test in tests {
			assert_eq!(parse_cmd(&test.0.to_string()), test.1.iter().map(|e| e.to_string()).collect::<Vec<String>>())
		}
	}
}
