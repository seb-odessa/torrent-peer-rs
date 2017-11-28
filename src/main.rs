extern crate futures;
extern crate lib;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_service;

use std::env;
use futures::Future;
use tokio_proto::TcpServer;
use tokio_core::reactor::Core;

use lib::Echo;
use lib::PeerProto;
use lib::Client;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let address = "127.0.0.1:12345".parse().unwrap();
    if args.len() == 1 {
        let server = TcpServer::new(PeerProto, address);
        server.serve(|| Ok(Echo));
    } else if args[1] == "client" {
        let mut core = Core::new().unwrap();
        let handle = core.handle();
        let hash_info = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let peer_id = "-RT0001-0123456789A-";
        let client = Client::connect(&address, &handle).and_then(|client| {
            client.handshake(hash_info, peer_id.as_bytes())
        });

        // let client = Client::connect(&address, &handle).and_then(|client| {
        //     client.handshake()
        //         .and_then(move |_| {
        //         client.greeting().and_then(move |_| {
        //             client.question().and_then(move |_| {
        //                 client.story().and_then(move |_| client.bye())
        //             })
        //         })
        //     })
        // });
        core.run(client).unwrap();
    } else {
        println!("Nothing to do, unknown argument");
    }
}
