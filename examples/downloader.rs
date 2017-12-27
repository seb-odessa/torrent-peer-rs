#[macro_use]
extern crate log;
extern crate env_logger;
extern crate futures;
extern crate torrent_peer;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_service;
extern crate rustc_serialize;

use std::io;
use std::env;
use std::net::SocketAddr;
use std::collections::HashSet;
use std::collections::HashMap;

use tokio_core::reactor::Core;
use rustc_serialize::hex::FromHex;
use rustc_serialize::hex::ToHex;

use torrent_peer::hash::sha1;
use torrent_peer::Client;

const BLOCK_LEN: u32 = 16384;

pub struct Downloader {
    address: SocketAddr,
    info_hash: Vec<u8>,
    piece_len: usize,
    piece_count: usize,
    requests: HashSet<(u32, u32, u32)>,
    blocks: HashMap<(u32, u32), Vec<u8>>,
}
impl Downloader {
    pub fn new(
        ip: String,
        port: u16,
        hash: Vec<u8>,
        total: usize,
        piece: usize,
    ) -> Result<Self, io::Error> {
        let addr = format!("{}:{}", ip, port).parse().map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("{}", e))
        })?;
        Ok(Self {
            address: addr,
            info_hash: hash,
            piece_len: piece,
            piece_count: (total / piece + !!(total % piece)),
            requests: HashSet::new(),
            blocks: HashMap::new(),
        })
    }

    /// returns true if request queue is empty or false otherwise
    pub fn is_done(&self) -> bool {
        self.requests.is_empty()
    }

    /// enqueue piece index to downloader
    pub fn enqueue_index(&mut self, index: u32) {
        info!("Downloader.enqueue_index({})", index);
        if self.piece_count < index as usize {
            return;
        }
        let mut offset = 0;
        let mut piece = self.piece_len as u32;
        while piece > BLOCK_LEN {
            info!(
                "Downloader.requests.insert({}, {}, {})",
                index,
                offset,
                BLOCK_LEN
            );
            self.requests.insert((index, offset, BLOCK_LEN));
            piece -= BLOCK_LEN;
            offset += BLOCK_LEN;
        }
        if piece > 0 {
            info!(
                "Downloader.requests.insert({}, {}, {})",
                index,
                offset,
                piece
            );
            self.requests.insert((index, offset, piece));
        }
    }

    /// get vector of received indices
    pub fn get_indices(&self) -> Vec<u32> {
        self.blocks
            .iter()
            .map(|(&(idx, _), _)| idx)
            .collect::<Vec<u32>>()
    }

    /// load pieces from client into own storage
    fn load(&mut self, blocks: &HashMap<(u32, u32), Vec<u8>>) {
        for (&(index, offset), block) in blocks.iter() {
            self.blocks.insert((index, offset), block.clone());
        }
    }

    /// pop request from queue and return it to the caller
    fn get_request(&mut self) -> Option<(u32, u32, u32)> {
        if let Some(&request) = self.requests.iter().next() {
            self.requests.remove(&request);
            return Some(request);
        }
        None
    }

    /// returns vector of u8 with content of the piece by index
    pub fn get_piece(&mut self, index: u32) -> Option<Vec<u8>> {
        let mut piece = Vec::new();
        let mut offset = 0;
        while let Some(block) = self.blocks.get_mut(&(index, offset)) {
            piece.append(block);
            offset += BLOCK_LEN;
        }
        if offset as usize == self.piece_len {
            Some(piece)
        } else {
            None
        }
    }

    /// invoke downloader to get all queued indexes
    fn invoke(&mut self, id: &str, mut attempts: u8) -> Result<(), io::Error> {
        let mut core = Core::new().unwrap();
        let handle = core.handle();
        let info = self.info_hash.clone();

        let mut client = core.run(Client::connect(&self.address, &handle))?;
        client = core.run(client.handshake(info, id.as_bytes()))?;
        client = core.run(client.ping())?;
        while !self.is_done() {
            if 0 == attempts {
                use io::Error;
                use io::ErrorKind::Other;
                return Err(Error::new(Other, "Attempt limit exceeded"));
            }
            if client.peer_choked && client.peer_intrested {
                client = core.run(client.unchoke_peer())?;
            }

            if client.am_choked {
                client = core.run(client.unchoke_me())?;
                attempts -= 1;
            } else {
                attempts += 1;
                if let Some(request) = self.get_request() {
                    client = core.run(client.request(request.0, request.1, request.2))?;
                }
            }
        }
        self.load(&client.blocks);
        Ok(())
    }
}

fn main() {
    env_logger::init().unwrap();
    let mut args = env::args().collect::<Vec<_>>();
    if args.len() == 1 {
        println!(
            "Usage:\n\t{} {} {} {} {} {}...",
            args[0],
            "192.168.0.100:6881",
            "5E433EDAE53E68AF02BC2650E057D0FC4FE41FCD",
            "924600668",
            "524288",
            "1"
        );
    } else {
        args.reverse();
        args.pop();
        let address = args.pop().unwrap();
        let target: Vec<_> = address.split(':').collect();
        let host = target[0].to_string();
        let port = target[1].parse::<u16>().unwrap();
        let hash = args.pop().unwrap().as_str().from_hex().unwrap();
        let total_len = args.pop().unwrap().parse::<usize>().unwrap();
        let piece_len = args.pop().unwrap().parse::<usize>().unwrap();
        let mut dl = Downloader::new(host, port, hash, total_len, piece_len).unwrap();

        while let Some(index) = args.pop() {
            if let Some(index) = index.parse::<u32>().ok() {
                dl.enqueue_index(index);
            }
        }

        match dl.invoke("-01-TORRENT-PEER-RS-", 2) {
            Ok(_) => {}
            Err(e) => println!("{}", e),
        }

        for index in dl.get_indices() {
            if let Some(piece) = dl.get_piece(index) {
                println!("{}", sha1(&piece).to_hex());
            }
        }
    }
}
