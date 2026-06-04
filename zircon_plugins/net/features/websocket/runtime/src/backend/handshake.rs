use zircon_runtime::core::framework::net::NetWebSocketListenerDescriptor;

type ServerHandshakeRequest = tokio_tungstenite::tungstenite::handshake::server::Request;
type ServerHandshakeResponse = tokio_tungstenite::tungstenite::handshake::server::Response;
type ServerHandshakeErrorResponse =
    tokio_tungstenite::tungstenite::handshake::server::ErrorResponse;
type WebSocketHeaderValue = tokio_tungstenite::tungstenite::http::HeaderValue;
type WebSocketStatusCode = tokio_tungstenite::tungstenite::http::StatusCode;

#[derive(Debug)]
pub(super) struct ListenerPolicyCallback {
    pub(super) descriptor: NetWebSocketListenerDescriptor,
}

impl tokio_tungstenite::tungstenite::handshake::server::Callback for ListenerPolicyCallback {
    fn on_request(
        self,
        request: &ServerHandshakeRequest,
        response: ServerHandshakeResponse,
    ) -> Result<ServerHandshakeResponse, ServerHandshakeErrorResponse> {
        apply_listener_policy(&self.descriptor, request, response)
    }
}

fn apply_listener_policy(
    descriptor: &NetWebSocketListenerDescriptor,
    request: &ServerHandshakeRequest,
    mut response: ServerHandshakeResponse,
) -> Result<ServerHandshakeResponse, ServerHandshakeErrorResponse> {
    if let Err(reason) = validate_listener_request(descriptor, request) {
        return Err(websocket_policy_rejection(reason));
    }

    if let Some(protocol) =
        select_websocket_subprotocol(descriptor, request).map_err(websocket_policy_rejection)?
    {
        let value = WebSocketHeaderValue::from_str(&protocol).map_err(|_| {
            websocket_policy_rejection("WebSocket server selected invalid subprotocol")
        })?;
        response.headers_mut().insert(
            tokio_tungstenite::tungstenite::http::header::SEC_WEBSOCKET_PROTOCOL,
            value,
        );
    }

    Ok(response)
}

fn validate_listener_request(
    descriptor: &NetWebSocketListenerDescriptor,
    request: &ServerHandshakeRequest,
) -> Result<(), String> {
    let path = request.uri().path();
    if !descriptor.allowed_paths.is_empty()
        && !descriptor
            .allowed_paths
            .iter()
            .any(|allowed| allowed == path)
    {
        return Err(format!("WebSocket server rejected path: {path}"));
    }

    for (name, expected_value) in &descriptor.required_headers {
        let Some(actual_value) = request.headers().get(name.as_str()) else {
            return Err(format!("WebSocket server rejected missing header: {name}"));
        };
        let actual_value = actual_value
            .to_str()
            .map_err(|_| format!("WebSocket server rejected non-text header: {name}"))?;
        if actual_value != expected_value {
            return Err(format!("WebSocket server rejected header: {name}"));
        }
    }

    Ok(())
}

fn select_websocket_subprotocol(
    descriptor: &NetWebSocketListenerDescriptor,
    request: &ServerHandshakeRequest,
) -> Result<Option<String>, String> {
    if descriptor.allowed_protocols.is_empty() {
        return Ok(None);
    }

    let Some(requested) = request
        .headers()
        .get(tokio_tungstenite::tungstenite::http::header::SEC_WEBSOCKET_PROTOCOL)
        .and_then(|value| value.to_str().ok())
    else {
        return Err("WebSocket server rejected missing subprotocol".to_string());
    };

    requested
        .split(',')
        .map(str::trim)
        .find(|protocol| {
            !protocol.is_empty()
                && descriptor
                    .allowed_protocols
                    .iter()
                    .any(|allowed| allowed == protocol)
        })
        .map(|protocol| Some(protocol.to_string()))
        .ok_or_else(|| "WebSocket server rejected unsupported subprotocol".to_string())
}

fn websocket_policy_rejection(reason: impl Into<String>) -> ServerHandshakeErrorResponse {
    tokio_tungstenite::tungstenite::http::Response::builder()
        .status(WebSocketStatusCode::FORBIDDEN)
        .body(Some(reason.into()))
        .expect("websocket policy rejection response should be valid")
}

pub(super) fn is_websocket_policy_rejection(error: &tokio_tungstenite::tungstenite::Error) -> bool {
    matches!(
        error,
        tokio_tungstenite::tungstenite::Error::Http(response)
            if response.status() == WebSocketStatusCode::FORBIDDEN
    )
}
