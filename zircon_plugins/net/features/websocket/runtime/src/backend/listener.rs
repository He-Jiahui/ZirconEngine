use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::StreamExt;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use tokio::time::timeout;
use zircon_plugin_net_runtime::{WebSocketRuntimeConnection, WebSocketRuntimeListener};
use zircon_runtime::core::framework::net::{
    NetConnectionId, NetEndpoint, NetError, NetEvent, NetWebSocketListenerDescriptor,
};

use super::connection::TungsteniteWebSocketConnection;
use super::handshake::{is_websocket_policy_rejection, ListenerPolicyCallback};
use super::reader::spawn_reader;

#[derive(Debug)]
pub(super) struct TungsteniteWebSocketListener {
    listener: TcpListener,
    local_endpoint: NetEndpoint,
    descriptor: NetWebSocketListenerDescriptor,
}

pub(super) fn listen_websocket(
    runtime: &Runtime,
    descriptor: NetWebSocketListenerDescriptor,
) -> Result<TungsteniteWebSocketListener, NetError> {
    let bind = descriptor.bind.to_socket_addr()?;
    let listener = runtime
        .block_on(TcpListener::bind(bind))
        .map_err(|error| NetError::Io(error.to_string()))?;
    let local_endpoint = listener
        .local_addr()
        .map(NetEndpoint::from)
        .map_err(|error| NetError::Io(error.to_string()))?;
    Ok(TungsteniteWebSocketListener {
        listener,
        local_endpoint,
        descriptor,
    })
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
        let websocket = match runtime.block_on(tokio_tungstenite::accept_hdr_async(
            stream,
            ListenerPolicyCallback {
                descriptor: self.descriptor.clone(),
            },
        )) {
            Ok(websocket) => websocket,
            Err(error) if is_websocket_policy_rejection(&error) => return Ok(None),
            Err(error) => return Err(NetError::Io(error.to_string())),
        };
        let (sink, stream) = websocket.split();
        let (network, read_half) = TungsteniteWebSocketConnection::server(sink, stream);
        spawn_reader(runtime, connection, &network, read_half, events);
        Ok(Some((NetEndpoint::from(remote_addr), Box::new(network))))
    }
}
