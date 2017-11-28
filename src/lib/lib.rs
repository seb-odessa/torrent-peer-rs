extern crate bytes;
extern crate futures;
extern crate tokio_io;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;
extern crate rustc_serialize;

mod codec;
mod proto;
mod client;
mod validate;
mod echo_server;

pub use codec::PeerCodec;
pub use proto::PeerProto;

pub use validate::Validate;
pub use client::Client;
pub use echo_server::Echo;


#[derive(PartialEq, Debug, Clone)]
pub enum Message {
    Handshake(Vec<u8>, Vec<u8>),
    KeepAlive(u8),
    Choke(u8),
    Unchoke(u8),
    Interested(u8),
    NotInterested(u8),
    Have(u32),
    Bitfield(Vec<u8>),
    Request(u32, u32, u32),
    Piece(u32, u32, Vec<u8>),
    Cancel(u32, u32, u32),
    Port(u16),
    Error,
}

use std::io;
fn make_error<T: Into<String>>(msg: T) -> Result<(), io::Error> {
    return Err(io::Error::new(io::ErrorKind::Other, msg.into()));
}
