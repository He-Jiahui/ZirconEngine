use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::StreamExt;
use tokio::runtime::Runtime;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use zircon_plugin_net_runtime::{rustls_client_config_for_websocket, WebSocketRuntimeConnection};
use zircon_runtime::core::framework::net::{
    NetConnectionId, NetError, NetEvent, NetWebSocketConnectDescriptor,
};

use super::connection::TungsteniteWebSocketConnection;
use super::reader::spawn_reader;
use super::security::{validate_websocket_security_policy, websocket_url_host};

pub(super) fn connect_websocket(
    runtime: &Runtime,
    connection: NetConnectionId,
    descriptor: NetWebSocketConnectDescriptor,
    events: Arc<Mutex<VecDeque<NetEvent>>>,
) -> Result<Box<dyn WebSocketRuntimeConnection>, NetError> {
    validate_websocket_security_policy(&descriptor)?;
    let tls_connector = if descriptor.url.starts_with("wss://")
        && (descriptor.security.certificate_pinning || descriptor.security.has_certificate_roots())
    {
        let host = websocket_url_host(&descriptor.url).ok_or_else(|| {
            NetError::SecurityPolicyViolation {
                reason: "WebSocket TLS policy requires a valid request host".to_string(),
            }
        })?;
        Some(tokio_tungstenite::Connector::Rustls(Arc::new(
            rustls_client_config_for_websocket(&descriptor.security, host)?,
        )))
    } else {
        None
    };
    let mut request = descriptor
        .url
        .as_str()
        .into_client_request()
        .map_err(|error| NetError::Io(error.to_string()))?;
    for (name, value) in &descriptor.headers {
        let name =
            tokio_tungstenite::tungstenite::http::header::HeaderName::from_bytes(name.as_bytes())
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
            timeout(timeout_duration, async {
                match tls_connector {
                    Some(connector) => {
                        tokio_tungstenite::connect_async_tls_with_config(
                            request,
                            None,
                            false,
                            Some(connector),
                        )
                        .await
                    }
                    None => tokio_tungstenite::connect_async(request).await,
                }
            })
            .await
        })
        .map_err(|_| NetError::Io("websocket connect timed out".to_string()))?
        .map_err(|error| NetError::Io(error.to_string()))?;
    let (sink, stream) = stream.split();
    let (network, read_half) = TungsteniteWebSocketConnection::client(runtime, sink, stream);
    spawn_reader(runtime, connection, &network, read_half, events);
    Ok(Box::new(network))
}
