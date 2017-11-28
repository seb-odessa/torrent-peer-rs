use std::io;
use std::str;
use bytes::BytesMut;
use tokio_io::codec::{Encoder, Decoder};
use rustc_serialize::hex::ToHex;


use Message;
use make_error;

const PSTR: &'static str = "BitTorrent protocol";
const PSTRLEN: usize = 19;
const HASH_INFO_LEN: usize = 20;
const PEER_ID_LEN: usize = 20;
const RESERVED_LEN: usize = 8;
const RESERVED: [u8; RESERVED_LEN] = [0, 0, 0, 0, 0, 0, 0, 0];

pub struct PeerCodec;
impl PeerCodec {
    fn is_handshake(&self, buf: &mut BytesMut) -> Option<Message> {
        //<PSTRLEN: u8><PSTR: 'BitTorrent protocol'><RESERVED[0u8; 8]><info_hash: [u8; 20]><peer_id: [u8; 20]>
        const HANDSHAKE_LENGTH: usize = 1 + PSTRLEN + RESERVED_LEN + HASH_INFO_LEN + PEER_ID_LEN;

        if HANDSHAKE_LENGTH == buf.len() && buf[0] as usize == PSTRLEN &&
            &buf[1..(PSTRLEN + 1) as usize] == PSTR.as_bytes()
        {
            let mut hash_info = Vec::with_capacity(HASH_INFO_LEN);
            let mut peer_id = Vec::with_capacity(PEER_ID_LEN);
            buf.split_to(1); // consume PSTRLEN
            buf.split_to(PSTRLEN); // consume PSTR
            buf.split_to(RESERVED_LEN); // consume RESERVED
            hash_info.extend_from_slice(buf.split_to(HASH_INFO_LEN).as_ref());
            peer_id.extend_from_slice(buf.split_to(PEER_ID_LEN).as_ref());
            Some(Message::Handshake(hash_info, peer_id))
        } else {
            None
        }
    }
}
impl Decoder for PeerCodec {
    type Item = Message;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Message>> {
        println!("PeerCodec::decode() <= {:?}", &buf);
        if buf.is_empty() {
            Ok(None)
        } else if let Some(handshake) = self.is_handshake(buf) {
            Ok(Some(handshake))
        } else {
            let len = buf.len();
            buf.split_to(len);
            Ok(None)
        }
    }
}
impl Encoder for PeerCodec {
    type Item = Message;
    type Error = io::Error;

    fn encode(&mut self, msg: Message, buf: &mut BytesMut) -> io::Result<()> {
        match msg.clone() {
            Message::Handshake(hash_info, peer_id) => {
                if hash_info.len() != HASH_INFO_LEN {
                    return make_error("HASH INFO length shall be 20 bytes");
                }
                if peer_id.len() != PEER_ID_LEN {
                    return make_error("PEER ID length shall be 20 bytes");
                }
                buf.extend_from_slice(&[19u8; 1]);
                buf.extend(PSTR.as_bytes());
                buf.extend_from_slice(&RESERVED);
                buf.extend_from_slice(&hash_info);
                buf.extend_from_slice(&peer_id);

                println!(
                    "PeerCodec::encode(Handshake([{}][{}])) -> {:?}",
                    hash_info.to_hex(),
                    String::from_utf8_lossy(&peer_id),
                    &buf
                );
            }
            _ => {
                println!("PeerCodec::encode({:?}) -> {:?}", &msg, &buf);
            }
        }

        Ok(())
    }
}
