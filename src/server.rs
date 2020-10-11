use std::collections::HashMap;
use std::sync::{ Arc, mpsc::Sender };
use std::io::Read;
use mio::net::{ TcpListener, TcpStream };
use parking_lot::Mutex;
use mio::Poll;

use crate::event::{ Event, Service, Command };

pub struct Client {
    socket: TcpStream,
    buffer: Option<String>
}

impl Client {
    fn new(socket: TcpStream) -> Self {
        // let socket = Mutex::new(socket);
        Client { socket, buffer: None }
    }

    // fn write(&self, data: String) {
    //     // let socket = self.socket.lock();
    //     if let Err(e) = self.socket.write(data.as_bytes()) {
    //         eprintln!("Error while writing to client : {}", e);
    //     }
    // }
}

fn parse_cmd(incomming: &&str) -> Option<Command> {
    if incomming.is_empty() {
        return None
    }
    let args: Vec<&str> = incomming.split_whitespace().collect();
    match args[0] {
        "ls" => Some(Command::Ls),
        _ => {
            eprintln!("not recognized command");
            None
        }
    }
}

// Would block "errors" are the OS's way of saying that the
// connection is not actually ready to perform this I/O operation.
fn would_block(err: &std::io::Error) -> bool {
    err.kind() == std::io::ErrorKind::WouldBlock
}

fn interrupted(err: &std::io::Error) -> bool {
    err.kind() == std::io::ErrorKind::Interrupted
}

fn next(current: &mut mio::Token) -> mio::Token {
    let next = current.0;
    current.0 += 1;
    mio::Token(next)
}

fn handle_connection_event(client: &mut Client, event: &mio::event::Event, token: mio::Token, sender: &Sender<Event>) -> bool {
    if !event.is_readable() {
        eprintln!("Error, event should be readable");
    }
    let mut connection_closed = false;
    let mut first_turn = true;
    let mut received_data = Vec::with_capacity(4096);
    // let socket = client.socket.lock();
    loop {
        let mut buf = [0; 256];
        match client.socket.read(&mut buf) {
            Ok(0) => {
                connection_closed = true;
                break;
            }
            Ok(n) => {
                first_turn = false;
                received_data.extend_from_slice(&buf[..n])
            },
            Err(ref e) if would_block(e) => {
                eprintln!("received a would block error: {}", e);
                break
            }
            Err(ref e) if interrupted(e) => {
                eprintln!("Received a is interrupted error: {}", e);
                continue
            }
            Err(e) => {
                eprintln!("Received a strange error while reading the client: {}", e);
                return true;
            }
        }
    }

    if let Ok(mut str_buf) = String::from_utf8(received_data) {
        if let Some(saved) = &client.buffer {
            str_buf = format!("{}{}", saved, str_buf);
            client.buffer = None;
        }
        println!("Received data: {}", str_buf.trim_end());
        let bufferise = str_buf.ends_with('\n');
        let mut cmds: Vec<&str> = str_buf.split('\n').collect();
        if bufferise {
            client.buffer = cmds.pop().and_then(|a| Some(a.to_string()))
        }
        cmds.iter().filter_map(parse_cmd).for_each(|cmd| { sender.send(Event::Cmd(token, cmd)).ok(); })
    } else {
        println!("Received (none UTF-8) data");
    }

    if connection_closed && first_turn {
        println!("Connection closed");
        return true;
    }
    false
}

const SERVER: mio::Token = mio::Token(0);

fn init_server(address: &String) -> Result<(mio::Poll, TcpListener, mio::Events), std::io::Error> {
    let poll = Poll::new()?;
    let events = mio::Events::with_capacity(128);
    
    let addr = address.parse().map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
    
    let mut server = TcpListener::bind(addr)?;
    
    
    // Register the server socket for readable event (incomming connection)
    poll.registry().register(&mut server, SERVER, mio::Interest::READABLE)?;
    Ok((poll, server, events))
}

pub fn server(address: String, sender: Sender<Event>, clients: Arc<Mutex<HashMap<mio::Token, Client>>>) {
    let (mut poll, server, mut events) = match init_server(&address) {
        Ok(args) => args,
        Err(e) => {
            sender.send(Event::Abort(format!("Error while initialising server: {}", e))).ok();
            return ;
        }
    };
    let mut unique_token = mio::Token(SERVER.0 + 1);
    sender.send(Event::Ready(Service::RpcServer)).ok();
    loop {
        if let Err(e) = poll.poll(&mut events, None) {
            sender.send(Event::Abort(format!("Polling error: {}", e))).ok();
            return ;
        }
        for event in events.iter() {
            match event.token() {
                SERVER => {
                    let (mut connection, address) = match server.accept() {
                        Ok((connection, address)) => (connection, address),
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break, // Empty imcomming connections queue
                        Err(e) => {
                            sender.send(Event::Error(format!("server accept connection error: {}", e))).ok();
                            continue;
                        }
                    };
                    println!("Accepted connection from: {}", address);
                    let token = next(&mut unique_token);
                    if let Err(e) = poll.registry().register(
                        &mut connection,
                        token,
                        mio::Interest::READABLE,
                    ) {
                        sender.send(Event::Error(format!("cannot register user: {}",e ))).ok();
                        continue;
                    }
                    let mut clients = clients.lock();
                    clients.insert(token, Client::new(connection));
                },
                token => {
                    let mut clients = clients.lock();
                    let done = if let Some(mut client) = clients.get_mut(&token) {
                        handle_connection_event(&mut client, event, token, &sender)
                    } else {
                        false // Sporadic events happen, we can safely ignore them.
                    };
                    if done {
                        clients.remove(&token);
                    }
                }
            }
        }
    }
}