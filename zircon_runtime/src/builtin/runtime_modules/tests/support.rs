use crate::builtin::RuntimePluginId;
use crate::core::framework::project::ProjectPluginSelection;
use crate::plugin::{
    PluginPackageManifest, RuntimeExtensionRegistry, RuntimePluginAvailabilityEntry,
    RuntimePluginRegistrationReport,
};

pub(super) fn linked_runtime_registration(
    plugin_id: RuntimePluginId,
) -> RuntimePluginRegistrationReport {
    RuntimePluginRegistrationReport {
        package_manifest: PluginPackageManifest::new(
            plugin_id.key(),
            format!("{} runtime", plugin_id.key()),
        )
        .with_runtime_crate(format!("zircon_plugin_{}_runtime", plugin_id.key())),
        project_selection: ProjectPluginSelection::runtime_plugin(plugin_id, true, true),
        extensions: RuntimeExtensionRegistry::default(),
        diagnostics: Vec::new(),
    }
}

pub(super) fn availability_contains(
    entries: &[RuntimePluginAvailabilityEntry],
    plugin_id: RuntimePluginId,
) -> bool {
    entries.iter().any(|entry| entry.runtime_id == plugin_id)
}
