use zircon_runtime::core::framework::net::{NetError, NetHttpRequestDescriptor};

pub(super) fn validate_http_security_policy(
    request: &NetHttpRequestDescriptor,
) -> Result<(), NetError> {
    if request.security.certificate_pinning {
        let host =
            http_url_host(&request.url).ok_or_else(|| NetError::SecurityPolicyViolation {
                reason: "HTTP certificate pinning requires a valid request host".to_string(),
            })?;
        if !request.security.has_pin_for_host(&host) {
            return Err(NetError::SecurityPolicyViolation {
                reason: format!("HTTP certificate pinning has no configured pin for host: {host}"),
            });
        }
    }

    if request.security.tls_required
        && !request.url.starts_with("https://")
        && !(request.security.allow_insecure_loopback && http_url_is_loopback(&request.url))
    {
        return Err(NetError::SecurityPolicyViolation {
            reason: "HTTP request requires HTTPS by security policy".to_string(),
        });
    }

    Ok(())
}

fn http_url_is_loopback(url: &str) -> bool {
    http_url_host(url)
        .is_some_and(|host| matches!(host.as_str(), "localhost" | "127.0.0.1" | "::1" | "[::1]"))
}

fn http_url_host(url: &str) -> Option<String> {
    let authority = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
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
