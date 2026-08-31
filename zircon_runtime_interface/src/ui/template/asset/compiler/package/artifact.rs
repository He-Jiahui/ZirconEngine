use serde::{Deserialize, Serialize};

use crate::ui::template::UiCompiledAssetPackageValidationReport;

pub const UI_COMPILED_ASSET_TOML_ENVELOPE_SCHEMA_VERSION: u32 = 3;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiCompiledAssetArtifact {
    pub report: UiCompiledAssetPackageValidationReport,
    #[serde(default)]
    pub bytes: Vec<u8>,
}
