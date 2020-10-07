use crate::config::Conf;

pub enum Event {
    FromChild(libc::c_int),
    // ...
}

pub fn execut(e: &Event, conf: &mut Conf) {
    match e {
        Event::FromChild(e) => {
            match *e {
                libc::SIGCHLD => conf.dead_task(),
                libc::SIGHUP => (),
                libc::SIGTERM => (),
                _ => ()
            }
        }
    }
}