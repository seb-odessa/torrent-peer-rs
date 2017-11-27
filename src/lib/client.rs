use Validate;
use {PeerProto, PeerRequest, PeerResponse, PeerError};

use std::io;
use std::net::SocketAddr;

use futures::Future;
use tokio_core::reactor::Handle;
use tokio_proto::pipeline::ClientService;
use tokio_core::net::TcpStream;
use tokio_proto::TcpClient;
use tokio_service::Service;

pub type ClientConnection = Future<Item = Client, Error = PeerError>;
pub type ClientResult = Future<Item = (), Error = PeerError>;


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

    pub fn handshake(&self) -> Box<ClientResult> {
        let resp = self.call("[handshake]".to_string()).and_then(
            |resp| if resp !=
                "[accept]"
            {
                Err(io::Error::new(io::ErrorKind::Other, "expected [accept]"))
            } else {
                Ok(())
            },
        );
        Box::new(resp)
    }

    fn exchange(&self, msg: &str) -> Box<ClientResult> {
        Box::new(self.call(msg.to_string()).and_then(|_| Ok(())))
    }

    pub fn greeting(&self) -> Box<ClientResult> {
        self.exchange("[Hello]")
    }

    pub fn question(&self) -> Box<ClientResult> {
        self.exchange("[How are You]")
    }

    pub fn story(&self) -> Box<ClientResult> {
        self.exchange("[It is a time for a wonderful stories]")
    }

    pub fn bye(&self) -> Box<ClientResult> {
        self.exchange("[Goodbye]")
    }

    pub fn execute(&self) -> Result<(), PeerError> {
        self.handshake()
            .and_then(move |_| {
                self.greeting().and_then(move |_| {
                    self.question().and_then(move |_| {
                        self.story().and_then(move |_| {
                            self.bye();
                            Ok(())
                        })
                    })
                })
            })
            .wait()
    }
}

impl Service for Client {
    type Request = PeerRequest;
    type Response = PeerResponse;
    type Error = PeerError;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, request: Self::Request) -> Self::Future {
        println!("Request to Server: {:?}", request);
        Box::new(self.inner.call(request).and_then(|response| {
            println!("Response from Server: {:?}", response);
            Ok(response)
        }))

    }
}