use tokio_tungstenite::tungstenite::http::Uri;

use zircon_runtime::core::framework::net::{NetError, NetWebSocketConnectDescriptor};

pub(super) fn validate_websocket_security_policy(
    descriptor: &NetWebSocketConnectDescriptor,
) -> Result<Uri, NetError> {
    let uri: Uri = descriptor
        .url
        .parse()
        .map_err(|_| NetError::SecurityPolicyViolation {
            reason: "WebSocket connection requires a valid WS/WSS URI".to_string(),
        })?;
    let secure = uri
        .scheme_str()
        .is_some_and(|scheme| scheme.eq_ignore_ascii_case("wss"));
    if (!secure
        && !uri
            .scheme_str()
            .is_some_and(|scheme| scheme.eq_ignore_ascii_case("ws")))
        || uri.host().is_none()
    {
        return Err(NetError::SecurityPolicyViolation {
            reason: "WebSocket connection requires a valid WS/WSS URI".to_string(),
        });
    }
    let has_tls_material =
        descriptor.security.certificate_pinning || descriptor.security.has_certificate_roots();
    if has_tls_material && !secure {
        return Err(NetError::SecurityPolicyViolation {
            reason: "WebSocket certificate roots and pinning require WSS".to_string(),
        });
    }

    if descriptor.security.certificate_pinning {
        let host = websocket_uri_host(&uri).ok_or_else(|| {
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
        && !secure
        && !(descriptor.security.allow_insecure_loopback
            && websocket_uri_host(&uri)
                .is_some_and(|host| matches!(host.as_str(), "localhost" | "127.0.0.1" | "::1")))
    {
        return Err(NetError::SecurityPolicyViolation {
            reason: "WebSocket connection requires WSS by security policy".to_string(),
        });
    }

    // Use this exact parsed authority and normalized scheme for the connection.
    let mut parts = uri.into_parts();
    parts.scheme = Some(
        if secure { "wss" } else { "ws" }
            .parse()
            .expect("static WS scheme"),
    );
    Uri::from_parts(parts).map_err(|error| NetError::Io(error.to_string()))
}

pub(super) fn websocket_uri_host(uri: &Uri) -> Option<String> {
    Some(uri.host()?.trim_matches(['[', ']']).to_ascii_lowercase())
}

#[cfg(test)]
fn websocket_url_host(url: &str) -> Option<String> {
    websocket_uri_host(&url.parse().ok()?)
}

#[cfg(test)]
#[path = "tests/security.rs"]
mod tests;
