use std::time::Duration;

use http_body_util::{BodyExt, Full, LengthLimitError, Limited};
use hyper::body::Bytes;
use hyper::HeaderMap;
use zircon_plugin_net_runtime::certificate_pin_matches;
use zircon_runtime::core::framework::net::{
    NetError, NetHttpRequestDescriptor, NetHttpResponseDescriptor,
};

use super::http1_client_policy;
use super::method::{method_to_hyper, method_to_reqwest};
use super::security::validate_http_security_policy;
use super::HTTP_RESPONSE_BODY_LIMIT_BYTES;

pub(super) async fn send_http_request(
    request: NetHttpRequestDescriptor,
) -> Result<NetHttpResponseDescriptor, NetError> {
    validate_http_security_policy(&request)?;
    let max_attempts = request.max_retry_attempts.saturating_add(1);
    let mut attempt = 0;
    let mut last_error = None;
    while attempt < max_attempts {
        match send_http_request_once(&request).await {
            Ok(response)
                if response_is_retryable(response.status_code) && attempt + 1 < max_attempts =>
            {
                attempt += 1;
                continue;
            }
            Ok(response) => return Ok(response),
            Err(error) if attempt + 1 < max_attempts => {
                last_error = Some(error);
                attempt += 1;
            }
            Err(error) => return Err(error),
        }
    }
    Err(last_error.unwrap_or_else(|| NetError::Io("HTTP retry attempts exhausted".to_string())))
}

async fn send_http_request_once(
    request: &NetHttpRequestDescriptor,
) -> Result<NetHttpResponseDescriptor, NetError> {
    if request.url.starts_with("http://") {
        return send_http_request_once_hyper(request).await;
    }
    send_http_request_once_reqwest(request).await
}

async fn send_http_request_once_hyper(
    request: &NetHttpRequestDescriptor,
) -> Result<NetHttpResponseDescriptor, NetError> {
    let uri = request
        .url
        .parse::<hyper::Uri>()
        .map_err(|error| NetError::Io(error.to_string()))?;
    let mut builder = hyper::Request::builder()
        .method(method_to_hyper(request.method))
        .uri(uri);
    for (name, value) in &request.headers {
        builder = builder.header(name, value);
    }
    let outbound = builder
        .body(Full::new(Bytes::from(request.body.clone())))
        .map_err(|error| NetError::Io(error.to_string()))?;
    let client = http1_client_policy::plain_http_client();
    let response = tokio::time::timeout(
        Duration::from_millis(request.timeout_ms),
        client.request(outbound),
    )
    .await
    .map_err(|_| NetError::Io("HTTP request timed out".to_string()))?
    .map_err(|error| NetError::Io(error.to_string()))?;
    let status_code = response.status().as_u16();
    let headers = headers_to_descriptor(response.headers());
    let body = Limited::new(response.into_body(), HTTP_RESPONSE_BODY_LIMIT_BYTES)
        .collect()
        .await
        .map_err(|error| {
            if error.downcast_ref::<LengthLimitError>().is_some() {
                response_body_limit_error()
            } else {
                NetError::Io(error.to_string())
            }
        })?
        .to_bytes()
        .to_vec();
    let mut response = NetHttpResponseDescriptor::new(request.request, status_code, body);
    response.headers = headers;
    Ok(response)
}

async fn send_http_request_once_reqwest(
    request: &NetHttpRequestDescriptor,
) -> Result<NetHttpResponseDescriptor, NetError> {
    let mut builder = reqwest::Client::builder()
        .timeout(Duration::from_millis(request.timeout_ms))
        .use_rustls_tls()
        .tls_info(request.security.certificate_pinning);
    if request.security.certificate_pinning {
        builder = builder.danger_accept_invalid_certs(true);
    }
    for root in &request.security.certificate_roots {
        let certificate = reqwest::Certificate::from_der(&root.der)
            .map_err(|error| NetError::Io(error.to_string()))?;
        builder = builder.add_root_certificate(certificate);
    }
    let client = builder
        .build()
        .map_err(|error| NetError::Io(error.to_string()))?;
    let mut request_builder = client.request(method_to_reqwest(request.method), &request.url);
    for (name, value) in &request.headers {
        request_builder = request_builder.header(name, value);
    }
    if !request.body.is_empty() {
        request_builder = request_builder.body(request.body.clone());
    }
    let mut response = request_builder
        .send()
        .await
        .map_err(|error| NetError::Io(error.to_string()))?;
    validate_pinned_peer_certificate(request, &response)?;
    let status_code = response.status().as_u16();
    let headers = headers_to_descriptor(response.headers());
    let declared_length = response.content_length();
    if declared_length.is_some_and(|length| length > HTTP_RESPONSE_BODY_LIMIT_BYTES as u64) {
        return Err(response_body_limit_error());
    }
    let initial_capacity = declared_length
        .and_then(|length| usize::try_from(length).ok())
        .unwrap_or_default()
        .min(HTTP_RESPONSE_BODY_LIMIT_BYTES);
    let mut body = Vec::with_capacity(initial_capacity);
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| NetError::Io(error.to_string()))?
    {
        append_response_chunk(&mut body, &chunk)?;
    }
    let mut response = NetHttpResponseDescriptor::new(request.request, status_code, body);
    response.headers = headers;
    Ok(response)
}

fn append_response_chunk(body: &mut Vec<u8>, chunk: &[u8]) -> Result<(), NetError> {
    let next_length = body
        .len()
        .checked_add(chunk.len())
        .filter(|length| *length <= HTTP_RESPONSE_BODY_LIMIT_BYTES)
        .ok_or_else(response_body_limit_error)?;
    body.reserve(next_length - body.len());
    body.extend_from_slice(chunk);
    Ok(())
}

fn response_body_limit_error() -> NetError {
    NetError::Io(format!(
        "HTTP response body exceeds the {HTTP_RESPONSE_BODY_LIMIT_BYTES}-byte limit"
    ))
}

fn validate_pinned_peer_certificate(
    request: &NetHttpRequestDescriptor,
    response: &reqwest::Response,
) -> Result<(), NetError> {
    if !request.security.certificate_pinning {
        return Ok(());
    }
    let host = http_url_host(&request.url).ok_or_else(|| NetError::SecurityPolicyViolation {
        reason: "HTTP certificate pinning requires a valid request host".to_string(),
    })?;
    let tls_info = response
        .extensions()
        .get::<reqwest::tls::TlsInfo>()
        .ok_or_else(|| NetError::SecurityPolicyViolation {
            reason: format!("HTTP certificate pinning could not inspect peer certificate: {host}"),
        })?;
    let certificate =
        tls_info
            .peer_certificate()
            .ok_or_else(|| NetError::SecurityPolicyViolation {
                reason: format!("HTTP certificate pinning found no peer certificate: {host}"),
            })?;
    if certificate_pin_matches(&request.security, &host, certificate) {
        Ok(())
    } else {
        Err(NetError::SecurityPolicyViolation {
            reason: format!("HTTP certificate pin mismatch for host: {host}"),
        })
    }
}

fn headers_to_descriptor(headers: &HeaderMap) -> Vec<(String, String)> {
    headers
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|value| (name.to_string(), value.to_string()))
        })
        .collect()
}

fn response_is_retryable(status_code: u16) -> bool {
    matches!(status_code, 408 | 425 | 429 | 500 | 502 | 503 | 504)
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
