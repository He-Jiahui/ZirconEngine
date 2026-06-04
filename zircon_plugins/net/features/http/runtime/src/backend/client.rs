use std::time::Duration;

use zircon_runtime::core::framework::net::{
    NetError, NetHttpRequestDescriptor, NetHttpResponseDescriptor,
};

use super::method::method_to_reqwest;
use super::security::validate_http_security_policy;

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
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(request.timeout_ms))
        .use_rustls_tls()
        .build()
        .map_err(|error| NetError::Io(error.to_string()))?;
    let mut builder = client.request(method_to_reqwest(request.method), &request.url);
    for (name, value) in &request.headers {
        builder = builder.header(name, value);
    }
    if !request.body.is_empty() {
        builder = builder.body(request.body.clone());
    }
    let response = builder
        .send()
        .await
        .map_err(|error| NetError::Io(error.to_string()))?;
    let status_code = response.status().as_u16();
    let headers = response
        .headers()
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|value| (name.to_string(), value.to_string()))
        })
        .collect::<Vec<_>>();
    let body = response
        .bytes()
        .await
        .map_err(|error| NetError::Io(error.to_string()))?
        .to_vec();
    let mut response = NetHttpResponseDescriptor::new(request.request, status_code, body);
    response.headers = headers;
    Ok(response)
}

fn response_is_retryable(status_code: u16) -> bool {
    matches!(status_code, 408 | 425 | 429 | 500 | 502 | 503 | 504)
}
