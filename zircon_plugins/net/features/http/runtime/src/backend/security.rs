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

// BUG: [CR-PLUGIN-NET-0008] 按冒号拆 host 会把 IPv6 字面量截成 '['，无法识别 ::1 的 loopback 例外或匹配其 pin。
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
