use Validate;
use PeerProto;
use Message;
use Messages;

use std::io;
use std::net::SocketAddr;
use std::collections::HashSet;
use std::collections::HashMap;

use futures::Future;
use tokio_core::reactor::Handle;
use tokio_proto::pipeline::ClientService;
use tokio_core::net::TcpStream;
use tokio_proto::TcpClient;
use tokio_service::Service;
// use rustc_serialize::hex::ToHex;


pub type ClientConnection = Box<Future<Item = Client, Error = io::Error>>;

pub struct Client {
    inner: Validate<ClientService<TcpStream, PeerProto>>,
    pub am_choked: bool,
    pub am_intrested: bool,
    pub peer_choked: bool,
    pub peer_intrested: bool,
    pub peer_have: HashSet<u32>,
    pub peer_requests: HashSet<(u32, u32, u32)>,
    pub blocks: HashMap<(u32, u32), Vec<u8>>,
    pub messages: Messages,
    pub info_hash: Vec<u8>,
}

impl Client {
    pub fn connect(addr: &SocketAddr, handle: &Handle) -> ClientConnection {
        Box::new(TcpClient::new(PeerProto).connect(addr, handle).map(
            |service| {
                Client {
                    inner: Validate { inner: service },
                    am_choked: true,
                    am_intrested: false,
                    peer_choked: true,
                    peer_intrested: false,
                    peer_have: HashSet::new(),
                    peer_requests: HashSet::new(),
                    blocks: HashMap::new(),
                    messages: Messages::new(),
                    info_hash: Vec::new(),
                }
            },
        ))
    }

    pub fn handshake(mut self, info_hash: Vec<u8>, id: &[u8]) -> ClientConnection {
        self.info_hash = info_hash.clone();
        let msg = Message::Handshake(info_hash, Vec::from(id));
        Box::new(self.call(msg).and_then(
            |msgs| self.enqueue(msgs).and(Ok(self)),
        ))
    }

    fn enqueue(&mut self, mut messages: Messages) -> Result<(), io::Error> {
        self.messages.append(&mut messages);
        self.dispatch()
    }

    pub fn dispatch(&mut self) -> Result<(), io::Error> {
        println!("client::dispatch() START");
        while let Some(message) = self.messages.pop_front() {
            if let Some(msg) = message {
                self.process(msg)?;
            }
        }
        println!("client::dispatch() END");
        Ok(())
    }

    fn process(&mut self, msg: Message) -> Result<(), io::Error> {
        println!("Client::process() <= {}", msg);
        match msg {
            Message::Handshake(info_hash, _) => {
                if self.info_hash != info_hash {
                    return Err(io::Error::new(io::ErrorKind::Other, "Unexpected INFO hash"));
                }
            }

            Message::KeepAlive() => {
                // Just a ping
            }
            Message::Choke() => self.am_choked = true,
            Message::Unchoke() => self.am_choked = false,
            Message::Interested() => self.peer_intrested = true,
            Message::NotInterested() => self.peer_intrested = false,
            Message::Have(index) => {
                self.peer_have(index);
            }
            Message::Bitfield(bits) => {
                self.create_peer_have(bits);
            }
            Message::Request(index, offset, length) => {
                self.peer_requests.insert((index, offset, length));
            }
            Message::Piece(index, offset, block) => {
                self.blocks.insert((index, offset), block);
            }
            Message::Cancel(index, offset, length) => {
                self.peer_requests.remove(&(index, offset, length));
            }
            Message::Port(_) => {
                // Not implemented
            }
            //_ => return Err(io::Error::new(io::ErrorKind::Other, "Unexpected message")),
        }
        Ok(())
    }

    fn create_peer_have(&mut self, bits: Vec<u8>) {
        let mut index = 0;
        for byte in &bits {
            if 0 != *byte & 0b1000_0000u8 {
                self.peer_have(index + 0);
            }
            if 0 != *byte & 0b0100_0000u8 {
                self.peer_have(index + 1);
            }
            if 0 != *byte & 0b0010_0000u8 {
                self.peer_have(index + 2);
            }
            if 0 != *byte & 0b0001_0000u8 {
                self.peer_have(index + 3);
            }
            if 0 != *byte & 0b0000_1000u8 {
                self.peer_have(index + 4);
            }
            if 0 != *byte & 0b0000_0100u8 {
                self.peer_have(index + 5);
            }
            if 0 != *byte & 0b0000_0010u8 {
                self.peer_have(index + 6);
            }
            if 0 != *byte & 0b0000_0001u8 {
                self.peer_have(index + 7);
            }
            index += 8;
        }
    }

    pub fn peer_have(&mut self, index: u32) {
        self.peer_have.insert(index);
    }

    pub fn unchoke_me(mut self) -> ClientConnection {
        Box::new(self.call(Message::Interested()).and_then(|msgs| {
            self.enqueue(msgs).and(Ok(self))
        }))
    }

    pub fn unchoke_peer(mut self) -> ClientConnection {
        Box::new(self.call(Message::Unchoke()).and_then(|msgs| {
            self.enqueue(msgs).and(Ok(self))
        }))
    }

    pub fn request(mut self, index: u32, offset: u32, size: u32) -> ClientConnection {
        Box::new(self.call(Message::Request(index, offset, size)).and_then(
            |msgs| {
                self.enqueue(msgs).and(Ok(self))
            },
        ))
    }

    pub fn ping(mut self) -> ClientConnection {
        Box::new(self.call(Message::KeepAlive()).and_then(|msgs| {
            self.enqueue(msgs).and(Ok(self))
        }))
    }
}

impl Service for Client {
    type Request = Message;
    type Response = Messages;
    type Error = io::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, request: Self::Request) -> Self::Future {
        println!("Client: Request to Server: {:?}", request);
        Box::new(self.inner.call(request).and_then(|response| {
            println!("Client: Response from Server: {:?}", response);
            Ok(response)
        }))
    }
}
