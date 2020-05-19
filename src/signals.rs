use std::{ mem, ptr };

use crate::config::Conf;

/*
	Signals handling:
		SIGCHILD: a child died
		SIGHUP: reload conf
		SIGTERM: kill all the childs
*/

pub type Sigset = libc::sigset_t;

fn child_death(conf: &Conf) {
	let mut status: libc::c_int = 0;
	let pid = unsafe { libc::waitpid(-1, &mut status, libc::WNOHANG) };
	conf.dead_task(pid);
}

pub fn create_sigset() -> Sigset {
	unsafe {
		let mut set: libc::sigset_t = mem::zeroed();
		if libc::sigemptyset(&mut set) == -1 {
			unimplemented!("error handling");
		}
		if libc::sigaddset(&mut set, libc::SIGCHLD) == -1 {
			unimplemented!("error handling");
		}
		if libc::sigaddset(&mut set, libc::SIGHUP) == -1 {
			unimplemented!("error handling");
		}
		if libc::sigaddset(&mut set, libc::SIGTERM) == -1 {
			unimplemented!("error handling");
		}
		let err = libc::pthread_sigmask(libc::SIG_BLOCK, &set, ptr::null_mut());
		if err != 0 {
			unimplemented!("error handling");
		}
		set
	}
}

pub fn signal_handler(set: &Sigset, tasks: &Conf) {
	let mut sig: libc::c_int = 0;
	loop {
		unsafe {
			if libc::sigwait(set, &mut sig) != 0 {
				unimplemented!("error handling")
			}
		}
		match sig {
			libc::SIGCHLD => child_death(tasks),
			libc::SIGHUP => (),
			libc::SIGTERM => (),
			_ => ()
		}
		println!("got a signal: {}", sig);
	}
}