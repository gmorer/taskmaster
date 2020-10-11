use std::{ mem, ptr };
use std::sync::mpsc::Sender;

use crate::event::{ Event, Service };

/*
	Signals handling:
		SIGCHILD: a child died
		SIGHUP: reload conf
		SIGTERM: kill all the childs
*/

pub type Sigset = libc::sigset_t;

pub fn create_sigset() -> Sigset {
    unsafe {
        let mut set: libc::sigset_t = mem::zeroed();
        if libc::sigemptyset(&mut set) == -1 {
            unimplemented!("error sigemptyset");
        }
        if libc::sigaddset(&mut set, libc::SIGCHLD) == -1 {
            unimplemented!("error sigaddset");
        }
        if libc::sigaddset(&mut set, libc::SIGHUP) == -1 {
            unimplemented!("error sigaddset");
        }
        if libc::sigaddset(&mut set, libc::SIGTERM) == -1 {
            unimplemented!("error sigaddset");
        }
        let err = libc::pthread_sigmask(libc::SIG_BLOCK, &set, ptr::null_mut());
        if err != 0 {
            unimplemented!("error pthread_sigmask");
        }
        set
    }
}

pub fn signal_handler(set: &Sigset, sender: Sender<Event>) {
    let mut sig: libc::c_int = 0;
    sender.send(Event::Ready(Service::SignalHandler)).ok();
    loop {
        unsafe {
            if libc::sigwait(set, &mut sig) != 0 {
                unimplemented!("error sigwait")
            }
        }
        println!("got a signal: {}", sig);
        sender.send(Event::FromChild(sig)).ok();
    }
}