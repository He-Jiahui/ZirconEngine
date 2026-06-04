use std::collections::HashSet;

use zircon_runtime::core::framework::net::{
    NetDownloadManifest, NetDownloadProgress, NetDownloadStatus,
};

use super::NetContentDownloadRuntimeManager;

impl NetContentDownloadRuntimeManager {
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
        let mut state = self.state();
        state.manifests.insert(manifest.download, manifest.clone());
        state.progress.insert(manifest.download, progress.clone());
        progress
    }
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
