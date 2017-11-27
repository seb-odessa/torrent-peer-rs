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
    fn call(&self, message: Self::Request) -> Self::Future {
        println!("Request: {}", &message);
        let response;
        if message == "[ping]".to_owned() {
            response = "[pong]".to_owned()
        } else {
            response = format!("-={}=-", message)
        }
        println!("Response: {}", &response);
        Box::new(future::ok(response))
    }
}
