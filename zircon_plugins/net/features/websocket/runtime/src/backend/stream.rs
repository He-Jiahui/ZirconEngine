//! 封装 client TLS stream 与明文 server stream 的读半部差异，供同一 WS reader 接口消费。
//! 该枚举只统一内部类型，不意味着服务器已具备 TLS listener。

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
