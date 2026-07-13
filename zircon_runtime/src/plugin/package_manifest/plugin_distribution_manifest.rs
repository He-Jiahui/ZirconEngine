use serde::{Deserialize, Serialize};

use crate::core::framework::project::ExportPackagingStrategy;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginDistributionManifest {
    #[serde(default)]
    pub forms: Vec<String>,
    #[serde(default)]
    pub default_packaging: Vec<ExportPackagingStrategy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub abi_version: Option<u32>,
    #[serde(default)]
    pub engine_compat: String,
    #[serde(default)]
    pub dist_crate: String,
    #[serde(default)]
    pub descriptor_symbol: String,
    #[serde(default)]
    pub runtime_entry: String,
    #[serde(default)]
    pub editor_entry: String,
    #[serde(default)]
    pub assets: Vec<String>,
}
