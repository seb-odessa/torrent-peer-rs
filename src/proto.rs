use std::io;
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::Framed;
use tokio_proto::pipeline::ClientProto;
use tokio_proto::pipeline::ServerProto;

use Message;
use Messages;
use PeerCodec;

pub struct PeerProto;
impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for PeerProto {
    type Request = Messages;
    type Response = Message;
    type Transport = Framed<T, PeerCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;
    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(PeerCodec))
    }
}

impl<T: AsyncRead + AsyncWrite + 'static> ClientProto<T> for PeerProto {
    type Request = Message;
    type Response = Messages;
    type Transport = Framed<T, PeerCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;
    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(PeerCodec))
    }
}
