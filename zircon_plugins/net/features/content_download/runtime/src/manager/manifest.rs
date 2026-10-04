//! 验证下载清单的 chunk ID、URL、长度、范围及总长度后入队；无效输入直接返回 Failed，不污染管理态。
//! 清单是后续尝试/续传的基准，已入队下载按其 chunk 顺序恢复位图。

use std::collections::HashSet;

use zircon_runtime::core::framework::net::{
    NetDownloadManifest, NetDownloadProgress, NetDownloadStatus,
};

use super::NetContentDownloadRuntimeManager;

impl NetContentDownloadRuntimeManager {
    /// 验证清单后建立内存下载进度；同一 ID 再入队会替换清单，恢复数据需与新版本重新绑定。
    pub fn queue_manifest(&self, manifest: NetDownloadManifest) -> NetDownloadProgress {
        let Some(total_bytes) = manifest
            .chunks
            .iter()
            .try_fold(0u64, |total, chunk| total.checked_add(chunk.byte_len))
        else {
            return NetDownloadProgress::new(manifest.download, NetDownloadStatus::Failed, 0)
                .with_diagnostic("download manifest total byte size overflow");
        };
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
        // TODO: [CR-PLUGIN-NET-0009] 同一下载 ID 替换清单时未清理旧 attempts/prefix/bitmap；确认 ID 复用与清单版本合同。
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
        let Some(chunk_end) = chunk.byte_offset.checked_add(chunk.byte_len) else {
            return Some(format!("download chunk byte range overflow: {}", chunk.id));
        };
        if chunk.resume_from_byte.is_some_and(|resume_from_byte| {
            resume_from_byte < chunk.byte_offset || resume_from_byte > chunk_end
        }) {
            return Some(format!(
                "download chunk resume offset outside range: {}",
                chunk.id
            ));
        }
    }
    None
}
