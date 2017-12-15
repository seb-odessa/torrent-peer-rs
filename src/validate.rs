use std::io;
use tokio_service::{Service, NewService};
use futures::Future;
//use futures::future;
use Message;
use Messages;

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
    T: Service<Request = Message, Response = Messages, Error = io::Error>,
    T::Future: 'static,
{
    type Request = Message;
    type Response = Messages;
    type Error = io::Error;
    type Future = Box<Future<Item = Messages, Error = io::Error>>;


    fn call(&self, req: Message) -> Self::Future {
        Box::new(self.inner.call(req))
    }
}

impl<T> NewService for Validate<T>
where
    T: NewService<
        Request = Message,
        Response = Messages,
        Error = io::Error,
    >,
    <T::Instance as Service>::Future: 'static,
{
    type Request = Message;
    type Response = Messages;
    type Error = io::Error;
    type Instance = Validate<T::Instance>;

    fn new_service(&self) -> io::Result<Self::Instance> {
        let inner = try!(self.inner.new_service());
        Ok(Validate { inner: inner })
    }
}
