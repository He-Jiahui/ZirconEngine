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
            .insert((download, chunk_id.into()), bytes);
    }

    pub fn partial_chunk_bytes(&self, download: NetDownloadId, chunk_id: &str) -> Vec<u8> {
        self.state()
            .partial_chunks
            .get(&(download, chunk_id.to_string()))
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
        let key = (attempt.download, attempt.chunk_id.clone());
        let prefix = self
            .state()
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
}
