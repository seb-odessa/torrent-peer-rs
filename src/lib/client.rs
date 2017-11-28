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

pub type ClientConnection = Future<Item = Client, Error = io::Error>;
pub type ClientResult = Future<Item = (), Error = io::Error>;


pub struct Client {
    inner: Validate<ClientService<TcpStream, PeerProto>>,
}

impl Client {
    pub fn connect(addr: &SocketAddr, handle: &Handle) -> Box<ClientConnection> {
        let client = TcpClient::new(PeerProto).connect(addr, handle).map(
            |client_service| {
                let validate = Validate { inner: client_service };
                Client { inner: validate }
            },
        );
        Box::new(client)
    }

    pub fn handshake(&self, hash_info: Vec<u8>, id: &[u8]) -> Box<ClientResult> {
        let peer_id = Vec::from(id);
        Box::new(self.call(Message::Handshake(hash_info.clone(), peer_id))
            .and_then(move |response| match response {
                Message::Handshake(hash, _) => {
                    if hash == hash_info {
                        Ok(())
                    } else {
                        make_error(format!("expected {:?}", hash_info.to_hex()))
                    }
                }
                _ => make_error("Unexpected response"),
            }))
    }
}

impl Service for Client {
    type Request = Message;
    type Response = Message;
    type Error = io::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, request: Self::Request) -> Self::Future {
        //println!("Client: Request to Server: {:?}", request);
        Box::new(self.inner.call(request).and_then(|response| {
            // println!("Client: Response from Server: {:?}", response);
            Ok(response)
        }))

    }
}