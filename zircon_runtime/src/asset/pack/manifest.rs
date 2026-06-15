use serde::{Deserialize, Serialize};
use std::fmt;

use crate::core::framework::net::ZrPackManifest;

pub const ZRPACK_MAGIC: [u8; 4] = *b"ZRPK";
pub const ZRPACK_FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZrPackDocumentManifest {
    pub pack: ZrPackManifest,
    pub assets: Vec<ZrPackAssetEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZrPackAssetEntry {
    pub path: String,
    pub chunk_hash: [u8; 32],
    pub size: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ZrPackError {
    InvalidMagic,
    UnsupportedVersion(u32),
    HeaderTooSmall,
    ManifestOutOfBounds,
    ManifestDecode(String),
    DuplicateAssetPath(String),
    MissingChunk(String),
    ChunkOutOfBounds(String),
    AssetNotFound(String),
    DeltaBaseManifestMismatch,
    DeltaTargetManifestMismatch,
    SizeOverflow,
}

impl ZrPackDocumentManifest {
    pub fn new(pack: ZrPackManifest, assets: Vec<ZrPackAssetEntry>) -> Self {
        Self { pack, assets }
    }

    pub fn asset(&self, path: &str) -> Option<&ZrPackAssetEntry> {
        self.assets.iter().find(|asset| asset.path == path)
    }
}

impl ZrPackAssetEntry {
    pub fn new(path: impl Into<String>, chunk_hash: [u8; 32], size: u64) -> Self {
        Self {
            path: path.into(),
            chunk_hash,
            size,
        }
    }
}

impl fmt::Display for ZrPackError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMagic => write!(formatter, "zrpack header magic is invalid"),
            Self::UnsupportedVersion(version) => {
                write!(formatter, "zrpack format version {version} is unsupported")
            }
            Self::HeaderTooSmall => write!(formatter, "zrpack header is too small"),
            Self::ManifestOutOfBounds => {
                write!(formatter, "zrpack manifest range is out of bounds")
            }
            Self::ManifestDecode(error) => {
                write!(formatter, "failed to decode zrpack manifest: {error}")
            }
            Self::DuplicateAssetPath(path) => {
                write!(formatter, "zrpack asset path {path} is duplicated")
            }
            Self::MissingChunk(path) => {
                write!(formatter, "zrpack asset {path} references a missing chunk")
            }
            Self::ChunkOutOfBounds(path) => write!(
                formatter,
                "zrpack asset {path} chunk range is out of bounds"
            ),
            Self::AssetNotFound(path) => write!(formatter, "zrpack asset {path} was not found"),
            Self::DeltaBaseManifestMismatch => {
                write!(
                    formatter,
                    "zrpack delta base manifest does not match the installed pack"
                )
            }
            Self::DeltaTargetManifestMismatch => {
                write!(
                    formatter,
                    "zrpack delta target manifest could not be reconstructed"
                )
            }
            Self::SizeOverflow => {
                write!(formatter, "zrpack size does not fit into the binary format")
            }
        }
    }
}

impl std::error::Error for ZrPackError {}
