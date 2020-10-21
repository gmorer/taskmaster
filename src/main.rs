use std::{ thread, env };
use std::collections::HashMap;
use std::sync::{ Arc, mpsc::channel };
use parking_lot::Mutex;

mod task;
mod parser;

mod signals;
use signals::{ create_sigset, signal_handler };

mod config;
use config::Conf;

mod event;
use event::execut;

mod server;
use server::{ server, Waker };

/*
    Program threads: 1 for signal, 1 for rpc server, 1 one executing commands // TODO: think of sli
*/

type Clients = Arc<Mutex<HashMap<usize, server::Client>>>;
type Error = Box<dyn std::error::Error>;

const DFL_ADDRESS: &str = "localhost:6061";
const DFL_CONF: &str = "/home/tet/project/taskmaster/example.toml";

fn main() -> Result<(), Error> {
    let conf_file = env::args().nth(1).unwrap_or(DFL_CONF.to_string());
    println!("Conf file: {}", conf_file);
    let mut config = Conf::new(conf_file)?;
    let (tx, rx) = channel();


    let (listening_waker, emiting_waker) = Waker::create()?;
    /* Handle signals in an another thread */
    let sigset = create_sigset();
    let tx_signal = tx.clone();
    thread::spawn(move || {
        signal_handler(&sigset, tx_signal);
    });

    /* RPC server */
    let tx_server = tx.clone();
    let address = config.address.clone();
    let clients: Clients = Arc::new(Mutex::new(std::collections::HashMap::new()));
    let clients_clone = Arc::clone(&clients);
    thread::spawn(move || {
        server(address, tx_server, clients_clone, listening_waker)
    });

    let mut started = 0;
    loop {
        match rx.recv() {
            Ok(ev) => execut(&ev, &mut config, &mut started, &*clients),
            Err(err) => eprintln!("channel error: {}", err)
        }
    }
}