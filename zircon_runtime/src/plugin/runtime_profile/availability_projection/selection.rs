use std::collections::HashMap;

use crate::builtin::RuntimePluginId;
use crate::core::framework::project::ProjectPluginManifest;

use super::RuntimeProfileDescriptor;

#[cfg(test)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RuntimePluginAvailabilitySelectionMetrics {
    pub manifest_selection_rows: usize,
    pub indexed_lookup_rows: usize,
    pub unique_plugin_rows: usize,
    pub duplicate_merge_rows: usize,
}

#[cfg(test)]
impl RuntimePluginAvailabilitySelectionMetrics {
    pub fn selection_build_steps(self) -> usize {
        self.indexed_lookup_rows
    }
}

pub(super) struct RuntimePluginManifestSelectionProjection {
    pub(super) plugins: Vec<(RuntimePluginId, bool)>,
    #[cfg(test)]
    pub(super) metrics: RuntimePluginAvailabilitySelectionMetrics,
}

pub(super) fn project_manifest_plugin_selections(
    profile: &RuntimeProfileDescriptor,
    manifest: &ProjectPluginManifest,
) -> RuntimePluginManifestSelectionProjection {
    let mut plugins = Vec::<(RuntimePluginId, bool)>::new();
    let mut positions = HashMap::<RuntimePluginId, usize>::new();
    #[cfg(test)]
    let mut metrics = RuntimePluginAvailabilitySelectionMetrics::default();
    for selection in manifest.enabled_for_target(profile.target_mode) {
        #[cfg(test)]
        {
            metrics.manifest_selection_rows += 1;
        }
        let Some(runtime_id) = RuntimePluginId::parse_key(&selection.id) else {
            continue;
        };
        #[cfg(test)]
        {
            metrics.indexed_lookup_rows += 1;
        }
        if merge_runtime_plugin_selection(
            &mut plugins,
            &mut positions,
            runtime_id,
            selection.required,
        ) {
            #[cfg(test)]
            {
                metrics.duplicate_merge_rows += 1;
            }
        } else {
            #[cfg(test)]
            {
                metrics.unique_plugin_rows += 1;
            }
        }
    }
    RuntimePluginManifestSelectionProjection {
        plugins,
        #[cfg(test)]
        metrics,
    }
}

/// Preserves the first selection position while merging required state across every occurrence.
/// Both profile-default and manifest entry points use this one operation so their availability
/// generation and indexed lookup semantics cannot drift.
pub(super) fn merge_runtime_plugin_selection(
    plugins: &mut Vec<(RuntimePluginId, bool)>,
    positions: &mut HashMap<RuntimePluginId, usize>,
    runtime_id: RuntimePluginId,
    required: bool,
) -> bool {
    if let Some(index) = positions.get(&runtime_id).copied() {
        plugins[index].1 = plugins[index].1 || required;
        true
    } else {
        positions.insert(runtime_id.clone(), plugins.len());
        plugins.push((runtime_id, required));
        false
    }
}
