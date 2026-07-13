use serde::{Deserialize, Serialize};

/// Stable, algorithm-neutral 256-bit content digest used by export records.
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

/// A named pipeline artifact and its stable location/content identity.
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

/// Explicit inputs, outputs, and fingerprint for one export stage.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportStageIo {
    pub inputs: Vec<ExportArtifactRef>,
    pub outputs: Vec<ExportArtifactRef>,
    pub fingerprint: ExportDigest,
}
