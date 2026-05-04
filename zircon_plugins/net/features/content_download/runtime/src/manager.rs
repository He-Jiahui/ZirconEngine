use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use zircon_runtime::core::framework::net::{
    NetDownloadId, NetDownloadManifest, NetDownloadProgress, NetDownloadStatus,
};

#[derive(Clone, Debug, Default)]
pub struct NetContentDownloadRuntimeManager {
    state: Arc<Mutex<NetContentDownloadRuntimeState>>,
}

#[derive(Debug, Default)]
struct NetContentDownloadRuntimeState {
    manifests: HashMap<NetDownloadId, NetDownloadManifest>,
    progress: HashMap<NetDownloadId, NetDownloadProgress>,
    cache_hits: HashMap<NetDownloadId, Vec<String>>,
}

impl NetContentDownloadRuntimeManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn queue_manifest(&self, manifest: NetDownloadManifest) -> NetDownloadProgress {
        let total_bytes = manifest.chunks.iter().map(|chunk| chunk.byte_len).sum();
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
        let mut urls = Vec::with_capacity(1 + manifest.mirror_urls.len());
        urls.push(chunk.url.clone());
        urls.extend(
            manifest
                .mirror_urls
                .iter()
                .map(|mirror| format!("{}/{}", mirror.trim_end_matches('/'), chunk.id)),
        );
        Some(urls)
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

    pub fn progress(&self, download: NetDownloadId) -> Option<NetDownloadProgress> {
        self.state
            .lock()
            .expect("net content download state mutex poisoned")
            .progress
            .get(&download)
            .cloned()
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

pub fn net_content_download_runtime_manager() -> NetContentDownloadRuntimeManager {
    NetContentDownloadRuntimeManager::new()
}
