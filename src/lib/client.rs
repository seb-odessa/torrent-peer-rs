use Validate;
use PeerProto;
use Message;

use std::io;
use std::net::SocketAddr;
use std::collections::HashSet;

use futures::Future;
use tokio_core::reactor::Handle;
use tokio_proto::pipeline::ClientService;
use tokio_core::net::TcpStream;
use tokio_proto::TcpClient;
use tokio_service::Service;
use rustc_serialize::hex::ToHex;

fn create<T: Into<String>>(msg: T) -> Result<Client, io::Error> {
    return Err(io::Error::new(io::ErrorKind::Other, msg.into()));
}


pub type ClientConnection = Box<Future<Item = Client, Error = io::Error>>;
//pub type ClientResult = Box<Future<Item = (), Error = io::Error>>;

pub struct Client {
    inner: Validate<ClientService<TcpStream, PeerProto>>,
    pub done: bool,
    pub am_choked: bool,
    pub am_intrested: bool,
    pub peer_choked: bool,
    pub peer_intrested: bool,
    pub peer_have: HashSet<u32>,
}

impl Client {
    pub fn connect(addr: &SocketAddr, handle: &Handle) -> ClientConnection {
        Box::new(TcpClient::new(PeerProto).connect(addr, handle).map(
            |service| {
                Client {
                    inner: Validate { inner: service },
                    done: false,
                    am_choked: true,
                    am_intrested: false,
                    peer_choked: true,
                    peer_intrested: false,
                    peer_have: HashSet::new(),
                }
            },
        ))
    }

    pub fn handshake(self, hash_info: Vec<u8>, id: &[u8]) -> ClientConnection {
        let result = self.call(Message::Handshake(hash_info.clone(), Vec::from(id)))
            .and_then(move |response| match response {
                Message::Handshake(hash, _) => {
                    if hash == hash_info {
                        Ok(self)
                    } else {
                        return create(format!("expected {:?}", hash_info.to_hex()));
                    }
                }
                _ => return create("Unexpected response"),
            });
        Box::new(result)
    }

    fn process(&mut self, msg: Message) -> Result<(), io::Error> {
        match msg {
            Message::KeepAlive() => {}            
            Message::Choke() => self.am_choked = true,
            Message::Unchoke() => self.am_choked = false,
            Message::Interested() => self.peer_intrested = true,
            Message::NotInterested() => self.peer_intrested = false,            
            Message::Have(index) => {
                self.peer_have.insert(index);
            }
            Message::Bitfield(_) => {} // Not implemented
            Message::Request(_, _, _) => {
                // Not implemented
            } 
            Message::Piece(_, _, _) => {
                // Not implemented
                self.done = true;
            } 
            Message::Cancel(_, _, _) => {} // Not implemented
            Message::Port(_) => {} // Not implemented
            _ => return Err(io::Error::new(io::ErrorKind::Other, "Unexpected message")),
        }
        Ok(())
    }

    pub fn unchoke_me(mut self) -> ClientConnection {
        Box::new(self.call(Message::Interested()).and_then(|msg| {
            self.process(msg).and(Ok(self))
        }))
    }

    pub fn unchoke_peer(mut self) -> ClientConnection {
        Box::new(self.call(Message::Unchoke()).and_then(|msg| {
            self.process(msg).and(Ok(self))
        }))
    }

    pub fn request(mut self, index: u32, offset: u32, size: u32) -> ClientConnection {
        Box::new(self.call(Message::Request(index, offset, size)).and_then(
            |msg| {
                self.process(msg).and(Ok(self))
            },
        ))
    }

    pub fn ping(mut self) -> ClientConnection {
        Box::new(self.call(Message::KeepAlive()).and_then(|msg| {
            self.process(msg).and(Ok(self))
        }))
    }
}

impl Service for Client {
    type Request = Message;
    type Response = Message;
    type Error = io::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, request: Self::Request) -> Self::Future {
        // println!("Client: Request to Server: {:?}", request);
        Box::new(self.inner.call(request).and_then(|response| {
            // println!("Client: Response from Server: {:?}", response);
            Ok(response)
        }))
    }
}
