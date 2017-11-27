use tokio_service::Service;
use futures::{future, Future};
use {PeerRequest, PeerResponse, PeerError};

pub struct Echo;
impl Service for Echo {
    // These types must match the corresponding protocol types:
    type Request = PeerRequest;
    type Response = PeerResponse;
    // For non-streaming protocols, service errors are always io::Error
    type Error = PeerError;
    // The future for computing the response; box it for simplicity.
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    // Produce a future for computing a response from a request.
    fn call(&self, message: Self::Request) -> Self::Future {
        println!("Server: Request: {}", &message);
        let response;
        if message == "[handshake]".to_owned() {
            response = "[accept]".to_owned()
        } else {
            response = format!("-={}=-", message)
        }
        println!("Server: Response: {}", &response);
        Box::new(future::ok(response))
    }
}
