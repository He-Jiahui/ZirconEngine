use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AssetUri;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetId(Uuid);

impl AssetId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_stable_label(label: &str) -> Self {
        fn hash_with(prefix: &str, value: &str) -> u64 {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            std::hash::Hash::hash(&prefix, &mut hasher);
            std::hash::Hash::hash(&value, &mut hasher);
            std::hash::Hasher::finish(&hasher)
        }

        let high = hash_with("zircon-asset-id/high", label).to_be_bytes();
        let low = hash_with("zircon-asset-id/low", label).to_be_bytes();
        let mut bytes = [0_u8; 16];
        bytes[..8].copy_from_slice(&high);
        bytes[8..].copy_from_slice(&low);
        Self(Uuid::from_bytes(bytes))
    }
}

impl std::fmt::Display for AssetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetKind {
    Texture,
    Shader,
    Material,
    Scene,
    Model,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetMetadata {
    pub id: AssetId,
    pub uri: AssetUri,
    pub kind: AssetKind,
    pub artifact_uri: Option<AssetUri>,
    pub source_hash: String,
    pub importer_version: u32,
    pub config_hash: String,
}

impl AssetMetadata {
    pub fn new(id: AssetId, uri: AssetUri, kind: AssetKind) -> Self {
        Self {
            id,
            uri,
            kind,
            artifact_uri: None,
            source_hash: String::new(),
            importer_version: 0,
            config_hash: String::new(),
        }
    }

    pub fn id(&self) -> AssetId {
        self.id
    }

    pub fn uri(&self) -> &AssetUri {
        &self.uri
    }

    pub fn artifact_uri(&self) -> Option<&AssetUri> {
        self.artifact_uri.as_ref()
    }

    pub fn with_source_hash(mut self, source_hash: impl Into<String>) -> Self {
        self.source_hash = source_hash.into();
        self
    }

    pub fn with_importer_version(mut self, importer_version: u32) -> Self {
        self.importer_version = importer_version;
        self
    }

    pub fn with_config_hash(mut self, config_hash: impl Into<String>) -> Self {
        self.config_hash = config_hash.into();
        self
    }

    pub fn with_artifact_uri(mut self, artifact_uri: AssetUri) -> Self {
        self.artifact_uri = Some(artifact_uri);
        self
    }
}
