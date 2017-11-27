extern crate bytes;
extern crate futures;
extern crate tokio_io;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;

mod codec;
mod proto;
mod client;
mod validate;
mod echo_server;

pub use codec::Codec;
pub use client::Client;
pub use proto::PeerProto;
pub use echo_server::Echo;
pub use validate::Validate;


use std::io;

pub type PeerRequest = String;
pub type PeerResponse = String;
pub type PeerError = io::Error;
