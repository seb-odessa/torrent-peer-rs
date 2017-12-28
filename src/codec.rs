use std::io;
use std::str;
use std::mem::size_of;
use rustc_serialize::hex::ToHex;

use std::thread;
use std::time::Duration;

use bytes::BytesMut;
use tokio_io::codec::{Encoder, Decoder};
use byteorder::{ByteOrder, BigEndian};

use Message;
use Messages;

const PSTR: &'static str = "BitTorrent protocol";
const PSTR_SIZE: usize = 19;
const HASH_INFO_LEN: usize = 20;
const PEER_ID_LEN: usize = 20;
const RESERVED_LEN: usize = 8;
const RESERVED: [u8; RESERVED_LEN] = [0, 0, 0, 0, 0, 0, 0, 0];

const BYTE_SIZE: usize = size_of::<u8>();
const SHORT_SIZE: usize = size_of::<u16>();
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
        //<PSTRLIN: u8><PSTR: 'BitTorrent protocol'>
        //  <RESERVED[0u8; 8]>
        //  <info_hash: [u8; 20]>
        //  <peer_id: [u8; 20]>
        const HANDSHAKE_LENGTH: usize = BYTE_SIZE + PSTR_SIZE + RESERVED_LEN + HASH_INFO_LEN +
            PEER_ID_LEN;

        if HANDSHAKE_LENGTH <= buf.len() && buf[0] as usize == PSTR_SIZE &&
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

    fn choke(&self) -> Option<Message> {
        // choke: <len=0001><id=0>";
        Some(Message::Choke())
    }

    fn unchoke(&self) -> Option<Message> {
        // unchoke: <len=0001><id=1>";
        Some(Message::Unchoke())
    }

    fn interested(&self) -> Option<Message> {
        // interested: <len=0001><id=2>";
        Some(Message::Unchoke())
    }

    fn not_interested(&self) -> Option<Message> {
        // not interested: <len=0001><id=3>";
        Some(Message::Unchoke())
    }

    fn have(&self, buf: &mut BytesMut) -> Option<Message> {
        // have: <len=0005><id=4><index>
        if buf.len() >= NUMBER_SIZE {
            let index = BigEndian::read_u32(&buf.split_to(NUMBER_SIZE));
            println!("Msg Have index {}", index);
            Some(Message::Have(index))
        } else {
            None
        }
    }

    fn bitfield(&self, buf: &mut BytesMut, len: usize) -> Option<Message> {
        // bitfield: <len=0001+size_of bitfield><id=5><bitfield>
        let data_len = len - BYTE_SIZE;
        if buf.len() >= data_len {
            let bits = Vec::from(buf.split_to(data_len).as_ref());
            Some(Message::Bitfield(bits))
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
        // piece: <len=0007+X><id=7><index><begin><block>
        if buf.len() >= len - BYTE_SIZE {
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
    type Item = Messages;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Messages>> {
        let mut messages = Messages::new();
        if buf.len() < 200 {
            println!("Decoder::decode() <= '{}'", &buf.to_hex());
        }
        if buf.is_empty() {
            return Ok(None);
        } else if let Some(handshake) = self.handshake(buf) {
            messages.push_back(Some(handshake));
        } else {
            while buf.len() >= size_of::<u32>() {
                // println!("Decoder::decode(): buf.len(): {}", buf.len());
                let payload_length = BigEndian::read_u32(&buf) as usize;
                // println!("Decoder::decode(): payload_length: {}", payload_length);
                let message_length = size_of::<u32>() + payload_length;
                // println!("Decoder::decode(): message_length: {}", message_length);
                if buf.len() < message_length {
                    //println!("Decoder::decode(): rest of buf: {}", buf.to_hex());
                    break;
                }
                // println!(
                //     "Decoder::decode(): msg: {} ",
                //     buf[0..message_length].to_hex()
                // );

                buf.split_to(size_of::<u32>()); // consume payload length

                //thread::sleep(Duration::from_millis(30));
                if 0 == payload_length {
                    messages.push_back(Some(Message::KeepAlive()));
                } else if buf.len() >= payload_length {
                    let msg_code = buf.split_to(1)[0]; // consume cmd code
                    // println!("Decoder::decode(): msg_code: {}", msg_code);
                    let msg = match msg_code {
                        CHOCKE_ID => self.choke(),
                        UNCHOCKE_ID => self.unchoke(),
                        INTERESTED_ID => self.interested(),
                        NOT_INTERESTED_ID => self.not_interested(),
                        HAVE_ID => self.have(buf),
                        BITFIELD_ID => self.bitfield(buf, payload_length),
                        REQUEST_ID => self.request(buf),
                        PIECE_ID => self.piece(buf, payload_length),
                        CANCEL_ID => self.cancel(buf),
                        PORT_ID => self.port(buf),
                        _ => {
                            println!("Decoder::decode(): Unknown Message: {:X}", msg_code);
                            None
                        }
                    };
                    messages.push_back(msg);
                }
            }
        }
        // println!("Decoder::decode() messages.len(): {}", messages.len());
        // for msg in &messages {
        //     println!("Decoder::decode() msg: {:?}", msg);
        // }
        if messages.is_empty() {
            Ok(None)
        } else {
            Ok(Some(messages))
        }
    }
}
impl Encoder for PeerCodec {
    type Item = Message;
    type Error = io::Error;

    fn encode(&mut self, msg: Message, buf: &mut BytesMut) -> io::Result<()> {
        match msg {
            Message::Handshake(hash_info, peer_id) => {
                if hash_info.len() != HASH_INFO_LEN {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "HASH INFO length shall be 20 bytes",
                    ));
                }
                if peer_id.len() != PEER_ID_LEN {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "PEER ID length shall be 20 bytes",
                    ));
                }
                add_u8(buf, PSTR.len() as u8);
                add_vec(buf, PSTR.as_bytes());
                add_vec(buf, &RESERVED);
                add_vec(buf, &hash_info);
                add_vec(buf, &peer_id);
            }
            Message::KeepAlive() => {
                // keep-alive: <len=0000>
                add_len(buf, 0x00);
            }
            Message::Choke() => {
                // choke: <len=0001><id=0>
                add_len(buf, 0x01);
                add_u8(buf, 0x00);
            }
            Message::Unchoke() => {
                // unchoke: <len=0001><id=1>
                add_len(buf, 0x01);
                add_u8(buf, 0x01);
            }
            Message::Interested() => {
                // interested: <len=0001><id=2>
                add_len(buf, 0x01);
                add_u8(buf, 0x02);
            }
            Message::NotInterested() => {
                // not interested: <len=0001><id=3>
                add_len(buf, 0x01);
                add_u8(buf, 0x03);
            }
            Message::Have(index) => {
                // have: <len=0005><id=4><piece index>
                add_len(buf, 0x05);
                add_u8(buf, 0x04);
                add_u32(buf, index);
            }
            Message::Bitfield(bitfield) => {
                // bitfield: <len=0001+X><id=5><bitfield>
                add_len(buf, 0x01 + bitfield.len() as u32);
                add_u8(buf, 0x05);
                add_vec(buf, &bitfield);
            }
            Message::Request(index, begin, length) => {
                // request: <len=0013><id=6><index><begin><length>
                add_len(buf, 0x0D);
                add_u8(buf, 0x06);
                add_u32(buf, index);
                add_u32(buf, begin);
                add_u32(buf, length);
            }
            Message::Piece(index, begin, block) => {
                // piece: <len=0007+X><id=7><index><begin><block>
                add_len(buf, 0x07 + block.len() as u32);
                add_u8(buf, 0x07);
                add_u32(buf, index);
                add_u32(buf, begin);
                add_vec(buf, &block);
            }
            Message::Cancel(index, begin, length) => {
                // cancel: <len=0013><id=8><index><begin><length>
                add_len(buf, 0x0D);
                add_u8(buf, 0x08);
                add_u32(buf, index);
                add_u32(buf, begin);
                add_u32(buf, length);
            }
            Message::Port(port) => {
                // port: <len=0003><id=9><listen-port>
                add_len(buf, 0x03);
                add_u8(buf, 0x09);
                add_u16(buf, port);
            }
        }
        // println!("Encoder::encode() => '{}'", &buf.to_hex());
        Ok(())
    }
}

fn add_u8(buf: &mut BytesMut, id: u8) {
    let container = [id; size_of::<u8>()];
    buf.extend_from_slice(&container);
}

fn add_len(buf: &mut BytesMut, value: u32) {
    add_u32(buf, value)
}

fn add_u32(buf: &mut BytesMut, value: u32) {
    let mut container = [0u8; size_of::<u32>()];
    BigEndian::write_u32(&mut container, value);
    buf.extend_from_slice(&container);
}

fn add_u16(buf: &mut BytesMut, value: u16) {
    let mut container = [0u8; size_of::<u16>()];
    BigEndian::write_u16(&mut container, value);
    buf.extend_from_slice(&container);
}

fn add_vec(buf: &mut BytesMut, container: &[u8]) {
    buf.extend_from_slice(container);
}
