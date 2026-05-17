use serde::{Deserialize, Serialize};

use super::NetDownloadId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetDownloadChunk {
    pub id: String,
    pub url: String,
    pub byte_offset: u64,
    pub byte_len: u64,
    pub sha256: String,
    pub resume_from_byte: Option<u64>,
    pub allow_range_resume: bool,
}

impl NetDownloadChunk {
    pub fn new(
        id: impl Into<String>,
        url: impl Into<String>,
        byte_offset: u64,
        byte_len: u64,
        sha256: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            url: url.into(),
            byte_offset,
            byte_len,
            sha256: sha256.into(),
            resume_from_byte: None,
            allow_range_resume: false,
        }
    }

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
