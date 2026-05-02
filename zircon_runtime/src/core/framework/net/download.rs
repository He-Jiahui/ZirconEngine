use serde::{Deserialize, Serialize};

use super::NetDownloadId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetDownloadChunk {
    pub id: String,
    pub url: String,
    pub byte_offset: u64,
    pub byte_len: u64,
    pub sha256: String,
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
