use Validate;
use PeerProto;

use std::io;
use std::net::SocketAddr;

use futures::Future;
use tokio_core::reactor::Handle;
use tokio_proto::pipeline::ClientService;
use tokio_core::net::TcpStream;
use tokio_proto::TcpClient;
use tokio_service::Service;

pub struct Client {
    inner: Validate<ClientService<TcpStream, PeerProto>>,
}

impl Client {
    /// Establish a connection to a line-based server at the provided `addr`.
    pub fn connect(
        addr: &SocketAddr,
        handle: &Handle,
    ) -> Box<Future<Item = Client, Error = io::Error>> {
        let client = TcpClient::new(PeerProto).connect(addr, handle).map(
            |client_service| {
                let validate = Validate { inner: client_service };
                Client { inner: validate }
            },
        );
        Box::new(client)
    }

    /// Send a `ping` to the remote. The returned future resolves when the
    /// remote has responded with a pong.
    ///
    /// This function provides a bit of sugar on top of the the `Service` trait.
    pub fn ping(&self) -> Box<Future<Item = (), Error = io::Error>> {
        // The `call` response future includes the string, but since this is a
        // "ping" request, we don't really need to include the "pong" response
        // string.
        let resp = self.call("[ping]".to_string()).and_then(
            |resp| if resp != "[pong]" {
                Err(io::Error::new(io::ErrorKind::Other, "expected pong"))
            } else {
                Ok(())
            },
        );
        // Box the response future because we are lazy and don't want to define
        // a new future type and `impl T` isn't stable yet...
        Box::new(resp)
    }
}

impl Service for Client {
    type Request = String;
    type Response = String;
    type Error = io::Error;
    // For simplicity, box the future.
    type Future = Box<Future<Item = String, Error = io::Error>>;

    fn call(&self, req: String) -> Self::Future {
        self.inner.call(req)
    }
}