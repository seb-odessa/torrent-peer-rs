extern crate futures;
extern crate lib;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_service;
extern crate rustc_serialize;

use std::env;
use futures::Future;
use tokio_proto::TcpServer;
use tokio_core::reactor::Core;
use rustc_serialize::hex::FromHex;

use lib::Echo;
use lib::PeerProto;
use lib::Client;

fn main() {
    let mut args = env::args().collect::<Vec<_>>();
    if args.len() == 1 {
        let address = "127.0.0.1:12345".parse().unwrap();
        println!("Server in debug mode started on {}", &address);
        let server = TcpServer::new(PeerProto, address);
        server.serve(|| Ok(Echo));
    } else {
        let port = args.pop().unwrap_or(String::from("6881"));
        let host = args.pop().unwrap_or(String::from("127.0.0.1"));
        let hash = args.pop().unwrap_or(String::from(
            "5E433EDAE53E68AF02BC2650E057D0FC4FE41FCD",
        ));
        let uri = format!("{}:{}", host, port);
        println!("{} get {}", uri, hash);
        let address = match uri.parse() {
            Ok(addr) => addr,
            Err(e) => {
                println!("{}", e);
                return;
            }
        };

        let mut core = Core::new().unwrap();
        let handle = core.handle();
        let hash_info: Vec<u8> = hash.as_str().from_hex().unwrap();
        let peer_id = "-01-TORRENT-PEER-RS-";
        let client = Client::connect(&address, &handle).and_then(|mut client| {
            client.handshake(hash_info, peer_id.as_bytes()).and_then(
                move |_| {
                    client.unchoke().and_then(move |_| client.request(0, 0, 64))
                },
            )
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
    }
}
