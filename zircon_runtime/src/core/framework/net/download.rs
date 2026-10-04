use serde::{Deserialize, Serialize};

use super::NetDownloadId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 内容清单中的一个可校验块；偏移和长度指资源字节范围，哈希覆盖完整块，镜像失败后仍按同一块身份重试。
pub struct NetDownloadChunk {
    pub id: String,
    pub url: String,
    pub byte_offset: u64,
    pub byte_len: u64,
    pub content_hash: [u8; 32],
    pub resume_from_byte: Option<u64>,
    pub allow_range_resume: bool,
}

impl NetDownloadChunk {
    pub fn new(
        id: impl Into<String>,
        url: impl Into<String>,
        byte_offset: u64,
        byte_len: u64,
        content_hash: [u8; 32],
    ) -> Self {
        Self {
            id: id.into(),
            url: url.into(),
            byte_offset,
            byte_len,
            content_hash,
            resume_from_byte: None,
            allow_range_resume: false,
        }
    }

    // BUG: [CR-FRAMEWORK-NET-0001] 完整块末尾的续传偏移会通过清单校验，但 HTTP 拉取将其判为非法范围并使下载失败。
    pub fn with_resume_from_byte(mut self, resume_from_byte: u64) -> Self {
        self.resume_from_byte = Some(resume_from_byte);
        self.allow_range_resume = true;
        self
    }

    pub fn with_range_resume(mut self, allow_range_resume: bool) -> Self {
        self.allow_range_resume = allow_range_resume;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 某块的一次候选 URL 拉取；range_start 是本次 HTTP Range 的起点，失败后 attempt_index 推进到镜像。
pub struct NetDownloadAttemptDescriptor {
    pub download: NetDownloadId,
    pub chunk_id: String,
    pub url: String,
    pub byte_offset: u64,
    pub byte_len: u64,
    pub range_start: Option<u64>,
    pub attempt_index: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 下载任务的稳定块清单；先提交给下载管理器校验，再按块选择主 URL 与镜像，进度按整个清单累计。
pub struct NetDownloadManifest {
    pub download: NetDownloadId,
    pub resource_id: String,
    pub chunks: Vec<NetDownloadChunk>,
    pub mirror_urls: Vec<String>,
}

impl NetDownloadManifest {
    pub fn new(download: NetDownloadId, resource_id: impl Into<String>) -> Self {
        Self {
            download,
            resource_id: resource_id.into(),
            chunks: Vec::new(),
            mirror_urls: Vec::new(),
        }
    }

    pub fn with_chunk(mut self, chunk: NetDownloadChunk) -> Self {
        self.chunks.push(chunk);
        self
    }

    pub fn with_mirror_url(mut self, url: impl Into<String>) -> Self {
        self.mirror_urls.push(url.into());
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetDownloadStatus {
    Queued,
    Downloading,
    Verifying,
    Complete,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetDownloadProgress {
    pub download: NetDownloadId,
    pub status: NetDownloadStatus,
    pub completed_chunks: Vec<String>,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub diagnostic: Option<String>,
}

impl NetDownloadProgress {
    pub fn new(download: NetDownloadId, status: NetDownloadStatus, total_bytes: u64) -> Self {
        Self {
            download,
            status,
            completed_chunks: Vec::new(),
            downloaded_bytes: 0,
            total_bytes,
            diagnostic: None,
        }
    }

    pub fn with_diagnostic(mut self, diagnostic: impl Into<String>) -> Self {
        self.diagnostic = Some(diagnostic.into());
        self
    }
}
