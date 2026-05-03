use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use zircon_runtime::core::framework::net::{
    NetConnectionState, NetError, NetEvent, NetTransportKind, NetWebSocketCloseReason,
    NetWebSocketFrame,
};

pub(crate) type ClientWebSocketStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
pub(crate) type ServerWebSocketStream = WebSocketStream<TcpStream>;

#[derive(Debug)]
pub(crate) struct ManagedWebSocketListener {
    pub listener: tokio::net::TcpListener,
    pub local_endpoint: zircon_runtime::core::framework::net::NetEndpoint,
}

#[derive(Debug)]
pub(crate) enum ManagedWebSocketConnection {
    Loopback(LoopbackWebSocketConnection),
    Network(NetworkWebSocketConnection),
}

#[derive(Debug)]
pub(crate) struct LoopbackWebSocketConnection {
    pub peer: zircon_runtime::core::framework::net::NetConnectionId,
    pub state: NetConnectionState,
    pub inbound: VecDeque<NetWebSocketFrame>,
}

#[derive(Debug)]
pub(crate) struct NetworkWebSocketConnection {
    pub state: Arc<Mutex<NetConnectionState>>,
    pub outbound: NetworkWebSocketSink,
    pub inbound: Arc<Mutex<VecDeque<NetWebSocketFrame>>>,
}

#[derive(Debug)]
pub(crate) enum NetworkWebSocketSink {
    Client(Arc<Mutex<SplitSink<ClientWebSocketStream, tokio_tungstenite::tungstenite::Message>>>),
    Server(Arc<Mutex<SplitSink<ServerWebSocketStream, tokio_tungstenite::tungstenite::Message>>>),
}

pub(crate) enum NetworkWebSocketReadHalf {
    Client(SplitStream<ClientWebSocketStream>),
    Server(SplitStream<ServerWebSocketStream>),
}

impl ManagedWebSocketConnection {
    pub(crate) fn state(&self) -> NetConnectionState {
        match self {
            Self::Loopback(connection) => connection.state,
            Self::Network(connection) => *connection
                .state
                .lock()
                .expect("net WebSocket state mutex poisoned"),
        }
    }
}

impl NetworkWebSocketConnection {
    pub(crate) fn client(
        sink: SplitSink<ClientWebSocketStream, tokio_tungstenite::tungstenite::Message>,
        stream: SplitStream<ClientWebSocketStream>,
    ) -> (Self, NetworkWebSocketReadHalf) {
        let state = Arc::new(Mutex::new(NetConnectionState::Open));
        let inbound = Arc::new(Mutex::new(VecDeque::new()));
        (
            Self {
                state,
                outbound: NetworkWebSocketSink::Client(Arc::new(Mutex::new(sink))),
                inbound,
            },
            NetworkWebSocketReadHalf::Client(stream),
        )
    }

    pub(crate) fn server(
        sink: SplitSink<ServerWebSocketStream, tokio_tungstenite::tungstenite::Message>,
        stream: SplitStream<ServerWebSocketStream>,
    ) -> (Self, NetworkWebSocketReadHalf) {
        let state = Arc::new(Mutex::new(NetConnectionState::Open));
        let inbound = Arc::new(Mutex::new(VecDeque::new()));
        (
            Self {
                state,
                outbound: NetworkWebSocketSink::Server(Arc::new(Mutex::new(sink))),
                inbound,
            },
            NetworkWebSocketReadHalf::Server(stream),
        )
    }

    pub(crate) fn set_state(&self, state: NetConnectionState) {
        *self
            .state
            .lock()
            .expect("net WebSocket state mutex poisoned") = state;
    }

    pub(crate) fn drain_frames(&self, max_frames: usize) -> Vec<NetWebSocketFrame> {
        let mut inbound = self
            .inbound
            .lock()
            .expect("net WebSocket inbound mutex poisoned");
        let mut frames = Vec::new();
        while frames.len() < max_frames {
            match inbound.pop_front() {
                Some(NetWebSocketFrame::Close(reason)) => {
                    *self
                        .state
                        .lock()
                        .expect("net WebSocket state mutex poisoned") = NetConnectionState::Closed;
                    frames.push(NetWebSocketFrame::Close(reason));
                }
                Some(frame) => frames.push(frame),
                None => break,
            }
        }
        frames
    }

    pub(crate) async fn send(&self, frame: NetWebSocketFrame) -> Result<(), NetError> {
        let message = frame_to_message(frame.clone());
        match &self.outbound {
            NetworkWebSocketSink::Client(sink) => {
                let mut sink = sink
                    .lock()
                    .expect("net WebSocket client sink mutex poisoned");
                sink.send(message)
                    .await
                    .map_err(|error| NetError::Io(error.to_string()))?;
            }
            NetworkWebSocketSink::Server(sink) => {
                let mut sink = sink
                    .lock()
                    .expect("net WebSocket server sink mutex poisoned");
                sink.send(message)
                    .await
                    .map_err(|error| NetError::Io(error.to_string()))?;
            }
        }
        if matches!(frame, NetWebSocketFrame::Close(_)) {
            self.set_state(NetConnectionState::Closing);
        }
        Ok(())
    }
}

pub(crate) async fn read_websocket_frames(
    connection: zircon_runtime::core::framework::net::NetConnectionId,
    read_half: NetworkWebSocketReadHalf,
    state: Arc<Mutex<NetConnectionState>>,
    inbound: Arc<Mutex<VecDeque<NetWebSocketFrame>>>,
    events: Arc<Mutex<VecDeque<NetEvent>>>,
) {
    match read_half {
        NetworkWebSocketReadHalf::Client(stream) => {
            read_stream(connection, stream, state, inbound, events).await;
        }
        NetworkWebSocketReadHalf::Server(stream) => {
            read_stream(connection, stream, state, inbound, events).await;
        }
    }
}

async fn read_stream<S>(
    connection: zircon_runtime::core::framework::net::NetConnectionId,
    mut stream: SplitStream<WebSocketStream<S>>,
    state: Arc<Mutex<NetConnectionState>>,
    inbound: Arc<Mutex<VecDeque<NetWebSocketFrame>>>,
    events: Arc<Mutex<VecDeque<NetEvent>>>,
) where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    while let Some(message) = stream.next().await {
        match message {
            Ok(message) => {
                let frame = message_to_frame(message);
                let mut queue = inbound
                    .lock()
                    .expect("net WebSocket inbound mutex poisoned");
                queue.push_back(frame.clone());
                let queued_frames = queue.len();
                drop(queue);
                events.lock().expect("net events mutex poisoned").push_back(
                    NetEvent::WebSocketFrameQueued {
                        connection,
                        queued_frames,
                    },
                );
                if matches!(frame, NetWebSocketFrame::Close(_)) {
                    *state.lock().expect("net WebSocket state mutex poisoned") =
                        NetConnectionState::Closed;
                    events
                        .lock()
                        .expect("net events mutex poisoned")
                        .push_back(NetEvent::ConnectionClosed { connection });
                    return;
                }
            }
            Err(error) => {
                *state.lock().expect("net WebSocket state mutex poisoned") =
                    NetConnectionState::Failed;
                events.lock().expect("net events mutex poisoned").push_back(
                    NetEvent::ConnectionStateChanged {
                        connection,
                        transport: NetTransportKind::WebSocket,
                        state: NetConnectionState::Failed,
                    },
                );
                events.lock().expect("net events mutex poisoned").push_back(
                    NetEvent::WebSocketFrameQueued {
                        connection,
                        queued_frames: 0,
                    },
                );
                let _ = error;
                return;
            }
        }
    }
    *state.lock().expect("net WebSocket state mutex poisoned") = NetConnectionState::Closed;
    events
        .lock()
        .expect("net events mutex poisoned")
        .push_back(NetEvent::ConnectionClosed { connection });
}

fn frame_to_message(frame: NetWebSocketFrame) -> tokio_tungstenite::tungstenite::Message {
    match frame {
        NetWebSocketFrame::Text(text) => tokio_tungstenite::tungstenite::Message::Text(text.into()),
        NetWebSocketFrame::Binary(bytes) => {
            tokio_tungstenite::tungstenite::Message::Binary(bytes.into())
        }
        NetWebSocketFrame::Ping(bytes) => {
            tokio_tungstenite::tungstenite::Message::Ping(bytes.into())
        }
        NetWebSocketFrame::Pong(bytes) => {
            tokio_tungstenite::tungstenite::Message::Pong(bytes.into())
        }
        NetWebSocketFrame::Close(reason) => tokio_tungstenite::tungstenite::Message::Close(Some(
            tokio_tungstenite::tungstenite::protocol::CloseFrame {
                code: close_code_from_u16(reason.code),
                reason: reason.reason.into(),
            },
        )),
    }
}

fn message_to_frame(message: tokio_tungstenite::tungstenite::Message) -> NetWebSocketFrame {
    match message {
        tokio_tungstenite::tungstenite::Message::Text(text) => {
            NetWebSocketFrame::Text(text.to_string())
        }
        tokio_tungstenite::tungstenite::Message::Binary(bytes) => {
            NetWebSocketFrame::Binary(bytes.to_vec())
        }
        tokio_tungstenite::tungstenite::Message::Ping(bytes) => {
            NetWebSocketFrame::Ping(bytes.to_vec())
        }
        tokio_tungstenite::tungstenite::Message::Pong(bytes) => {
            NetWebSocketFrame::Pong(bytes.to_vec())
        }
        tokio_tungstenite::tungstenite::Message::Close(reason) => {
            let reason = reason
                .map(|reason| NetWebSocketCloseReason {
                    code: u16::from(reason.code),
                    reason: reason.reason.to_string(),
                    clean: true,
                })
                .unwrap_or_else(|| NetWebSocketCloseReason::normal("peer closed"));
            NetWebSocketFrame::Close(reason)
        }
        tokio_tungstenite::tungstenite::Message::Frame(_) => {
            NetWebSocketFrame::Close(NetWebSocketCloseReason::normal("raw frame unsupported"))
        }
    }
}

fn close_code_from_u16(
    code: u16,
) -> tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode {
    use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;

    match code {
        1000 => CloseCode::Normal,
        1001 => CloseCode::Away,
        1002 => CloseCode::Protocol,
        1003 => CloseCode::Unsupported,
        1005 => CloseCode::Status,
        1006 => CloseCode::Abnormal,
        1007 => CloseCode::Invalid,
        1008 => CloseCode::Policy,
        1009 => CloseCode::Size,
        1010 => CloseCode::Extension,
        1011 => CloseCode::Error,
        1012 => CloseCode::Restart,
        1013 => CloseCode::Again,
        other => CloseCode::Library(other),
    }
}
