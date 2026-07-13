use serde::{Deserialize, Serialize};

use crate::serialization::Loaded;

use super::ProjectManifestSummaryError;

/// Lightweight project identity consumed by Hub and other runtime-independent tools.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectManifestSummary {
    pub name: String,
    #[serde(default)]
    pub engine_version_req: Option<String>,
    pub default_scene: String,
    pub format_version: u32,
}

impl ProjectManifestSummary {
    pub fn parse_toml_str(document: &str) -> Result<Loaded<Self>, ProjectManifestSummaryError> {
        super::parse::parse_str(document)
    }

    pub fn parse_toml_bytes(document: &[u8]) -> Result<Loaded<Self>, ProjectManifestSummaryError> {
        let document = std::str::from_utf8(document)
            .map_err(|source| ProjectManifestSummaryError::InvalidUtf8 { source })?;
        Self::parse_toml_str(document)
    }
}
