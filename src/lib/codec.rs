use std::io;
use std::str;
use std::mem::size_of;
use bytes::BytesMut;
use tokio_io::codec::{Encoder, Decoder};
use rustc_serialize::hex::ToHex;
use byteorder::{ByteOrder, BigEndian};

use Message;
use make_error;

const PSTR: &'static str = "BitTorrent protocol";
const PSTR_SIZE: usize = 19;
const HASH_INFO_LEN: usize = 20;
const PEER_ID_LEN: usize = 20;
const RESERVED_LEN: usize = 8;
const RESERVED: [u8; RESERVED_LEN] = [0, 0, 0, 0, 0, 0, 0, 0];
const KEEP_ALIVE: [u8; 4] = [0,0,0,0];
const BYTE_SIZE: usize =  size_of::<u8>();
const SHORT_SIZE: usize =  size_of::<u16>();
const NUMBER_SIZE: usize = size_of::<u32>();

const CHOCKE_ID: u8 = 0;
const UNCHOCKE_ID: u8 = 1;
const INTERESTED_ID: u8 = 2;
const NOT_INTERESTED_ID: u8 = 3;
const HAVE_ID: u8 = 4;
const BITFIELD_ID: u8 = 5;
const REQUEST_ID: u8 = 6;
const PIECE_ID: u8 = 7;
const CANCEL_ID: u8 = 8;
const PORT_ID: u8 = 9;


pub struct PeerCodec;
impl PeerCodec {
    fn handshake(&self, buf: &mut BytesMut) -> Option<Message> {
        //<PSTRLIN: u8><PSTR: 'BitTorrent protocol'><RESERVED[0u8; 8]><info_hash: [u8; 20]><peer_id: [u8; 20]>
        const HANDSHAKE_LENGTH: usize = BYTE_SIZE + PSTR_SIZE + RESERVED_LEN + HASH_INFO_LEN + PEER_ID_LEN;

        if HANDSHAKE_LENGTH == buf.len() && buf[0] as usize == PSTR_SIZE &&
            &buf[1..(PSTR_SIZE + BYTE_SIZE) as usize] == PSTR.as_bytes()
        {
            let mut hash_info = Vec::with_capacity(HASH_INFO_LEN);
            let mut peer_id = Vec::with_capacity(PEER_ID_LEN);
            buf.split_to(BYTE_SIZE); // consume PSTR_SIZE
            buf.split_to(PSTR_SIZE); // consume PSTR
            buf.split_to(RESERVED_LEN); // consume RESERVED
            hash_info.extend_from_slice(buf.split_to(HASH_INFO_LEN).as_ref());
            peer_id.extend_from_slice(buf.split_to(PEER_ID_LEN).as_ref());
            Some(Message::Handshake(hash_info, peer_id))
        } else {
            None
        }
    }

    fn have(&self, buf: &mut BytesMut) -> Option<Message> {
        // have: <len=0005><id=4><piece index>
        if buf.len() >= NUMBER_SIZE {
            let index = BigEndian::read_u32(&buf.split_to(NUMBER_SIZE));
            Some(Message::Have(index))
        } else {
            None
        }
    }

    fn bitfield(&self, buf: &mut BytesMut, len: usize) -> Option<Message> {
        // bitfield: <len=0001+size_of bitfield><id=5><bitfield>
        if buf.len() >= len {
            let bitfield = Vec::from(buf.split_to(len - BYTE_SIZE).as_ref());
            Some(Message::Bitfield(bitfield))
        } else {
            None
        }
    }

    fn request(&self, buf: &mut BytesMut) -> Option<Message> {
        // request: <len=0013><id=6><index><begin><length>
        if buf.len() >= 3 * NUMBER_SIZE {
            let index = BigEndian::read_u32(&buf.split_to(NUMBER_SIZE));
            let begin = BigEndian::read_u32(&buf.split_to(NUMBER_SIZE));
            let length = BigEndian::read_u32(&buf.split_to(NUMBER_SIZE));
            Some(Message::Request(index, begin, length))
        } else {
            None
        }
    }

    fn cancel(&self, buf: &mut BytesMut) -> Option<Message> {
        // cancel: <len=0013><id=8><index><begin><length>
        if buf.len() >= 3 * NUMBER_SIZE {
            let index = BigEndian::read_u32(&buf.split_to(NUMBER_SIZE));
            let begin = BigEndian::read_u32(&buf.split_to(NUMBER_SIZE));
            let length = BigEndian::read_u32(&buf.split_to(NUMBER_SIZE));
            Some(Message::Cancel(index, begin, length))
        } else {
            None
        }
    }

    fn piece(&self, buf: &mut BytesMut, len: usize) -> Option<Message> {
        // piece: <len=0009+X><id=7><index><begin><block>
        if buf.len() >= len {
            let index = BigEndian::read_u32(&buf.split_to(NUMBER_SIZE));
            let begin = BigEndian::read_u32(&buf.split_to(NUMBER_SIZE));
            let block = Vec::from(buf.split_to(len - BYTE_SIZE - 2 * NUMBER_SIZE).as_ref());
            Some(Message::Piece(index, begin, block))
        } else {
            None
        }
    }

    fn port(&self, buf: &mut BytesMut) -> Option<Message> {
        // port: <len=0003><id=9><listen-port>
        if buf.len() >= SHORT_SIZE {
            let port = BigEndian::read_u16(&buf.split_to(SHORT_SIZE));
            Some(Message::Port(port))
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
        } else if let Some(handshake) = self.handshake(buf) {
            Ok(Some(handshake))
        } else if buf.len() >= NUMBER_SIZE {
            let msg_len = BigEndian::read_u32(&buf.split_to(NUMBER_SIZE)) as usize;
            println!("PeerCodec::decode() msg_len = {}", &msg_len);
            if 0 == msg_len {
                Ok(Some(Message::KeepAlive()))
            } else if buf.len() > 0 {
                let msg_id = buf.split_to(1)[0];
                println!("PeerCodec::decode() msg_id = {}", &msg_id);
                match msg_id {
                    CHOCKE_ID  => Ok(Some(Message::Choke())),
                    UNCHOCKE_ID => Ok(Some(Message::Unchoke())),
                    INTERESTED_ID  => Ok(Some(Message::Interested())),
                    NOT_INTERESTED_ID => Ok(Some(Message::NotInterested())),
                    HAVE_ID  => Ok(self.have(buf)),
                    BITFIELD_ID => Ok(self.bitfield(buf, msg_len)),
                    REQUEST_ID  => Ok(self.request(buf)),
                    PIECE_ID => Ok(self.piece(buf, msg_len)),
                    CANCEL_ID  => Ok(self.cancel(buf)),
                    PORT_ID => Ok(self.port(buf)),
                    _ => Ok(None)
                }
            } else {
                Ok(None)
            }
        } else {
            let length = buf.len();
            buf.split_to(length);
            Ok(None)
        }
    }
}
impl Encoder for PeerCodec {
    type Item = Message;
    type Error = io::Error;

    fn encode(&mut self, msg: Message, buf: &mut BytesMut) -> io::Result<()> {
        println!("PeerCodec::encode() <= {:?}", &msg);
        match msg {
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

                println!("Handshake([{}][{}])",hash_info.to_hex(), String::from_utf8_lossy(&peer_id));
            },
            Message::KeepAlive() => {
                buf.extend_from_slice(&KEEP_ALIVE);
            },
            Message::Choke() => {
                let mut length: [u8; 4] = [0,0,0,0];
                BigEndian::write_u32(&mut length, BYTE_SIZE as u32);
                buf.extend_from_slice(&length);
                buf.extend_from_slice(&[0x00; 1]);
                println!("Choke()");
            }
            _ => {
                //
            }
        }
        println!("PeerCodec::encode() => {:?}", &buf);
        Ok(())
    }
}
