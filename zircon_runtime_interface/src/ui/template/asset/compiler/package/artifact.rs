use serde::{Deserialize, Serialize};

use crate::ui::template::UiCompiledAssetPackageValidationReport;

pub const UI_COMPILED_ASSET_BINARY_ARTIFACT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiCompiledAssetArtifact {
    pub report: UiCompiledAssetPackageValidationReport,
    #[serde(default)]
    pub bytes: Vec<u8>,
}
