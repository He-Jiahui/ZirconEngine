use zircon_runtime::core::framework::net::{NetDownloadId, NetDownloadProgress};

use super::NetContentDownloadRuntimeManager;

impl NetContentDownloadRuntimeManager {
    pub fn store_resume_bitmap(
        &self,
        download: NetDownloadId,
        completed_chunks: impl IntoIterator<Item = bool>,
    ) {
        self.state()
            .resume_bitmaps
            .insert(download, completed_chunks.into_iter().collect());
    }

    pub fn resume_bitmap(&self, download: NetDownloadId) -> Vec<bool> {
        let state = self.state();
        if let Some(bitmap) = state.resume_bitmaps.get(&download) {
            return bitmap.clone();
        }
        let Some(manifest) = state.manifests.get(&download) else {
            return Vec::new();
        };
        let completed = state
            .progress
            .get(&download)
            .map(|progress| progress.completed_chunks.as_slice())
            .unwrap_or_default();
        manifest
            .chunks
            .iter()
            .map(|chunk| completed.iter().any(|id| id == &chunk.id))
            .collect()
    }

    pub fn apply_resume_bitmap(&self, download: NetDownloadId) -> Option<NetDownloadProgress> {
        let chunk_ids = {
            let state = self.state();
            let manifest = state.manifests.get(&download)?;
            let bitmap = state.resume_bitmaps.get(&download)?;
            manifest
                .chunks
                .iter()
                .zip(bitmap.iter())
                .filter_map(|(chunk, completed)| completed.then(|| chunk.id.clone()))
                .collect::<Vec<_>>()
        };

        let mut progress = self.progress(download)?;
        for chunk_id in chunk_ids {
            progress = self.mark_cache_hit(download, &chunk_id)?;
        }
        Some(progress)
    }
}

impl super::state::NetContentDownloadRuntimeState {
    pub(in crate::manager) fn mark_resume_bitmap_chunk_complete(
        &mut self,
        download: NetDownloadId,
        chunk_id: &str,
    ) {
        let Some(manifest) = self.manifests.get(&download) else {
            return;
        };
        let Some(index) = manifest
            .chunks
            .iter()
            .position(|chunk| chunk.id == chunk_id)
        else {
            return;
        };
        let bitmap = self
            .resume_bitmaps
            .entry(download)
            .or_insert_with(|| vec![false; manifest.chunks.len()]);
        if index < bitmap.len() {
            bitmap[index] = true;
        }
    }
}
