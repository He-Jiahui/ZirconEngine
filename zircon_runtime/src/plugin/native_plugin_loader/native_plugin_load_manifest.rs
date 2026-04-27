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
}
