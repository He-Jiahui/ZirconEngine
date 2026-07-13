use std::collections::HashSet;

use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::ProjectPluginManifest;

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn enabled_plugin_ids_for_target(
    completed: &ProjectPluginManifest,
    target: RuntimeTargetMode,
) -> HashSet<String> {
    completed
        .enabled_for_target(target)
        .map(|selection| selection.id.clone())
        .collect()
}
