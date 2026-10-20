//! Validate TLS admission before HTTP I/O. HTTPS verifies chain, hostname and
//! configured certificate pins during the TLS handshake.

use zircon_runtime::core::framework::net::{NetError, NetHttpRequestDescriptor};

/// HTTP 后端发请求前的策略准入；根 manager 的本地路由分支不会经过此函数。
/// 此处只检查 pin 配置覆盖目标 host，不证明证书链有效或摘要匹配。
/// The client enforces certificate verification during the TLS handshake.
pub(super) fn validate_http_security_policy(
    request: &NetHttpRequestDescriptor,
) -> Result<(), NetError> {
    let url = reqwest::Url::parse(&request.url).map_err(|_| NetError::SecurityPolicyViolation {
        reason: "HTTP request requires a valid HTTP URL".to_string(),
    })?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(NetError::SecurityPolicyViolation {
            reason: "HTTP request requires a valid HTTP URL".to_string(),
        });
    }
    if (request.security.certificate_pinning || request.security.has_certificate_roots())
        && url.scheme() != "https"
    {
        return Err(NetError::SecurityPolicyViolation {
            reason: "HTTP certificate roots and pinning require HTTPS".to_string(),
        });
    }
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
        && url.scheme() != "https"
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
    let url = reqwest::Url::parse(url).ok()?;
    if !matches!(url.scheme(), "http" | "https") {
        return None;
    }
    Some(
        url.host_str()?
            .trim_matches(['[', ']'])
            .to_ascii_lowercase(),
    )
}

#[cfg(test)]
mod host_tests {
    use super::{http_url_host, http_url_is_loopback, validate_http_security_policy};
    use zircon_runtime::core::framework::net::{
        NetError, NetHttpMethod, NetHttpRequestDescriptor, NetRequestId, NetSecurityPolicy,
    };

    #[test]
    fn parses_bracketed_ipv6_host_with_or_without_port() {
        assert_eq!(
            http_url_host("https://[::1]/health"),
            Some("::1".to_string())
        );
        assert_eq!(
            http_url_host("https://[2001:db8::1]:8443/health"),
            Some("2001:db8::1".to_string())
        );
        assert!(http_url_host("https://[::1:8443/health").is_none());
        assert_eq!(
            http_url_host("https://[::1]:/health"),
            Some("::1".to_string())
        );
        assert!(http_url_host("https://[::1]:invalid/health").is_none());
    }

    #[test]
    fn bracketed_ipv6_loopback_is_recognized() {
        assert!(http_url_is_loopback("http://[::1]/health"));
        assert!(http_url_is_loopback("http://[::1]:8080/health"));
        assert!(!http_url_is_loopback("http://[2001:db8::1]/health"));
    }

    #[test]
    fn ipv6_certificate_pin_is_checked_against_the_full_host() {
        let mut request = NetHttpRequestDescriptor::new(
            NetRequestId::new(1),
            NetHttpMethod::Get,
            "https://[::1]/health",
        );
        request.security =
            NetSecurityPolicy::production_tls().with_certificate_pin("::1", "sha256/example");

        assert_eq!(validate_http_security_policy(&request), Ok(()));
        request.security.certificate_pins.clear();
        assert_eq!(
            validate_http_security_policy(&request),
            Err(NetError::SecurityPolicyViolation {
                reason: "HTTP certificate pinning has no configured pin for host: ::1".to_string(),
            })
        );
    }
}
