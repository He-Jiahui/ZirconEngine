use futures_util::stream::SplitStream;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub(super) type TungsteniteMessage = tokio_tungstenite::tungstenite::Message;
pub(super) type ClientWebSocketStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
pub(super) type ServerWebSocketStream = WebSocketStream<TcpStream>;

pub(super) enum TungsteniteWebSocketReadHalf {
    Client(SplitStream<ClientWebSocketStream>),
    Server(SplitStream<ServerWebSocketStream>),
}
