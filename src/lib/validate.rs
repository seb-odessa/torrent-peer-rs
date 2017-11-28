use std::io;
use tokio_service::{Service, NewService};
use futures::{future, Future};
use Message;

const ERROR_MESSAGE: &'static str = "Was found malformed message";

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
        println!("Validator: Request:  {:?}", &req);

        if req == Message::Error {
            let err = io::Error::new(io::ErrorKind::InvalidInput, ERROR_MESSAGE);
            return Box::new(future::done(Err(err)));
        }
        Box::new(self.inner.call(req).and_then(|resp| {
            println!("Validator: Response: {:?}", &resp);
            if resp == Message::Error {
                Err(io::Error::new(io::ErrorKind::InvalidInput, ERROR_MESSAGE))
            } else {
                Ok(resp)
            }
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