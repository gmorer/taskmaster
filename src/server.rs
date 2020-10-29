use std::sync::mpsc::Sender;
use std::io::Read;
use std::net::{ TcpListener, TcpStream };
use std::os::unix::net::UnixStream;
use std::os::unix::io::RawFd;
use std::io::Write;
use polling::Poller;
use crate::event::{ Event, Service, Command };
use std::os::unix::io::AsRawFd;

pub struct Waker(UnixStream);

impl Waker {
    pub fn create() -> std::io::Result<(Self, Self)> {
        let (peer1, peer2) = UnixStream::pair()?;
        Ok((Self(peer2), Self(peer1)))
    }

    pub fn wake(&mut self) {
		println!("wakinng");
        self.0.write(&[1]).ok();
    }
}

impl std::os::unix::io::AsRawFd for Waker {
    fn as_raw_fd(&self) ->  std::os::unix::io::RawFd {
        self.0.as_raw_fd()
    }
}

// impl std::os::unix::io::IntoRawFd for Waker {
//     fn into_raw_fd(self) ->  std::os::unix::io::RawFd {
//         self.0.into_raw_fd()
//     }
// }


pub struct Client {
    socket: TcpStream,
	buffer: Option<String>,
	pub queue: Vec<String>
}

impl Client {
    pub fn new(socket: TcpStream) -> Self {
        // let socket = Mutex::new(socket);
        Self { socket, buffer: None, queue: Vec::new() }
	}
	
	pub fn add_queue(&mut self, result: &str) {
		self.queue.push(String::from(result));
	}

    fn write(&mut self) -> std::io::Result<usize> {
		if self.queue.len() == 0 {
			return Ok(0); // notting to write
		}
		let mut index = 0;
		for message in self.queue.iter() {
			match self.socket.write(message.as_bytes()) {
				Ok(0) => { /* is that normal? */},
				Ok(_) => index += 1,
				Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    eprintln!("received a would block error: {}", e);
					break;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => {
                    eprintln!("Received a is interrupted error: {}", e);
                }
				Err(e) => {
					eprintln!("Error while writting: {}", e);
					break;
				}
			}
		}
		self.queue = self.queue.split_off(index);
		Ok(index)
    }

    //return None if you should close the socket, the readed size otherwise
    pub fn read(&mut self, buffer: &mut Vec<u8>) -> Option<usize> {
        let mut size = 0;
        loop {
            let mut buf = [0; 256];
            match self.socket.read(&mut buf) {
                Ok(0) if size == 0 => return None,
                Ok(0) => return Some(size),
                Ok(n) => {
                    size += n;
                    buffer.extend_from_slice(&buf[..n]);
                    if buf.contains(&('\n' as u8)) {
                        return Some(size);
                    }
                },
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    eprintln!("received a would block error: {}", e);
                    break;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => {
                    eprintln!("Received a is interrupted error: {}", e);
                }
                Err(e) => {
                    eprintln!("Received a strange error while reading the client: {}", e);
                    break; // not sure this is the good things to do
                }
            }
        }
        Some(size)
    }
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

// return true if you should close the socket, false otherwise
fn handle_readable_event(client: &mut Client, event: &polling::Event, token: usize, sender: &Sender<Event>) -> bool {
    if !event.readable {
        eprintln!("Error, event should be readable");
    }
    let mut received_data = Vec::with_capacity(4096);

    match client.read(&mut received_data) {
        Some(0) => return false,
        Some(n) => n,
        None => return true,
    };

    if let Ok(mut str_buf) = String::from_utf8(received_data) {
        if let Some(saved) = &client.buffer {
            str_buf.insert_str(0, saved);
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
    false
}

const SERVER_KEY: usize = 0;
const WAKER_KEY: usize = 1;

fn set_fds(poll: &polling::Poller, clients: &std::collections::HashMap<usize, Client>) -> Result<(), std::io::Error> {
	for (token, client) in clients.iter() {
		if client.queue.len() > 0 {
			poll.modify(&client.socket, polling::Event::all(*token)).ok();
		} else {
			poll.modify(&client.socket, polling::Event::readable(*token)).ok();
		}
	}
	Ok(())
}

fn init_server(address: &String, waker: &Waker) -> Result<(Poller, TcpListener), std::io::Error> {
    // Create a TCP listener.
    let socket = TcpListener::bind(address)?;
    socket.set_nonblocking(true)?;

    // Create a poller and register interest in readability on the socket.
    let poller = Poller::new()?;
    poller.add(&socket, polling::Event::readable(SERVER_KEY))?;
    poller.add(waker.0.as_raw_fd(), polling::Event::readable(WAKER_KEY))?;
    Ok((poller, socket))
}

pub fn server(address: String, sender: Sender<Event>, clients: std::sync::Arc<crate::Clients>, waker: Waker) {
    let (poll, server) = match init_server(&address, &waker) {
        Ok(args) => args,
        Err(e) => {
            sender.send(Event::Abort(format!("Error while initialising server: {}", e))).ok();
            return ;
        }
    };
    let client_id = WAKER_KEY + 1;
    sender.send(Event::Ready(Service::RpcServer)).ok();
    let mut events = Vec::new();
    loop {
		events.clear();
        if let Err(e) = poll.wait(&mut events, None /* Timeout */) {
            sender.send(Event::Abort(format!("Polling error: {}", e))).ok();
            return ;
		}
    	for event in &events {
    	    match event.key {
    	        SERVER_KEY => {
    	            let (connection, address) = match server.accept() {
    	                Ok(args) => args,
    	                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue, // Empty imcomming connections queue
    	                Err(e) => {
    	                    sender.send(Event::Error(format!("server accept connection error: {}", e))).ok();
    	                    continue;
    	                }
    	            };
    	            println!("Accepted connection from: {}", address);
    	            if let Err(e) = poll.add(&connection, polling::Event::readable(client_id)) {
    	                sender.send(Event::Error(format!("cannot register user: {}",e ))).ok();
    	                continue;
					}
					let mut clients = clients.lock();
    	            clients.insert(client_id, Client::new(connection));
					poll.modify(&server, polling::Event::readable(SERVER_KEY)).ok();
    	        },
    	        WAKER_KEY => {
					/* navigate throught all client, look for waiting output */
					let clients = clients.lock();
					set_fds(&poll, &*clients);
					poll.modify(&waker, polling::Event::readable(WAKER_KEY)).ok();
    	        },
    	        token => {
					let mut clients = clients.lock();
    	            let client = match clients.get_mut(&token) {
    	                Some(client) => client,
    	                None => continue
    	            };
    	            let should_write = false;
    	            if event.readable && handle_readable_event(client, event, token, &sender) {
    	                // client closed connection or error
    	                poll.delete(&client.socket).ok();
    	                clients.remove(&token);
    	                continue;
					}
					if event.writable {
						if client.write().is_ok() {
							poll.modify(&client.socket, polling::Event::readable(token)).ok();
						}
    	            }
    	            if should_write {
    	                poll.modify(&client.socket, polling::Event::all(token)).ok();
    	            } else {
    	                poll.modify(&client.socket, polling::Event::readable(token)).ok();
    	            }
    	        }
			}
		}
    }
}