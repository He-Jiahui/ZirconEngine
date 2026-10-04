//! 保存每个下载/chunk 的内存前缀，并在 range fetch 前核对前缀长度与清单偏移。
//! 恢复字节仍属 manager 内存；调用者负责持久化、来源验证和与同一清单绑定。

use zircon_runtime::core::framework::net::{NetDownloadAttemptDescriptor, NetDownloadId};

use super::NetContentDownloadRuntimeManager;

impl NetContentDownloadRuntimeManager {
    pub fn store_partial_chunk(
        &self,
        download: NetDownloadId,
        chunk_id: impl Into<String>,
        bytes: Vec<u8>,
    ) {
        self.state()
            .partial_chunks
            .entry(download)
            .or_default()
            .insert(chunk_id.into(), bytes);
    }

    pub fn partial_chunk_bytes(&self, download: NetDownloadId, chunk_id: &str) -> Vec<u8> {
        self.state()
            .partial_chunks
            .get(&download)
            .and_then(|chunks| chunks.get(chunk_id))
            .cloned()
            .unwrap_or_default()
    }

    pub(in crate::manager) fn partial_prefix_for_attempt(
        &self,
        attempt: &NetDownloadAttemptDescriptor,
    ) -> Option<Vec<u8>> {
        let Some(range_start) = attempt.range_start else {
            return Some(Vec::new());
        };
        let expected_prefix_len = range_start.checked_sub(attempt.byte_offset)? as usize;
        let prefix = self
            .state()
            .partial_chunks
            .get(&attempt.download)
            .and_then(|chunks| chunks.get(attempt.chunk_id.as_str()))
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
}

#[cfg(test)]
#[path = "tests/resume_nested_partial_chunk_tests.rs"]
mod nested_partial_chunk_tests;
