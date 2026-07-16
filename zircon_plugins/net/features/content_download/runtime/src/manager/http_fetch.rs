use zircon_runtime::core::framework::net::{
    NetDownloadAttemptDescriptor, NetDownloadId, NetError, NetHttpMethod, NetHttpRequestDescriptor,
    NetHttpResponseDescriptor, NetManager, NetRequestId, NetSecurityPolicy,
};

use super::{
    FetchAttemptResponse, NetContentDownloadRuntimeManager, CONTENT_DOWNLOAD_HTTP_RETRY_ATTEMPTS,
    CONTENT_DOWNLOAD_HTTP_TIMEOUT_MS, HTTP_PARTIAL_CONTENT_STATUS,
};

impl NetContentDownloadRuntimeManager {
    pub fn fetch_next_chunk(
        &self,
        download: NetDownloadId,
        chunk_id: &str,
    ) -> Option<zircon_runtime::core::framework::net::NetDownloadProgress> {
        let attempt = self.next_attempt(download, chunk_id)?;
        let Some(prefix) = self.partial_prefix_for_attempt(&attempt) else {
            return self.progress(download);
        };
        let response = match self.fetch_attempt(&attempt) {
            Ok(response) => response,
            Err(diagnostic) => {
                self.mark_attempt_failed(download, chunk_id, diagnostic);
                return self.progress(download);
            }
        };
        if !response.status_code_is_successful() {
            self.mark_attempt_failed(
                download,
                chunk_id,
                format!(
                    "chunk HTTP fetch failed with status: {}",
                    response.status_code
                ),
            );
            return self.progress(download);
        }
        if attempt.range_start.is_some() && response.status_code != HTTP_PARTIAL_CONTENT_STATUS {
            self.mark_attempt_failed(
                download,
                chunk_id,
                format!("chunk range fetch did not return partial content: {chunk_id}"),
            );
            return self.progress(download);
        }

        let mut bytes = prefix;
        bytes.extend_from_slice(&response.body);
        if bytes.len() != attempt.byte_len as usize {
            self.mark_attempt_failed(
                download,
                chunk_id,
                format!(
                    "chunk HTTP fetch length mismatch: expected {} bytes, got {} bytes",
                    attempt.byte_len,
                    bytes.len()
                ),
            );
            return self.progress(download);
        }
        let actual_content_hash = zircon_runtime::asset::pack::zrpack_content_hash(&bytes);
        if !self.chunk_hash_matches(download, chunk_id, &actual_content_hash) {
            let progress = self.mark_attempt_failed(
                download,
                chunk_id,
                format!("chunk hash mismatch: {chunk_id}"),
            )?;
            if progress.status == zircon_runtime::core::framework::net::NetDownloadStatus::Failed {
                self.fail_progress(download, format!("chunk hash mismatch: {chunk_id}"))
            } else {
                Some(progress)
            }
        } else {
            self.store_partial_chunk(download, chunk_id.to_string(), bytes);
            self.mark_chunk_complete(download, chunk_id, &actual_content_hash)
        }
    }

    fn fetch_attempt(
        &self,
        attempt: &NetDownloadAttemptDescriptor,
    ) -> Result<FetchAttemptResponse, String> {
        let Some(net) = self.net().map_err(|error| error.to_string())? else {
            return Err("content download HTTP fetch requires NetManager".to_string());
        };
        fetch_attempt_via_net(net.as_ref(), attempt)
    }
}

fn fetch_attempt_via_net(
    net: &dyn NetManager,
    attempt: &NetDownloadAttemptDescriptor,
) -> Result<FetchAttemptResponse, String> {
    let mut request = NetHttpRequestDescriptor::new(
        NetRequestId::new(attempt.attempt_index as u64 + 1),
        NetHttpMethod::Get,
        attempt.url.clone(),
    );
    request.timeout_ms = CONTENT_DOWNLOAD_HTTP_TIMEOUT_MS;
    request.max_retry_attempts = CONTENT_DOWNLOAD_HTTP_RETRY_ATTEMPTS;
    request.security = NetSecurityPolicy::development();
    if let Some((range_start, range_end)) = attempt_range_bounds(attempt)? {
        request = request.with_byte_range(range_start, range_end);
    }
    let response = net
        .send_http_request(request)
        .map_err(download_http_error_diagnostic)?;
    validate_response_range(attempt, &response)?;
    validate_response_length(attempt, &response)?;
    Ok(FetchAttemptResponse {
        status_code: response.status_code,
        body: response.body,
    })
}

fn attempt_range_bounds(
    attempt: &NetDownloadAttemptDescriptor,
) -> Result<Option<(u64, u64)>, String> {
    let Some(range_start) = attempt.range_start else {
        return Ok(None);
    };
    let chunk_end = attempt
        .byte_offset
        .checked_add(attempt.byte_len)
        .and_then(|end_exclusive| end_exclusive.checked_sub(1))
        .ok_or_else(|| format!("chunk byte range overflow: {}", attempt.chunk_id))?;
    if range_start < attempt.byte_offset || range_start > chunk_end {
        return Err(format!(
            "chunk range start outside byte range: {}",
            attempt.chunk_id
        ));
    }
    Ok(Some((range_start, chunk_end)))
}

fn validate_response_length(
    attempt: &NetDownloadAttemptDescriptor,
    response: &NetHttpResponseDescriptor,
) -> Result<(), String> {
    let expected_body_len = attempt
        .range_start
        .map(|range_start| attempt.byte_offset + attempt.byte_len - range_start)
        .unwrap_or(attempt.byte_len);
    if response.body.len() > expected_body_len as usize {
        return Err(format!(
            "chunk HTTP fetch exceeded expected body length: {}",
            attempt.chunk_id
        ));
    }
    Ok(())
}

fn validate_response_range(
    attempt: &NetDownloadAttemptDescriptor,
    response: &NetHttpResponseDescriptor,
) -> Result<(), String> {
    let Some((range_start, range_end)) = attempt_range_bounds(attempt)? else {
        return Ok(());
    };
    let expected_prefix = format!("bytes {range_start}-{range_end}/");
    response
        .headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("content-range"))
        .filter(|(_, value)| value.starts_with(&expected_prefix))
        .map(|_| ())
        .ok_or_else(|| format!("chunk HTTP content-range mismatch: {}", attempt.chunk_id))
}

fn download_http_error_diagnostic(error: NetError) -> String {
    match error {
        NetError::SecurityPolicyViolation { reason } => {
            format!("chunk HTTP security policy rejected request: {reason}")
        }
        NetError::ProtocolUnavailable { capability } => {
            format!("chunk HTTP fetch unavailable: {capability}")
        }
        NetError::Io(message) => format!("chunk HTTP fetch failed: {message}"),
        other => format!("chunk HTTP fetch failed: {other:?}"),
    }
}
