use std::io;

use tokio_service::Service;
use futures::{future, Future};

use Message;

pub struct Echo;
impl Service for Echo {
    type Request = Message;
    type Response = Message;
    type Error = io::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, message: Self::Request) -> Self::Future {
        println!("Server: Request: {:?}", &message);
        let response = message;
        println!("Server: Response: {:?}", &response);
        Box::new(future::ok(response))
    }
}
