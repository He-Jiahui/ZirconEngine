use serde::{Deserialize, Serialize};

/// 导出记录共用的 256 位摘要载体，可表示产物内容、阶段参数或执行指纹。
/// 字节不携带算法与用途标识；比较前需确认双方来自同一摘要域。
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct ExportDigest([u8; 32]);

impl ExportDigest {
    pub const ZERO: Self = Self([0; 32]);

    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// 阶段输入或输出的角色与定位信息，供指纹计算和恢复回执引用。
/// digest 可缺省（如 dry-run）；恢复方须核验当前磁盘产物，不能仅凭 locator 复用。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportArtifactRef {
    pub key: String,
    pub locator: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub digest: Option<ExportDigest>,
}

impl ExportArtifactRef {
    pub fn new(key: impl Into<String>, locator: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            locator: locator.into(),
            digest: None,
        }
    }

    pub fn with_digest(mut self, digest: ExportDigest) -> Self {
        self.digest = Some(digest);
        self
    }
}

/// 单个阶段的输入、输出及执行指纹；持久化在回执中供后续增量恢复比对。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportStageIo {
    pub inputs: Vec<ExportArtifactRef>,
    pub outputs: Vec<ExportArtifactRef>,
    pub fingerprint: ExportDigest,
}
