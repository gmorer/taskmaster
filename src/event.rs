use crate::{ server, Clients };
use crate::config::Conf;

#[derive(Debug)]
pub enum Command {
    Ls,
}

#[derive(Debug)]
pub enum Service {
    RpcServer,
    SignalHandler,
}

#[derive(Debug)]
pub enum Event {
    FromChild(libc::c_int),
    Error(String),
    Log(String),
    Abort(String),
    Ready(Service),
    Cmd(usize, Command)
    // ...
}

pub const MAX_SERVICE: u32 = 2;

pub fn process_cmd(token: &usize, cmd: &Command, clients: &Clients, waker: &mut server::Waker) {
    let res = match cmd {
        Command::Ls => "this was an ls"
    };
    if let Some(client) = clients.lock().get_mut(token) {
		waker.wake();
		client.add_queue(res);
	} else {
		eprintln!("client has been removed during the command");
	}
}

pub fn execut(e: &Event, conf: &mut Conf, started: &mut u32, clients: &Clients, waker: &mut server::Waker) {
    println!("{:?}", e);
    match e {
        Event::FromChild(e) => {
            match *e {
                libc::SIGCHLD => conf.dead_task(),
                libc::SIGHUP => (),
                libc::SIGTERM => (),
                _ => ()
            }
        }
        Event::Ready(Service::SignalHandler) => {
            *started += 1;
            if *started == MAX_SERVICE {
                conf.autostart();
            }
        },
        Event::Ready(Service::RpcServer) => {
            *started += 1;
            if *started == MAX_SERVICE {
                conf.autostart();
            }
        },
        Event::Cmd(token, cmd) => process_cmd(token, cmd, clients, waker),
        Event::Error(e) => eprintln!("{}", e),
        Event::Log(e) => println!("{}", e),
        Event::Abort(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}