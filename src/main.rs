extern crate futures;
extern crate lib;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_service;

use std::env;
use tokio_proto::{TcpClient, TcpServer};
use tokio_core::reactor::Core;

use lib::Echo;
use lib::ServiceProto;


use std::io;
use tokio_service::Service;
use futures::{future, Future};

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() == 1 {
        let addr = "0.0.0.0:12345".parse().unwrap();
        let server = TcpServer::new(ServiceProto, addr);
        server.serve(|| Ok(Echo));
    } else {
        let mut core = Core::new().unwrap();
        let handle = core.handle();
        let address = "127.0.0.1:12345".parse().unwrap();
        let connect = TcpClient::new(ServiceProto).connect(&address, &handle);
        let send = connect.and_then(|connection| connection.call(String::from("hello")));
        core.run(send).unwrap();
    }
}
