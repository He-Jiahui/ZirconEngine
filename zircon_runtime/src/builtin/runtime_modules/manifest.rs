use crate::core::framework::project::RuntimeProfileId;
use crate::core::framework::project::{ProjectPluginManifest, ProjectPluginSelection};
use crate::plugin::RuntimeProfileDescriptor;

use super::ids::RuntimePluginId;
use crate::core::framework::platform::RuntimeTargetMode;

pub fn manifest_with_mode_baseline(
    target: RuntimeTargetMode,
    manifest_override: Option<&ProjectPluginManifest>,
) -> ProjectPluginManifest {
    let mut manifest = default_manifest_for_target(target);
    let baseline_runtime_ids = manifest
        .selections
        .iter()
        .filter_map(|selection| RuntimePluginId::parse_key(&selection.id))
        .collect::<Vec<_>>();
    if let Some(override_manifest) = manifest_override {
        for selection in &override_manifest.selections {
            let mut selection = selection.clone();
            if let Some(runtime_id) = RuntimePluginId::parse_key(&selection.id)
                .filter(|runtime_id| baseline_runtime_ids.contains(runtime_id))
            {
                selection.id = runtime_id.key().to_string();
            }
            manifest.set_enabled(selection);
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
    #[cfg(feature = "ui")]
    {
        vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Ui, true, true),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::UiDocumentImporter, true, true),
        ]
    }
    #[cfg(not(feature = "ui"))]
    {
        Vec::new()
    }
}
