#![feature(const_size_of)]
//#![feature(rustc_private)]

extern crate bytes;
extern crate futures;
extern crate tokio_io;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;
extern crate rustc_serialize;
extern crate byteorder;

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

use std::fmt;
use rustc_serialize::hex::ToHex;

#[derive(PartialEq, Debug, Clone)]
pub enum Message {
    Handshake(Vec<u8>, Vec<u8>),
    KeepAlive(),
    Choke(),
    Unchoke(),
    Interested(),
    NotInterested(),
    Have(u32),
    Bitfield(Vec<u8>),
    Request(u32, u32, u32),
    Piece(u32, u32, Vec<u8>),
    Cancel(u32, u32, u32),
    Port(u16),
}

impl fmt::Display for Message {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            &Message::Handshake(ref info, ref id) => {
                write!(
                    fmt,
                    "Handshake([{}][{}])",
                    info.to_hex(),
                    String::from_utf8_lossy(&id)
                )?;
            }
            &Message::KeepAlive() => {
                write!(fmt, "KeepAlive()")?;
            }
            &Message::Choke() => {
                write!(fmt, "Choke()")?;
            }
            &Message::Unchoke() => {
                write!(fmt, "Unchoke()")?;
            }
            &Message::Interested() => {
                write!(fmt, "Interested()")?;
            }
            &Message::NotInterested() => {
                write!(fmt, "NotInterested()")?;
            }
            &Message::Have(ref index) => {
                write!(fmt, "Have({})", index)?;
            }
            &Message::Bitfield(ref bits) => {
                write!(fmt, "Bitfield([u8; {}])", bits.len())?;
            }
            &Message::Request(ref index, ref offset, ref length) => {
                write!(fmt, "Request({}, {}, {})", index, offset, length)?;
            }
            &Message::Piece(ref index, ref offset, ref data) => {
                write!(fmt, "Piece({}, {}, [u8; {}])", index, offset, data.len())?;
            }
            &Message::Cancel(ref index, ref offset, ref length) => {
                write!(fmt, "Cancel({}, {}, {})", index, offset, length)?;
            }
            &Message::Port(ref port) => {
                write!(fmt, "Port({})", port)?;
            }
        };
        write!(fmt, "")
    }
}
