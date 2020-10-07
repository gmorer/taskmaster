use std::{ thread, env };
use std::sync::mpsc::channel;

mod task;
mod parser;
mod signals;
use signals::{ create_sigset, signal_handler };

mod config;
use config::Conf;

mod event;
use event::execut;

/*
    Program threads: 1 for signal, 1 for rpc server, 1 one executing commands // TODO: think of sli
*/

type Error = Box<dyn std::error::Error>;

const DFL_CONF: &str = "/home/tet/project/taskmaster/example.toml";

fn main() -> Result<(), Error> {
    let conf_file = env::args().nth(1).unwrap_or(DFL_CONF.to_string());
    println!("Conf file: {}", conf_file);
    let mut config = Conf::new(conf_file)?;
    config.autostart();
    let (tx, rx) = channel();

    let sigset = create_sigset();
    thread::spawn(move || {
        signal_handler(&sigset, tx.clone());
    });

    loop {
        match rx.recv() {
            Ok(ev) => execut(&ev, &mut config),
            Err(err) => eprintln!("channel error: {}", err)
        }
    }
}