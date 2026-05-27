use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use zircon_runtime::core::framework::net::{
    NetDownloadAttemptDescriptor, NetDownloadChunk, NetDownloadId, NetDownloadManifest,
    NetDownloadProgress, NetDownloadStatus, NetError, NetHttpMethod, NetHttpRequestDescriptor,
    NetHttpResponseDescriptor, NetManager, NetRequestId, NetSecurityPolicy,
};

const HTTP_PARTIAL_CONTENT_STATUS: u16 = 206;
const HTTP_SUCCESS_STATUS: u16 = 200;
const CONTENT_DOWNLOAD_HTTP_TIMEOUT_MS: u64 = 30_000;
const CONTENT_DOWNLOAD_HTTP_RETRY_ATTEMPTS: u8 = 1;

#[derive(Clone, Default)]
pub struct NetContentDownloadRuntimeManager {
    state: Arc<Mutex<NetContentDownloadRuntimeState>>,
    net: Option<Arc<dyn NetManager>>,
}

impl std::fmt::Debug for NetContentDownloadRuntimeManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NetContentDownloadRuntimeManager")
            .field("has_net_manager", &self.net.is_some())
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Default)]
struct NetContentDownloadRuntimeState {
    manifests: HashMap<NetDownloadId, NetDownloadManifest>,
    progress: HashMap<NetDownloadId, NetDownloadProgress>,
    cache_hits: HashMap<NetDownloadId, Vec<String>>,
    attempt_indices: HashMap<(NetDownloadId, String), usize>,
    failed_attempts: HashMap<(NetDownloadId, String), Vec<String>>,
    partial_chunks: HashMap<(NetDownloadId, String), Vec<u8>>,
}

impl NetContentDownloadRuntimeManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_net_manager(net: Arc<dyn NetManager>) -> Self {
        Self {
            state: Arc::default(),
            net: Some(net),
        }
    }

    pub fn queue_manifest(&self, manifest: NetDownloadManifest) -> NetDownloadProgress {
        let total_bytes = manifest.chunks.iter().map(|chunk| chunk.byte_len).sum();
        if let Some(diagnostic) = validate_manifest(&manifest) {
            return NetDownloadProgress::new(
                manifest.download,
                NetDownloadStatus::Failed,
                total_bytes,
            )
            .with_diagnostic(diagnostic);
        }
        let progress =
            NetDownloadProgress::new(manifest.download, NetDownloadStatus::Queued, total_bytes);
        let mut state = self
            .state
            .lock()
            .expect("net content download state mutex poisoned");
        state.manifests.insert(manifest.download, manifest.clone());
        state.progress.insert(manifest.download, progress.clone());
        progress
    }

    pub fn candidate_urls(&self, download: NetDownloadId, chunk_id: &str) -> Option<Vec<String>> {
        let state = self
            .state
            .lock()
            .expect("net content download state mutex poisoned");
        let manifest = state.manifests.get(&download)?;
        let chunk = manifest.chunks.iter().find(|chunk| chunk.id == chunk_id)?;
        Some(candidate_urls_for_chunk(manifest, chunk))
    }

    pub fn next_attempt(
        &self,
        download: NetDownloadId,
        chunk_id: &str,
    ) -> Option<NetDownloadAttemptDescriptor> {
        let state = self
            .state
            .lock()
            .expect("net content download state mutex poisoned");
        let manifest = state.manifests.get(&download)?;
        let chunk = manifest.chunks.iter().find(|chunk| chunk.id == chunk_id)?;
        let urls = candidate_urls_for_chunk(manifest, chunk);
        let key = (download, chunk_id.to_string());
        let attempt_index = state.attempt_indices.get(&key).copied().unwrap_or_default();
        let url = urls.get(attempt_index)?;
        Some(attempt_descriptor_for_chunk(
            download,
            chunk,
            url.clone(),
            attempt_index,
        ))
    }

    pub fn mark_attempt_failed(
        &self,
        download: NetDownloadId,
        chunk_id: &str,
        diagnostic: impl Into<String>,
    ) -> Option<NetDownloadProgress> {
        let mut state = self
            .state
            .lock()
            .expect("net content download state mutex poisoned");
        let manifest = state.manifests.get(&download)?;
        let chunk = manifest.chunks.iter().find(|chunk| chunk.id == chunk_id)?;
        let urls = candidate_urls_for_chunk(manifest, chunk);
        let key = (download, chunk_id.to_string());
        let attempt_index = state.attempt_indices.get(&key).copied().unwrap_or_default();
        state
            .failed_attempts
            .entry(key.clone())
            .or_default()
            .push(diagnostic.into());
        let exhausted = attempt_index + 1 >= urls.len();
        let next_attempt_index = if exhausted {
            urls.len()
        } else {
            attempt_index + 1
        };
        state.attempt_indices.insert(key, next_attempt_index);
        let progress = state.progress.get_mut(&download)?;
        if exhausted {
            progress.status = NetDownloadStatus::Failed;
            progress.diagnostic = Some(format!("chunk attempts exhausted: {chunk_id}"));
        } else {
            progress.status = NetDownloadStatus::Downloading;
            progress.diagnostic = Some(format!(
                "chunk attempt failed, switching mirror: {chunk_id}"
            ));
        }
        Some(progress.clone())
    }

    pub fn failed_attempts(&self, download: NetDownloadId, chunk_id: &str) -> Vec<String> {
        self.state
            .lock()
            .expect("net content download state mutex poisoned")
            .failed_attempts
            .get(&(download, chunk_id.to_string()))
            .cloned()
            .unwrap_or_default()
    }

    pub fn store_partial_chunk(
        &self,
        download: NetDownloadId,
        chunk_id: impl Into<String>,
        bytes: Vec<u8>,
    ) {
        self.state
            .lock()
            .expect("net content download state mutex poisoned")
            .partial_chunks
            .insert((download, chunk_id.into()), bytes);
    }

    pub fn partial_chunk_bytes(&self, download: NetDownloadId, chunk_id: &str) -> Vec<u8> {
        self.state
            .lock()
            .expect("net content download state mutex poisoned")
            .partial_chunks
            .get(&(download, chunk_id.to_string()))
            .cloned()
            .unwrap_or_default()
    }

    pub fn fetch_next_chunk(
        &self,
        download: NetDownloadId,
        chunk_id: &str,
    ) -> Option<NetDownloadProgress> {
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
        let actual_sha256 = sha256_hex(&bytes);
        if !self.chunk_hash_matches(download, chunk_id, &actual_sha256) {
            self.fail_progress(download, format!("chunk hash mismatch: {chunk_id}"))
        } else {
            self.store_partial_chunk(download, chunk_id.to_string(), bytes);
            self.mark_chunk_complete(download, chunk_id, &actual_sha256)
        }
    }

    fn fetch_attempt(
        &self,
        attempt: &NetDownloadAttemptDescriptor,
    ) -> Result<FetchAttemptResponse, String> {
        let Some(net) = self.net.as_ref() else {
            return Err("content download HTTP fetch requires NetManager".to_string());
        };
        fetch_attempt_via_net(net.as_ref(), attempt)
    }

    fn partial_prefix_for_attempt(
        &self,
        attempt: &NetDownloadAttemptDescriptor,
    ) -> Option<Vec<u8>> {
        let Some(range_start) = attempt.range_start else {
            return Some(Vec::new());
        };
        let expected_prefix_len = range_start.checked_sub(attempt.byte_offset)? as usize;
        let key = (attempt.download, attempt.chunk_id.clone());
        let prefix = self
            .state
            .lock()
            .expect("net content download state mutex poisoned")
            .partial_chunks
            .get(&key)
            .cloned()
            .unwrap_or_default();
        if prefix.len() == expected_prefix_len {
            Some(prefix)
        } else {
            self.fail_progress(
                attempt.download,
                format!(
                    "chunk resume requires existing partial bytes: {}",
                    attempt.chunk_id
                ),
            )?;
            None
        }
    }

    fn fail_progress(
        &self,
        download: NetDownloadId,
        diagnostic: String,
    ) -> Option<NetDownloadProgress> {
        let mut state = self
            .state
            .lock()
            .expect("net content download state mutex poisoned");
        let progress = state.progress.get_mut(&download)?;
        progress.status = NetDownloadStatus::Failed;
        progress.diagnostic = Some(diagnostic);
        Some(progress.clone())
    }

    pub fn mark_cache_hit(
        &self,
        download: NetDownloadId,
        chunk_id: &str,
    ) -> Option<NetDownloadProgress> {
        let mut state = self
            .state
            .lock()
            .expect("net content download state mutex poisoned");
        let chunk = state
            .manifests
            .get(&download)?
            .chunks
            .iter()
            .find(|chunk| chunk.id == chunk_id)?
            .clone();
        let cache_hits = state.cache_hits.entry(download).or_default();
        if !cache_hits.iter().any(|id| id == chunk_id) {
            cache_hits.push(chunk_id.to_string());
        }
        let progress = state.progress.get_mut(&download)?;
        if !progress.completed_chunks.iter().any(|id| id == chunk_id) {
            progress.completed_chunks.push(chunk_id.to_string());
            progress.downloaded_bytes += chunk.byte_len;
        }
        progress.status = if progress.downloaded_bytes >= progress.total_bytes {
            NetDownloadStatus::Complete
        } else {
            NetDownloadStatus::Downloading
        };
        Some(progress.clone())
    }

    pub fn mark_chunk_complete(
        &self,
        download: NetDownloadId,
        chunk_id: &str,
        actual_sha256: &str,
    ) -> Option<NetDownloadProgress> {
        let mut state = self
            .state
            .lock()
            .expect("net content download state mutex poisoned");
        let chunk = state
            .manifests
            .get(&download)?
            .chunks
            .iter()
            .find(|chunk| chunk.id == chunk_id)?
            .clone();
        let progress = state.progress.get_mut(&download)?;
        if chunk.sha256 != actual_sha256 {
            progress.status = NetDownloadStatus::Failed;
            progress.diagnostic = Some(format!("chunk hash mismatch: {chunk_id}"));
            return Some(progress.clone());
        }
        if !progress.completed_chunks.iter().any(|id| id == chunk_id) {
            progress.completed_chunks.push(chunk_id.to_string());
            progress.downloaded_bytes += chunk.byte_len;
        }
        progress.status = if progress.downloaded_bytes >= progress.total_bytes {
            NetDownloadStatus::Complete
        } else {
            NetDownloadStatus::Downloading
        };
        Some(progress.clone())
    }

    fn chunk_hash_matches(
        &self,
        download: NetDownloadId,
        chunk_id: &str,
        actual_sha256: &str,
    ) -> bool {
        self.state
            .lock()
            .expect("net content download state mutex poisoned")
            .manifests
            .get(&download)
            .and_then(|manifest| manifest.chunks.iter().find(|chunk| chunk.id == chunk_id))
            .is_some_and(|chunk| chunk.sha256 == actual_sha256)
    }

    pub fn progress(&self, download: NetDownloadId) -> Option<NetDownloadProgress> {
        self.state
            .lock()
            .expect("net content download state mutex poisoned")
            .progress
            .get(&download)
            .cloned()
    }

    pub fn cancel_download(&self, download: NetDownloadId) -> Option<NetDownloadProgress> {
        let mut state = self
            .state
            .lock()
            .expect("net content download state mutex poisoned");
        let progress = state.progress.get_mut(&download)?;
        progress.status = NetDownloadStatus::Cancelled;
        progress.diagnostic = Some("download cancelled".to_string());
        Some(progress.clone())
    }

    pub fn cache_hits(&self, download: NetDownloadId) -> Vec<String> {
        self.state
            .lock()
            .expect("net content download state mutex poisoned")
            .cache_hits
            .get(&download)
            .cloned()
            .unwrap_or_default()
    }
}

struct FetchAttemptResponse {
    status_code: u16,
    body: Vec<u8>,
}

impl FetchAttemptResponse {
    fn status_code_is_successful(&self) -> bool {
        matches!(
            self.status_code,
            HTTP_SUCCESS_STATUS | HTTP_PARTIAL_CONTENT_STATUS
        )
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
        request = request.with_header("range", format!("bytes={range_start}-{range_end}"));
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

fn candidate_urls_for_chunk(
    manifest: &NetDownloadManifest,
    chunk: &NetDownloadChunk,
) -> Vec<String> {
    let mut urls = Vec::with_capacity(1 + manifest.mirror_urls.len());
    urls.push(chunk.url.clone());
    urls.extend(
        manifest
            .mirror_urls
            .iter()
            .map(|mirror| format!("{}/{}", mirror.trim_end_matches('/'), chunk.id)),
    );
    urls
}

fn validate_manifest(manifest: &NetDownloadManifest) -> Option<String> {
    if manifest.chunks.is_empty() {
        return Some("download manifest has no chunks".to_string());
    }

    let mut chunk_ids = HashSet::new();
    for chunk in &manifest.chunks {
        if chunk.id.trim().is_empty() {
            return Some("download chunk has empty id".to_string());
        }
        if !chunk_ids.insert(chunk.id.as_str()) {
            return Some(format!("duplicate download chunk id: {}", chunk.id));
        }
        if chunk.url.trim().is_empty() {
            return Some(format!("download chunk has empty URL: {}", chunk.id));
        }
        if chunk.byte_len == 0 {
            return Some(format!("download chunk has zero byte length: {}", chunk.id));
        }
        if chunk.sha256.trim().is_empty() {
            return Some(format!("download chunk has empty sha256: {}", chunk.id));
        }
        if chunk.resume_from_byte.is_some_and(|resume_from_byte| {
            resume_from_byte < chunk.byte_offset
                || resume_from_byte > chunk.byte_offset + chunk.byte_len
        }) {
            return Some(format!(
                "download chunk resume offset outside range: {}",
                chunk.id
            ));
        }
    }
    None
}

fn attempt_descriptor_for_chunk(
    download: NetDownloadId,
    chunk: &NetDownloadChunk,
    url: String,
    attempt_index: usize,
) -> NetDownloadAttemptDescriptor {
    NetDownloadAttemptDescriptor {
        download,
        chunk_id: chunk.id.clone(),
        url,
        byte_offset: chunk.byte_offset,
        byte_len: chunk.byte_len,
        range_start: chunk
            .allow_range_resume
            .then_some(chunk.resume_from_byte.unwrap_or(chunk.byte_offset)),
        attempt_index,
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = ring::digest::digest(&ring::digest::SHA256, bytes);
    digest
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

pub fn net_content_download_runtime_manager() -> NetContentDownloadRuntimeManager {
    NetContentDownloadRuntimeManager::new()
}
