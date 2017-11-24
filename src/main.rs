extern crate tokio_proto;
extern crate lib;

use std::env;
use tokio_proto::{TcpServer, TcpClient};

use lib::Echo;
use lib::ServiceProto;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() == 1 {
        let addr = "0.0.0.0:12345".parse().unwrap();
        let server = TcpServer::new(ServiceProto, addr);
        server.serve(|| Ok(Echo));
    } else {
        //let client = TcpClient::new(Proto);
    }


}
