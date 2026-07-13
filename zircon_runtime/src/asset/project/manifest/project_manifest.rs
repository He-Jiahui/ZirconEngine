use serde::{Deserialize, Serialize};

use crate::asset::AssetUri;
use crate::{
    core::framework::project::ExportProfile, core::framework::project::ProjectPluginManifest,
};
use zircon_runtime_interface::project::{
    ProjectManifestSummary, RelPath, PROJECT_MANIFEST_FORMAT_VERSION,
};

use super::export_profiles::deserialize_export_profiles;
use super::validation::default_asset_roots;
use crate::asset::project::ProjectScriptManifest;

/// Authoritative runtime project document. `format_version` describes this structure;
/// `library_version` independently describes the generated asset-library contents.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub name: String,
    #[serde(default = "default_project_format_version")]
    pub format_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine_version_req: Option<String>,
    pub default_scene: AssetUri,
    #[serde(default = "default_asset_roots")]
    pub asset_roots: Vec<RelPath>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settings: Option<RelPath>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_manifest: Option<String>,
    #[serde(alias = "schema_version")]
    pub library_version: u32,
    #[serde(default, skip_serializing_if = "ProjectPluginManifest::is_empty")]
    pub plugins: ProjectPluginManifest,
    #[serde(default, skip_serializing_if = "ProjectScriptManifest::is_empty")]
    pub scripts: ProjectScriptManifest,
    #[serde(
        default,
        deserialize_with = "deserialize_export_profiles",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub export_profiles: Vec<ExportProfile>,
}

impl ProjectManifest {
    pub fn new(name: impl Into<String>, default_scene: AssetUri, library_version: u32) -> Self {
        Self {
            name: name.into(),
            format_version: PROJECT_MANIFEST_FORMAT_VERSION,
            engine_version_req: None,
            default_scene,
            asset_roots: default_asset_roots(),
            settings: None,
            asset_manifest: None,
            library_version,
            plugins: ProjectPluginManifest::default(),
            scripts: ProjectScriptManifest::default(),
            export_profiles: Vec::new(),
        }
    }

    pub fn summary(&self) -> ProjectManifestSummary {
        ProjectManifestSummary {
            name: self.name.clone(),
            engine_version_req: self.engine_version_req.clone(),
            default_scene: self.default_scene.to_string(),
            format_version: self.format_version,
        }
    }
}

fn default_project_format_version() -> u32 {
    PROJECT_MANIFEST_FORMAT_VERSION
}
