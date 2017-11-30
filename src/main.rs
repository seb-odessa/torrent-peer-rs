extern crate futures;
extern crate lib;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_service;
extern crate rustc_serialize;

use std::env;
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
        let info: Vec<u8> = hash.as_str().from_hex().unwrap();
        let id = "-01-TORRENT-PEER-RS-".as_bytes();

        let mut client = core.run(Client::connect(&address, &handle)).unwrap();
        client = core.run(client.handshake(info, id)).unwrap();

        client = core.run(client.ping()).unwrap();

        client = core.run(client.unchoke_me()).unwrap();
        if client.peer_choked && client.peer_intrested {
            client = core.run(client.unchoke_peer()).unwrap();
        }

        let mut attempts = 5;
        while attempts > 0 && client.am_choked {
            client = core.run(client.unchoke_me()).unwrap();
            attempts -= 1;
        }

        if !client.am_choked {
            client = core.run(client.request(2, 4, 16384)).unwrap();
        }
        attempts = 5;
        while attempts > 0 && !client.done {
            client = core.run(client.ping()).unwrap();
            attempts -= 1;
        }
        if !client.am_choked && !client.done {
            core.run(client.request(3, 4, 16384)).unwrap();
        }

    }
}
