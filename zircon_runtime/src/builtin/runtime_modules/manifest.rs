use crate::plugin::{ProjectPluginManifest, ProjectPluginSelection};
use crate::plugin::{RuntimeProfileDescriptor, RuntimeProfileId};

use super::{RuntimePluginId, RuntimeTargetMode};

pub fn manifest_with_mode_baseline(
    target: RuntimeTargetMode,
    manifest_override: Option<&ProjectPluginManifest>,
) -> ProjectPluginManifest {
    let mut manifest = default_manifest_for_target(target);
    if let Some(override_manifest) = manifest_override {
        for selection in &override_manifest.selections {
            manifest.set_enabled(selection.clone());
        }
    }
    manifest
}

pub fn manifest_for_runtime_profile(profile_id: RuntimeProfileId) -> ProjectPluginManifest {
    RuntimeProfileDescriptor::for_id(profile_id).project_manifest()
}

pub fn default_manifest_for_target(target: RuntimeTargetMode) -> ProjectPluginManifest {
    let selections = match target {
        RuntimeTargetMode::ClientRuntime => default_ui_plugin_selection(),
        RuntimeTargetMode::ServerRuntime => Vec::new(),
        RuntimeTargetMode::EditorHost => default_ui_plugin_selection(),
    };
    ProjectPluginManifest { selections }
}

fn default_ui_plugin_selection() -> Vec<ProjectPluginSelection> {
    #[cfg(feature = "plugin-ui")]
    {
        vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Ui,
            true,
            true,
        )]
    }
    #[cfg(not(feature = "plugin-ui"))]
    {
        Vec::new()
    }
}
