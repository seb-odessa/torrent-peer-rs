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
use lib::PeerProto;
use lib::Client;

use tokio_service::Service;
use futures::Future;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let address = "127.0.0.1:12345".parse().unwrap();
    if args.len() == 1 {
        let server = TcpServer::new(PeerProto, address);
        server.serve(|| Ok(Echo));
    } else if args[1] == "raw" {
        let mut core = Core::new().unwrap();
        let handle = core.handle();

        let connect = TcpClient::new(PeerProto).connect(&address, &handle);
        let send = connect.and_then(|connection| {
            connection
                .call(String::from("hello"))
                .and_then(|response| {
                    println!("Received from Server: {:?}", response);
                    Ok(())
                })
                .and_then(move |_| {
                    connection
                        .call(String::from("world"))
                        .and_then(|response| {
                            println!("Received from Server: {:?}", response);
                            Ok(())
                        })
                        .and_then(move |_| {
                            connection.call(String::from("Viva la Victoria")).and_then(
                                |response| {
                                    println!("Received from Server: {:?}", response);
                                    Ok(())
                                },
                            )
                        })
                })
        });
        core.run(send).unwrap();
    } else if args[1] == "client" {
        let mut core = Core::new().unwrap();
        let handle = core.handle();

        let client = Client::connect(&address, &handle).and_then(|client| {
            // Start with a ping
            client
                .ping()
                .and_then(move |_| {
                    println!("Pong received...");
                    client.call("Goodbye".to_string())
                })
                .and_then(|response| {
                    println!("CLIENT: {:?}", response);
                    Ok(())
                })
        });
        core.run(client).unwrap();
    } else {
        println!("Nothing to do, unknown argument");
    }
}
