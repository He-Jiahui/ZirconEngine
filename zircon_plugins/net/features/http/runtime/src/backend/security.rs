//! 在实际 HTTP 请求前判断 TLS 要求、loopback 例外和 pin 配置的 host 准入，不执行证书校验。
//! 当前 HTTPS 无 pin 时使用客户端默认 TLS 验证；pin 模式关闭默认无效证书拒绝，并在发送后检查应用层摘要。明文 HTTP 分支没有该摘要检查。

use zircon_runtime::core::framework::net::{NetError, NetHttpRequestDescriptor};

/// HTTP 后端发请求前的策略准入；根 manager 的本地路由分支不会经过此函数。
/// 此处只检查 pin 配置覆盖目标 host，不证明证书链有效或摘要匹配。
/// HTTPS pin 摘要由当前客户端在发送后检查，明文分支没有该检查；调用者必须分别考虑这些信任边界。
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
        .map(|rest| rest.split(['/', '?', '#']).next().unwrap_or_default())?;
    let authority = authority
        .rsplit_once('@')
        .map_or(authority, |(_, host)| host);
    let host = if let Some(bracketed) = authority.strip_prefix('[') {
        let (host, suffix) = bracketed.split_once(']')?;
        if !suffix.is_empty() {
            let port = suffix.strip_prefix(':')?;
            if port.is_empty() || !port.chars().all(|character| character.is_ascii_digit()) {
                return None;
            }
        }
        host
    } else {
        if let Some((host, port)) = authority.rsplit_once(':') {
            if port.is_empty() || !port.chars().all(|character| character.is_ascii_digit()) {
                return None;
            }
            host
        } else {
            authority
        }
    };
    (!host.is_empty()).then(|| host.to_string())
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
        assert!(http_url_host("https://[::1]:/health").is_none());
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
