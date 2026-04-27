use std::path::PathBuf;

use crate::PluginPackageManifest;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginCandidate {
    pub plugin_id: String,
    pub package_manifest: PluginPackageManifest,
    pub manifest_path: PathBuf,
    pub library_path: PathBuf,
}
