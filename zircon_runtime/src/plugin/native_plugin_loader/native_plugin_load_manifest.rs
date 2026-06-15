use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativePluginLoadManifest {
    #[serde(default)]
    pub plugins: Vec<NativePluginLoadManifestEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativePluginLoadManifestEntry {
    pub id: String,
    pub path: String,
    pub manifest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_report: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub abi: Option<NativePluginLoadManifestAbiV3Contract>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativePluginLoadManifestAbiV3Contract {
    pub abi_version: u32,
    pub descriptor_symbol: String,
    pub descriptor_contract: String,
    pub runtime_entry_source: String,
    pub editor_entry_source: String,
    pub host_function_table: String,
    pub entry_report_contract: String,
    pub behavior_contract: String,
    pub state_snapshot_contract: String,
    pub bridge_method_table: String,
}
