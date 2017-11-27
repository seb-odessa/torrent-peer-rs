use std::io;
use tokio_service::{Service, NewService};
use futures::{future, Future};



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
    T: Service<Request = String, Response = String, Error = io::Error>,
    T::Future: 'static,
{
    type Request = String;
    type Response = String;
    type Error = io::Error;
    // For simplicity, box the future.
    type Future = Box<Future<Item = String, Error = io::Error>>;

    fn call(&self, req: String) -> Self::Future {
        // Make sure that the request does not include any new lines
        if req.chars().find(|&c| c == '\n').is_some() {
            let err = io::Error::new(io::ErrorKind::InvalidInput, "message contained new line");
            return Box::new(future::done(Err(err)));
        }

        // Call the upstream service and validate the response
        Box::new(self.inner.call(req).and_then(|resp| {
            if resp.chars().find(|&c| c == '\n').is_some() {
                Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "message contained new line",
                ))
            } else {
                Ok(resp)
            }
        }))
    }
}

impl<T> NewService for Validate<T>
    where T: NewService<Request = String, Response = String, Error = io::Error>,
          <T::Instance as Service>::Future: 'static
{
    type Request = String;
    type Response = String;
    type Error = io::Error;
    type Instance = Validate<T::Instance>;

    fn new_service(&self) -> io::Result<Self::Instance> {
        let inner = try!(self.inner.new_service());
        Ok(Validate { inner: inner })
    }
}