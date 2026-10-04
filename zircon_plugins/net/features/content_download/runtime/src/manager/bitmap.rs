//! 在清单 chunk 顺序与完成 ID 集合间转换外部续传位图，并据位图声明更新完成进度和 cache hit。
//! 这里不读取磁盘内容或计算哈希；位图可信度与清单绑定仍需由恢复入口确认。

use std::collections::HashSet;

use zircon_runtime::core::framework::net::{NetDownloadId, NetDownloadProgress};

use super::NetContentDownloadRuntimeManager;

const LINEAR_COMPLETION_LOOKUP_LIMIT: usize = 8;

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
        indexed_completion_bitmap(
            manifest.chunks.iter().map(|chunk| chunk.id.as_str()),
            completed,
        )
    }

    // TODO: [CR-PLUGIN-NET-0012] 位图直接计入已完成字节；需明确位图来源与缓存哈希验证责任，防止未验证块被标记完成。
    pub fn apply_resume_bitmap(&self, download: NetDownloadId) -> Option<NetDownloadProgress> {
        let mut state = self.state();
        apply_resume_bitmap_to_state(&mut state, download)
    }
}

fn apply_resume_bitmap_to_state(
    state: &mut super::state::NetContentDownloadRuntimeState,
    download: NetDownloadId,
) -> Option<NetDownloadProgress> {
    state.progress.get(&download)?;
    let completed_chunks = {
        let manifest = state.manifests.get(&download)?;
        let bitmap = state.resume_bitmaps.get(&download)?;
        manifest
            .chunks
            .iter()
            .zip(bitmap.iter())
            .filter_map(|(chunk, completed)| completed.then(|| (chunk.id.clone(), chunk.byte_len)))
            .collect::<Vec<_>>()
    };
    if completed_chunks.is_empty() {
        return state.progress.get(&download).cloned();
    }

    let missing_cache_hits = {
        let cache_hits = state.cache_hits.get(&download);
        completed_chunks
            .iter()
            .filter(|(chunk_id, _)| {
                cache_hits
                    .is_none_or(|cache_hits| !cache_hits.iter().any(|cached| cached == chunk_id))
            })
            .map(|(chunk_id, _)| chunk_id.clone())
            .collect::<Vec<_>>()
    };
    state
        .cache_hits
        .entry(download)
        .or_default()
        .extend(missing_cache_hits);

    let progress = state.progress.get_mut(&download)?;
    let missing_progress_chunks = {
        let existing = &progress.completed_chunks;
        completed_chunks
            .iter()
            .filter(|(chunk_id, _)| !existing.iter().any(|current| current == chunk_id))
            .cloned()
            .collect::<Vec<_>>()
    };
    for (chunk_id, byte_len) in missing_progress_chunks {
        progress.completed_chunks.push(chunk_id);
        progress.downloaded_bytes += byte_len;
    }
    progress.status = if progress.downloaded_bytes >= progress.total_bytes {
        zircon_runtime::core::framework::net::NetDownloadStatus::Complete
    } else {
        zircon_runtime::core::framework::net::NetDownloadStatus::Downloading
    };
    Some(progress.clone())
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

fn indexed_completion_bitmap<'a>(
    chunk_ids: impl IntoIterator<Item = &'a str>,
    completed: &[String],
) -> Vec<bool> {
    let chunk_ids = chunk_ids.into_iter();
    if completed.len() <= LINEAR_COMPLETION_LOOKUP_LIMIT {
        return chunk_ids
            .map(|chunk_id| completed.iter().any(|id| id == chunk_id))
            .collect();
    }

    let completed = completed.iter().map(String::as_str).collect::<HashSet<_>>();
    chunk_ids
        .map(|chunk_id| completed.contains(chunk_id))
        .collect()
}

#[cfg(test)]
#[path = "tests/bitmap_completion_index_tests.rs"]
mod completion_index_tests;

#[cfg(test)]
#[path = "tests/bitmap_batched_resume_apply_tests.rs"]
mod batched_resume_apply_tests;
