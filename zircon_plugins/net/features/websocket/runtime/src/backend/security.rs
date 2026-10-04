use zircon_runtime::core::framework::net::{NetError, NetWebSocketConnectDescriptor};

pub(super) fn validate_websocket_security_policy(
    descriptor: &NetWebSocketConnectDescriptor,
) -> Result<(), NetError> {
    let has_tls_material =
        descriptor.security.certificate_pinning || descriptor.security.has_certificate_roots();
    if has_tls_material && !descriptor.url.starts_with("wss://") {
        return Err(NetError::SecurityPolicyViolation {
            reason: "WebSocket certificate roots and pinning require WSS".to_string(),
        });
    }

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
        .is_some_and(|host| matches!(host.as_str(), "localhost" | "127.0.0.1" | "::1"))
}

pub(super) fn websocket_url_host(url: &str) -> Option<String> {
    let authority = url
        .strip_prefix("ws://")
        .or_else(|| url.strip_prefix("wss://"))
        .map(|rest| rest.split('/').next().unwrap_or_default())?;
    let authority = authority
        .rsplit_once('@')
        .map(|(_, host)| host)
        .unwrap_or(authority);
    let host = if let Some(bracketed) = authority.strip_prefix('[') {
        bracketed.split(']').next().unwrap_or_default()
    } else {
        authority.split(':').next().unwrap_or_default()
    };
    (!host.is_empty()).then(|| host.to_string())
}

#[cfg(test)]
#[path = "tests/security.rs"]
mod tests;
