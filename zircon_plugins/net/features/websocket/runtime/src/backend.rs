use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Runtime;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use zircon_plugin_net_runtime::{
    WebSocketRuntimeBackend, WebSocketRuntimeConnection, WebSocketRuntimeListener,
};
use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetEndpoint, NetError, NetEvent, NetTransportKind,
    NetWebSocketCloseReason, NetWebSocketConnectDescriptor, NetWebSocketFrame,
};

type ClientWebSocketStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
type ServerWebSocketStream = WebSocketStream<TcpStream>;

#[derive(Clone, Debug, Default)]
pub struct TungsteniteWebSocketBackend;

#[derive(Debug)]
struct TungsteniteWebSocketListener {
    listener: TcpListener,
    local_endpoint: NetEndpoint,
}

#[derive(Debug)]
struct TungsteniteWebSocketConnection {
    state: Arc<Mutex<NetConnectionState>>,
    outbound: TungsteniteWebSocketSink,
    inbound: Arc<Mutex<VecDeque<NetWebSocketFrame>>>,
}

#[derive(Debug)]
enum TungsteniteWebSocketSink {
    Client(Arc<Mutex<SplitSink<ClientWebSocketStream, tokio_tungstenite::tungstenite::Message>>>),
    Server(Arc<Mutex<SplitSink<ServerWebSocketStream, tokio_tungstenite::tungstenite::Message>>>),
}

enum TungsteniteWebSocketReadHalf {
    Client(SplitStream<ClientWebSocketStream>),
    Server(SplitStream<ServerWebSocketStream>),
}

pub fn websocket_runtime_backend() -> Arc<dyn WebSocketRuntimeBackend> {
    Arc::new(TungsteniteWebSocketBackend)
}

impl WebSocketRuntimeBackend for TungsteniteWebSocketBackend {
    fn listen_websocket(
        &self,
        runtime: &Runtime,
        bind: SocketAddr,
    ) -> Result<Box<dyn WebSocketRuntimeListener>, NetError> {
        let listener = runtime
            .block_on(TcpListener::bind(bind))
            .map_err(|error| NetError::Io(error.to_string()))?;
        let local_endpoint = listener
            .local_addr()
            .map(NetEndpoint::from)
            .map_err(|error| NetError::Io(error.to_string()))?;
        Ok(Box::new(TungsteniteWebSocketListener {
            listener,
            local_endpoint,
        }))
    }

    fn connect_websocket(
        &self,
        runtime: &Runtime,
        connection: NetConnectionId,
        descriptor: NetWebSocketConnectDescriptor,
        events: Arc<Mutex<VecDeque<NetEvent>>>,
    ) -> Result<Box<dyn WebSocketRuntimeConnection>, NetError> {
        validate_websocket_security_policy(&descriptor)?;
        let mut request = descriptor
            .url
            .as_str()
            .into_client_request()
            .map_err(|error| NetError::Io(error.to_string()))?;
        for (name, value) in &descriptor.headers {
            let name = tokio_tungstenite::tungstenite::http::header::HeaderName::from_bytes(
                name.as_bytes(),
            )
            .map_err(|error| NetError::Io(error.to_string()))?;
            let value = tokio_tungstenite::tungstenite::http::HeaderValue::from_str(value)
                .map_err(|error| NetError::Io(error.to_string()))?;
            request.headers_mut().insert(name, value);
        }
        if !descriptor.protocols.is_empty() {
            let protocols = descriptor.protocols.join(", ");
            let value = tokio_tungstenite::tungstenite::http::HeaderValue::from_str(&protocols)
                .map_err(|error| NetError::Io(error.to_string()))?;
            request.headers_mut().insert(
                tokio_tungstenite::tungstenite::http::header::SEC_WEBSOCKET_PROTOCOL,
                value,
            );
        }
        let timeout_duration = Duration::from_millis(descriptor.timeout_ms);
        let (stream, _) = runtime
            .block_on(async {
                timeout(timeout_duration, tokio_tungstenite::connect_async(request)).await
            })
            .map_err(|_| NetError::Io("websocket connect timed out".to_string()))?
            .map_err(|error| NetError::Io(error.to_string()))?;
        let (sink, stream) = stream.split();
        let (network, read_half) = TungsteniteWebSocketConnection::client(sink, stream);
        spawn_reader(runtime, connection, &network, read_half, events);
        Ok(Box::new(network))
    }
}

fn validate_websocket_security_policy(
    descriptor: &NetWebSocketConnectDescriptor,
) -> Result<(), NetError> {
    if descriptor.security.certificate_pinning {
        let host = websocket_url_host(&descriptor.url).ok_or_else(|| {
            NetError::SecurityPolicyViolation {
                reason: "WebSocket certificate pinning requires a valid request host".to_string(),
            }
        })?;
        if !descriptor.security.has_pin_for_host(&host) {
            return Err(NetError::SecurityPolicyViolation {
                reason: format!(
                    "WebSocket certificate pinning has no configured pin for host: {host}"
                ),
            });
        }
    }

    if descriptor.security.tls_required
        && !descriptor.url.starts_with("wss://")
        && !(descriptor.security.allow_insecure_loopback
            && websocket_url_is_loopback(&descriptor.url))
    {
        return Err(NetError::SecurityPolicyViolation {
            reason: "WebSocket connection requires WSS by security policy".to_string(),
        });
    }

    Ok(())
}

fn websocket_url_is_loopback(url: &str) -> bool {
    websocket_url_host(url)
        .is_some_and(|host| matches!(host.as_str(), "localhost" | "127.0.0.1" | "::1" | "[::1]"))
}

fn websocket_url_host(url: &str) -> Option<String> {
    let authority = url
        .strip_prefix("ws://")
        .or_else(|| url.strip_prefix("wss://"))
        .map(|rest| rest.split('/').next().unwrap_or_default())?;
    Some(
        authority
            .rsplit_once('@')
            .map(|(_, host)| host)
            .unwrap_or(authority)
            .split(':')
            .next()
            .unwrap_or_default()
            .to_string(),
    )
}

impl WebSocketRuntimeListener for TungsteniteWebSocketListener {
    fn local_endpoint(&self) -> NetEndpoint {
        self.local_endpoint.clone()
    }

    fn accept_websocket(
        &self,
        runtime: &Runtime,
        connection: NetConnectionId,
        events: Arc<Mutex<VecDeque<NetEvent>>>,
        poll_timeout: Duration,
    ) -> Result<Option<(NetEndpoint, Box<dyn WebSocketRuntimeConnection>)>, NetError> {
        let accept_result =
            runtime.block_on(async { timeout(poll_timeout, self.listener.accept()).await });
        let (stream, remote_addr) = match accept_result {
            Ok(Ok(accepted)) => accepted,
            Ok(Err(error)) => return Err(NetError::Io(error.to_string())),
            Err(_) => return Ok(None),
        };
        let websocket = runtime
            .block_on(tokio_tungstenite::accept_async(stream))
            .map_err(|error| NetError::Io(error.to_string()))?;
        let (sink, stream) = websocket.split();
        let (network, read_half) = TungsteniteWebSocketConnection::server(sink, stream);
        spawn_reader(runtime, connection, &network, read_half, events);
        Ok(Some((NetEndpoint::from(remote_addr), Box::new(network))))
    }
}

impl WebSocketRuntimeConnection for TungsteniteWebSocketConnection {
    fn state(&self) -> NetConnectionState {
        *self
            .state
            .lock()
            .expect("net WebSocket state mutex poisoned")
    }

    fn set_state(&self, state: NetConnectionState) {
        *self
            .state
            .lock()
            .expect("net WebSocket state mutex poisoned") = state;
    }

    fn send(&self, runtime: &Runtime, frame: NetWebSocketFrame) -> Result<(), NetError> {
        runtime.block_on(self.send_async(frame))
    }

    fn drain_frames(&self, max_frames: usize) -> Vec<NetWebSocketFrame> {
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
}

impl TungsteniteWebSocketConnection {
    fn client(
        sink: SplitSink<ClientWebSocketStream, tokio_tungstenite::tungstenite::Message>,
        stream: SplitStream<ClientWebSocketStream>,
    ) -> (Self, TungsteniteWebSocketReadHalf) {
        let state = Arc::new(Mutex::new(NetConnectionState::Open));
        let inbound = Arc::new(Mutex::new(VecDeque::new()));
        (
            Self {
                state,
                outbound: TungsteniteWebSocketSink::Client(Arc::new(Mutex::new(sink))),
                inbound,
            },
            TungsteniteWebSocketReadHalf::Client(stream),
        )
    }

    fn server(
        sink: SplitSink<ServerWebSocketStream, tokio_tungstenite::tungstenite::Message>,
        stream: SplitStream<ServerWebSocketStream>,
    ) -> (Self, TungsteniteWebSocketReadHalf) {
        let state = Arc::new(Mutex::new(NetConnectionState::Open));
        let inbound = Arc::new(Mutex::new(VecDeque::new()));
        (
            Self {
                state,
                outbound: TungsteniteWebSocketSink::Server(Arc::new(Mutex::new(sink))),
                inbound,
            },
            TungsteniteWebSocketReadHalf::Server(stream),
        )
    }

    async fn send_async(&self, frame: NetWebSocketFrame) -> Result<(), NetError> {
        let message = frame_to_message(frame.clone());
        match &self.outbound {
            TungsteniteWebSocketSink::Client(sink) => {
                let mut sink = sink
                    .lock()
                    .expect("net WebSocket client sink mutex poisoned");
                sink.send(message)
                    .await
                    .map_err(|error| NetError::Io(error.to_string()))?;
            }
            TungsteniteWebSocketSink::Server(sink) => {
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

fn spawn_reader(
    runtime: &Runtime,
    connection: NetConnectionId,
    network: &TungsteniteWebSocketConnection,
    read_half: TungsteniteWebSocketReadHalf,
    events: Arc<Mutex<VecDeque<NetEvent>>>,
) {
    runtime.spawn(read_websocket_frames(
        connection,
        read_half,
        network.state.clone(),
        network.inbound.clone(),
        events,
    ));
}

async fn read_websocket_frames(
    connection: NetConnectionId,
    read_half: TungsteniteWebSocketReadHalf,
    state: Arc<Mutex<NetConnectionState>>,
    inbound: Arc<Mutex<VecDeque<NetWebSocketFrame>>>,
    events: Arc<Mutex<VecDeque<NetEvent>>>,
) {
    match read_half {
        TungsteniteWebSocketReadHalf::Client(stream) => {
            read_stream(connection, stream, state, inbound, events).await;
        }
        TungsteniteWebSocketReadHalf::Server(stream) => {
            read_stream(connection, stream, state, inbound, events).await;
        }
    }
}

async fn read_stream<S>(
    connection: NetConnectionId,
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
