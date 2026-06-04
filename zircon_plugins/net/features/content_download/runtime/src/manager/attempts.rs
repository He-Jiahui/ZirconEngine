use zircon_runtime::core::framework::net::{
    NetDownloadAttemptDescriptor, NetDownloadChunk, NetDownloadId, NetDownloadManifest,
    NetDownloadProgress, NetDownloadStatus,
};

use super::NetContentDownloadRuntimeManager;

impl NetContentDownloadRuntimeManager {
    pub fn candidate_urls(&self, download: NetDownloadId, chunk_id: &str) -> Option<Vec<String>> {
        let state = self.state();
        let manifest = state.manifests.get(&download)?;
        let chunk = manifest.chunks.iter().find(|chunk| chunk.id == chunk_id)?;
        Some(candidate_urls_for_chunk(manifest, chunk))
    }

    pub fn next_attempt(
        &self,
        download: NetDownloadId,
        chunk_id: &str,
    ) -> Option<NetDownloadAttemptDescriptor> {
        let state = self.state();
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
        let mut state = self.state();
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
        self.state()
            .failed_attempts
            .get(&(download, chunk_id.to_string()))
            .cloned()
            .unwrap_or_default()
    }
}

pub(in crate::manager) fn candidate_urls_for_chunk(
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
