use Validate;
use PeerProto;
use Message;
use make_error;

use std::io;
use std::net::SocketAddr;

use futures::Future;
use tokio_core::reactor::Handle;
use tokio_proto::pipeline::ClientService;
use tokio_core::net::TcpStream;
use tokio_proto::TcpClient;
use tokio_service::Service;
use rustc_serialize::hex::ToHex;

pub type ClientConnection = Box<Future<Item = Client, Error = io::Error>>;
pub type ClientResult = Box<Future<Item = (), Error = io::Error>>;


pub struct Client {
    inner: Validate<ClientService<TcpStream, PeerProto>>,
    a: bool,
}

impl Client {
    pub fn connect(addr: &SocketAddr, handle: &Handle) -> ClientConnection {
        Box::new(TcpClient::new(PeerProto).connect(addr, handle).map(
            |service| {
                Client {
                    inner: Validate { inner: service },
                    a: true,
                }
            },
        ))
    }

    pub fn run(&mut self) {
        self.a = false;
    }

    pub fn handshake(&mut self, hash_info: Vec<u8>, id: &[u8]) -> ClientResult {
        Box::new(
            self.call(Message::Handshake(hash_info.clone(), Vec::from(id)))
                .and_then(move |response| match response {
                    Message::Handshake(hash, _) => if hash == hash_info {
                        Ok(())
                    } else {
                        make_error(format!("expected {:?}", hash_info.to_hex()))
                    },
                    _ => make_error("Unexpected response"),
                }),
        )
    }

    pub fn unchoke(&mut self) -> ClientResult {
        Box::new(self.call(Message::Interested()).and_then(
            move |response| match response {
                Message::Unchoke() => {
                    Ok(())
                },
                _ => Ok(()),
            },
        ))
    }

    pub fn request(&mut self, index: u32, offset: u32, size: u32) -> ClientResult {
        Box::new(self.call(Message::Request(index, offset, size)).and_then(
            move |response| match response {
                Message::Piece(_index, _offset, _data) => Ok(()),
                _ => Ok(()),
            },
        ))
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
