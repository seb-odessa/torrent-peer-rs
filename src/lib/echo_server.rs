use std::io;
use tokio_service::Service;
use futures::{future, Future};

pub struct Echo;
impl Service for Echo {
    // These types must match the corresponding protocol types:
    type Request = String;
    type Response = String;
    // For non-streaming protocols, service errors are always io::Error
    type Error = io::Error;
    // The future for computing the response; box it for simplicity.
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    // Produce a future for computing a response from a request.
    fn call(&self, req: Self::Request) -> Self::Future {
        // processing request
        let mut copy = req.clone();
        copy.pop();
        let result = format!("Response is a '{}' string", copy);
        Box::new(future::ok(result))
    }
}