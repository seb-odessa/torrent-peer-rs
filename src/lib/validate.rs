use std::io;
use tokio_service::{Service, NewService};
use futures::Future;
//use futures::future;
use Message;

//const ERROR_MESSAGE: &'static str = "Was found malformed message";

pub struct Validate<T> {
    pub inner: T,
}

impl<T> Validate<T> {
    pub fn new(inner: T) -> Validate<T> {
        Validate { inner: inner }
    }
}

impl<T> Service for Validate<T>
where
    T: Service<Request = Message, Response = Message, Error = io::Error>,
    T::Future: 'static,
{
    type Request = Message;
    type Response = Message;
    type Error = io::Error;
    type Future = Box<Future<Item = Message, Error = io::Error>>;


    fn call(&self, req: Message) -> Self::Future {
        println!("Request: {}", &req);
        Box::new(self.inner.call(req).and_then(|resp| {
            println!("Response: {}", &resp);
            Ok(resp)
        }))
    }
}

impl<T> NewService for Validate<T>
where
    T: NewService<
        Request = Message,
        Response = Message,
        Error = io::Error,
    >,
    <T::Instance as Service>::Future: 'static,
{
    type Request = Message;
    type Response = Message;
    type Error = io::Error;
    type Instance = Validate<T::Instance>;

    fn new_service(&self) -> io::Result<Self::Instance> {
        let inner = try!(self.inner.new_service());
        Ok(Validate { inner: inner })
    }
}