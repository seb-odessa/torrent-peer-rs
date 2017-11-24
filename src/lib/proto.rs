use std::io;
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::Framed;
use tokio_proto::pipeline::ServerProto;

use Codec;

pub struct ServiceProto;
impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for ServiceProto {
    // For this protocol style, `Request` matches the `Item` type of the codec's `Decoder`
    type Request = String;

    // For this protocol style, `Response` matches the `Item` type of the codec's `Encoder`
    type Response = String;

    // A bit of boilerplate to hook in the codec:
    type Transport = Framed<T, Codec>;
    type BindTransport = Result<Self::Transport, io::Error>;
    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(Codec))
    }
}