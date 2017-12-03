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

const CHUNK_LEN: u32 = 16384;
const TRIES_TO_UNCHOKE: u8 = 5;

struct DataDesc {
    pub address: SocketAddr,
    pub info_hash: Vec<u8>,
    piece_len: usize,
    piece_count: usize,
    requests: HashSet<(u32, u32, u32)>,
    blocks: HashMap<(u32, u32), Vec<u8>>,
}
impl DataDesc {
    pub fn new(addr: SocketAddr, hash: Vec<u8>, total: usize, piece: usize) -> Self {
        Self {
            address: addr,
            info_hash: hash,
            piece_len: piece,
            piece_count: (total / piece + !!(total % piece)),
            requests: HashSet::new(),
            blocks: HashMap::new(),
        }
    }

    pub fn add_index(&mut self, index: u32) {
        info!("DataDesc.add_index({})", index);
        if self.piece_count < index as usize {
            return;
        }
        let mut offset = 0;
        let mut piece = self.piece_len as u32;
        while piece > CHUNK_LEN {
            info!(
                "DataDesc.requests.insert({}, {}, {})",
                index,
                offset,
                CHUNK_LEN
            );
            self.requests.insert((index, offset, CHUNK_LEN));
            piece -= CHUNK_LEN;
            offset += CHUNK_LEN;
        }
        if piece > 0 {
            info!(
                "DataDesc::.requests.insert({}, {}, {})",
                index,
                offset,
                piece
            );
            self.requests.insert((index, offset, piece));
        }
    }

    pub fn load_blocks(&mut self, blocks: &HashMap<(u32, u32), Vec<u8>>) {
        for (&(index, offset), block) in blocks.iter() {
            self.blocks.insert((index, offset), block.clone());
        }
    }

    pub fn request(&mut self) -> Option<(u32, u32, u32)> {
        if let Some(&request) = self.requests.iter().next() {
            self.requests.remove(&request);
            return Some(request);
        }
        None
    }

    pub fn get_piece(&mut self, index: u32) -> Option<Vec<u8>> {
        let mut piece = Vec::new();
        let mut offset = 0;
        while let Some(block) = self.blocks.get_mut(&(index, offset)) {
            piece.append(block);
            offset += CHUNK_LEN;
        }
        if  piece.len() > 0 {
            Some(piece)
        } else {
            None
        }
    }
}


fn download(desc: &mut DataDesc) -> Result<(), io::Error> {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let id = "-01-TORRENT-PEER-RS-".as_bytes();

    let mut client = core.run(Client::connect(&desc.address, &handle))?;
    client = core.run(client.handshake(desc.info_hash.clone(), id))?;
    client = core.run(client.ping())?;

    let mut attempts = TRIES_TO_UNCHOKE;
    loop {
        if 0 == attempts {
            return Err(io::Error::new(io::ErrorKind::Other, "Attempt limit exceeded"));
        }
        if desc.requests.is_empty() {
            break;
        }
        if client.peer_choked && client.peer_intrested {
            client = core.run(client.unchoke_peer())?;
        }
        if client.am_choked {
            client = core.run(client.unchoke_me())?;
            attempts -= 1;
        } else {
            attempts = TRIES_TO_UNCHOKE;
            if let Some(request) = desc.request() {
                client = core.run(client.request(request.0, request.1, request.2))?;
            }
        }
    }
    desc.load_blocks(&client.blocks);
    Ok(())
}

fn create_addr(host: String, port: String) -> Result<SocketAddr, io::Error> {
    let uri = format!("{}:{}", host, port);
    uri.parse().map_err(|e| {
        io::Error::new(io::ErrorKind::Other, format!("{}", e))
    })
}

// ;
fn main() {
    env_logger::init().unwrap();
    let mut args = env::args().collect::<Vec<_>>();
    if args.len() == 1 {
        //5E433EDAE53E68AF02BC2650E057D0FC4FE41FCD
        println!(
            "Usage:\n\t{} {} {} {} {} {} {}...",
            args[0],
            "192.168.0.100",
            "6881",
            "5E433EDAE53E68AF02BC2650E057D0FC4FE41FCD",
            "924600668",
            "524288",
            "1"
        );
    // let address = "127.0.0.1:12345".parse().unwrap();
    // println!("Server in debug mode started on {}", &address);
    // let server = TcpServer::new(PeerProto, address);
    // server.serve(|| Ok(Echo));
    } else {
        args.reverse();
        args.pop();
        let host = args.pop().unwrap();
        let port = args.pop().unwrap();
        let hash = args.pop().unwrap();
        let total_len = args.pop().unwrap().parse::<usize>().unwrap();
        let piece_len = args.pop().unwrap().parse::<usize>().unwrap();

        let address = create_addr(host, port).unwrap();
        let hash_info = hash.as_str().from_hex().unwrap();
        let mut desc = DataDesc::new(address, hash_info, total_len, piece_len);

        let mut indices = Vec::new();
        while let Some(index) = args.pop() {
            if let Some(index) = index.parse::<u32>().ok() {
                indices.push(index);
                desc.add_index(index);
            }
        }
        match download(&mut desc) {
            Ok(_) => {}
            Err(e) => println!("{}", e),
        }
        for index in &indices {
            if let Some(piece) = desc.get_piece(*index) {

                println!("{}", sha1(&piece).to_hex());
            }
        }

    }
}
