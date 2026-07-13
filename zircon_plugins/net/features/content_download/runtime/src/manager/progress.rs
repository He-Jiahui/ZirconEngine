use zircon_runtime::core::framework::net::{NetDownloadId, NetDownloadProgress, NetDownloadStatus};

use super::NetContentDownloadRuntimeManager;

impl NetContentDownloadRuntimeManager {
    pub(in crate::manager) fn fail_progress(
        &self,
        download: NetDownloadId,
        diagnostic: String,
    ) -> Option<NetDownloadProgress> {
        let mut state = self.state();
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
        let mut state = self.state();
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
        let progress = progress.clone();
        state.mark_resume_bitmap_chunk_complete(download, chunk_id);
        Some(progress)
    }

    pub fn mark_chunk_complete(
        &self,
        download: NetDownloadId,
        chunk_id: &str,
        actual_content_hash: &[u8; 32],
    ) -> Option<NetDownloadProgress> {
        let mut state = self.state();
        let chunk = state
            .manifests
            .get(&download)?
            .chunks
            .iter()
            .find(|chunk| chunk.id == chunk_id)?
            .clone();
        let progress = state.progress.get_mut(&download)?;
        if chunk.content_hash != *actual_content_hash {
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
        let progress = progress.clone();
        state.mark_resume_bitmap_chunk_complete(download, chunk_id);
        Some(progress)
    }

    pub fn progress(&self, download: NetDownloadId) -> Option<NetDownloadProgress> {
        self.state().progress.get(&download).cloned()
    }

    pub fn cancel_download(&self, download: NetDownloadId) -> Option<NetDownloadProgress> {
        let mut state = self.state();
        let progress = state.progress.get_mut(&download)?;
        progress.status = NetDownloadStatus::Cancelled;
        progress.diagnostic = Some("download cancelled".to_string());
        Some(progress.clone())
    }

    pub fn cache_hits(&self, download: NetDownloadId) -> Vec<String> {
        self.state()
            .cache_hits
            .get(&download)
            .cloned()
            .unwrap_or_default()
    }

    pub(in crate::manager) fn chunk_hash_matches(
        &self,
        download: NetDownloadId,
        chunk_id: &str,
        actual_content_hash: &[u8; 32],
    ) -> bool {
        self.state()
            .manifests
            .get(&download)
            .and_then(|manifest| manifest.chunks.iter().find(|chunk| chunk.id == chunk_id))
            .is_some_and(|chunk| chunk.content_hash == *actual_content_hash)
    }
}
