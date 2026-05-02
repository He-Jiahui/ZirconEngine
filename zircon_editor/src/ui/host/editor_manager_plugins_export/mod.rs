use std::path::{Path, PathBuf};

use zircon_runtime::{plugin::PluginPackageManifest, plugin::RuntimePluginCatalog};

use crate::core::editor_plugin::EditorPluginCatalog;

use super::editor_manager::EditorManager;

mod enablement;
mod export_build;
mod manifest_completion;
mod native_registration;
mod package_projection;
mod reports;
mod status;

pub use self::export_build::{EditorExportBuildReport, EditorExportCargoInvocation};
pub use self::reports::{
    EditorPluginEnableReport, EditorPluginFeatureDependencyStatus,
    EditorPluginFeatureSelectionUpdateReport, EditorPluginFeatureStatus,
    EditorPluginSelectionUpdateReport, EditorPluginStatus, EditorPluginStatusReport,
};

impl EditorManager {
    pub fn plugin_directory(&self, project_root: impl AsRef<Path>) -> PathBuf {
        project_root.as_ref().join("zircon_plugins")
    }

    pub fn plugin_catalog(&self) -> Vec<PluginPackageManifest> {
        self.editor_plugin_catalog().package_manifests()
    }

    pub fn runtime_plugin_catalog(&self) -> RuntimePluginCatalog {
        RuntimePluginCatalog::builtin()
    }

    pub fn editor_plugin_catalog(&self) -> EditorPluginCatalog {
        EditorPluginCatalog::builtin(self.runtime_plugin_catalog().package_manifests())
    }

    pub fn editor_plugin_capabilities(&self) -> Vec<String> {
        self.editor_plugin_catalog().capabilities()
    }
}
