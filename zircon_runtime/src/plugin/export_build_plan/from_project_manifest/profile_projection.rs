#[cfg(test)]
use std::cell::Cell;
use std::collections::{HashMap, HashSet};

use crate::core::framework::project::{
    ExportProfile, ProjectPluginFeatureSelection, ProjectPluginManifest, ProjectPluginSelection,
};

use super::super::project_manifest_validation::ProjectPluginManifestValidationProjection;

/// Profile-side lookup facts are built once and shared by profile diagnostics and mutation.
/// The ordered `ExportProfile` remains the diagnostic source so duplicate profile entries keep
/// their existing text and order.
pub(super) struct ExportProfileSelectionProjection {
    selected_plugin_ids: HashSet<String>,
    selected_feature_ids: HashMap<String, SelectedProfileFeatureIds>,
    #[cfg(test)]
    selected_plugin_rows_indexed: usize,
    #[cfg(test)]
    selected_feature_owner_rows_indexed: usize,
    #[cfg(test)]
    selected_feature_rows_indexed: usize,
    #[cfg(test)]
    lookup_probes: Cell<usize>,
}

struct SelectedProfileFeatureIds {
    qualified: HashSet<String>,
    short: HashSet<String>,
}

impl SelectedProfileFeatureIds {
    fn from_feature_ids(feature_ids: &[String]) -> Self {
        let qualified_count = feature_ids
            .iter()
            .filter(|feature_id| feature_id.contains('.'))
            .count();
        let short_count = feature_ids.len() - qualified_count;
        let mut qualified = HashSet::with_capacity(qualified_count);
        let mut short = HashSet::with_capacity(short_count);
        for feature_id in feature_ids {
            if feature_id.contains('.') {
                qualified.insert(feature_id.clone());
            } else {
                short.insert(feature_id.clone());
            }
        }
        Self { qualified, short }
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ExportProfileSelectionProjectionMetrics {
    projection_builds: usize,
    selected_plugin_rows_indexed: usize,
    selected_feature_owner_rows_indexed: usize,
    selected_feature_rows_indexed: usize,
    lookup_probes: usize,
}

#[cfg(test)]
std::thread_local! {
    static OBSERVED_PROFILE_PROJECTION_BUILDS: Cell<usize> = const { Cell::new(0) };
}

#[cfg(test)]
fn begin_profile_projection_build_observation() {
    OBSERVED_PROFILE_PROJECTION_BUILDS.with(|builds| builds.set(0));
}

#[cfg(test)]
fn observe_profile_projection_build() {
    OBSERVED_PROFILE_PROJECTION_BUILDS.with(|builds| builds.set(builds.get().saturating_add(1)));
}

#[cfg(test)]
fn observed_profile_projection_builds() -> usize {
    OBSERVED_PROFILE_PROJECTION_BUILDS.with(Cell::get)
}

impl ExportProfileSelectionProjection {
    pub(super) fn new(profile: &ExportProfile) -> Self {
        #[cfg(test)]
        observe_profile_projection_build();

        Self {
            selected_plugin_ids: profile.selected_plugins.iter().cloned().collect(),
            selected_feature_ids: profile
                .features
                .iter()
                .map(|(owner_id, feature_ids)| {
                    (
                        owner_id.clone(),
                        SelectedProfileFeatureIds::from_feature_ids(feature_ids),
                    )
                })
                .collect(),
            #[cfg(test)]
            selected_plugin_rows_indexed: profile.selected_plugins.len(),
            #[cfg(test)]
            selected_feature_owner_rows_indexed: profile.features.len(),
            #[cfg(test)]
            selected_feature_rows_indexed: profile.features.values().map(Vec::len).sum(),
            #[cfg(test)]
            lookup_probes: Cell::new(0),
        }
    }

    #[cfg(test)]
    fn metrics(&self) -> ExportProfileSelectionProjectionMetrics {
        ExportProfileSelectionProjectionMetrics {
            projection_builds: observed_profile_projection_builds(),
            selected_plugin_rows_indexed: self.selected_plugin_rows_indexed,
            selected_feature_owner_rows_indexed: self.selected_feature_owner_rows_indexed,
            selected_feature_rows_indexed: self.selected_feature_rows_indexed,
            lookup_probes: self.lookup_probes.get(),
        }
    }

    #[cfg(test)]
    fn observe_lookup_probe(&self) {
        self.lookup_probes
            .set(self.lookup_probes.get().saturating_add(1));
    }

    pub(super) fn has_selected_plugins(&self) -> bool {
        !self.selected_plugin_ids.is_empty()
    }

    pub(super) fn selects_plugin(&self, plugin_id: &str) -> bool {
        #[cfg(test)]
        self.observe_lookup_probe();
        self.selected_plugin_ids.contains(plugin_id)
    }

    pub(super) fn selects_feature(&self, owner_plugin_id: &str, feature_id: &str) -> bool {
        #[cfg(test)]
        self.observe_lookup_probe();
        let Some(selected_features) = self.selected_feature_ids.get(owner_plugin_id) else {
            return false;
        };
        selected_features.qualified.contains(feature_id)
            || feature_id
                .strip_prefix(owner_plugin_id)
                .and_then(|suffix| suffix.strip_prefix('.'))
                .is_some_and(|short_id| selected_features.short.contains(short_id))
    }
}

pub(super) struct ExportProfileProjectionDiagnostics {
    pub diagnostics: Vec<String>,
    pub fatal_diagnostics: Vec<String>,
}

pub(super) fn export_profile_selection_diagnostics(
    profile: &ExportProfile,
    manifest: &ProjectPluginManifest,
    manifest_projection: &ProjectPluginManifestValidationProjection,
    profile_projection: &ExportProfileSelectionProjection,
) -> ExportProfileProjectionDiagnostics {
    let mut diagnostics = Vec::new();
    let mut fatal_diagnostics = Vec::new();
    if profile_projection.has_selected_plugins() {
        for (selection_index, selection) in manifest.selections.iter().enumerate() {
            if !manifest_projection.selection_is_consumed_by_target(selection_index)
                || !selection.required
                || profile_projection.selects_plugin(&selection.id)
            {
                continue;
            }
            let diagnostic = format!(
                "export profile {} excludes required plugin {} from target {:?}",
                profile.name, selection.id, profile.target_mode
            );
            diagnostics.push(diagnostic.clone());
            fatal_diagnostics.push(diagnostic);
        }
    }

    for plugin_id in &profile.selected_plugins {
        if manifest_projection
            .first_selection_index(plugin_id)
            .is_none()
        {
            let diagnostic = format!(
                "export profile {} selects plugin {} but the project plugin manifest does not contain it",
                profile.name, plugin_id
            );
            diagnostics.push(diagnostic.clone());
            fatal_diagnostics.push(diagnostic);
        }
    }

    for (owner_plugin_id, feature_ids) in &profile.features {
        let Some(owner_index) = manifest_projection.first_selection_index(owner_plugin_id) else {
            let diagnostic = format!(
                "export profile {} selects features for plugin {} but the project plugin manifest does not contain it",
                profile.name, owner_plugin_id
            );
            diagnostics.push(diagnostic.clone());
            fatal_diagnostics.push(diagnostic);
            continue;
        };
        let Some(owner) = manifest.selections.get(owner_index) else {
            continue;
        };
        if profile_projection.has_selected_plugins()
            && !profile_projection.selects_plugin(owner_plugin_id)
        {
            let diagnostic = format!(
                "export profile {} selects features for plugin {} but the plugin is not selected by the profile",
                profile.name, owner_plugin_id
            );
            diagnostics.push(diagnostic.clone());
            fatal_diagnostics.push(diagnostic);
        }
        for feature in owner.features.iter().filter(|feature| {
            feature.enabled
                && feature.required
                && feature.supports_target(profile.target_mode)
                && !profile_projection
                    .selects_feature(owner_plugin_id, feature_short_or_full_id(feature))
        }) {
            let diagnostic = format!(
                "export profile {} excludes required feature {} from plugin {}",
                profile.name, feature.id, owner_plugin_id
            );
            diagnostics.push(diagnostic.clone());
            fatal_diagnostics.push(diagnostic);
        }
        for feature_id in feature_ids {
            if !manifest_projection.selection_contains_profile_feature(owner_index, feature_id) {
                let diagnostic = format!(
                    "export profile {} selects feature {} for plugin {} but the project plugin manifest does not contain it",
                    profile.name, feature_id, owner_plugin_id
                );
                diagnostics.push(diagnostic.clone());
                fatal_diagnostics.push(diagnostic);
            }
        }
    }

    ExportProfileProjectionDiagnostics {
        diagnostics,
        fatal_diagnostics,
    }
}

pub(super) fn project_plugins_for_export_profile(
    profile_projection: &ExportProfileSelectionProjection,
    manifest: &mut ProjectPluginManifest,
) {
    if profile_projection.has_selected_plugins() {
        for selection in &mut manifest.selections {
            if !profile_projection.selects_plugin(&selection.id) {
                selection.enabled = false;
                selection.required = false;
                for feature in &mut selection.features {
                    feature.enabled = false;
                    feature.required = false;
                }
                continue;
            }
            selection.enabled = true;
        }
    }

    for selection in &mut manifest.selections {
        apply_profile_feature_projection(profile_projection, selection);
    }
}

fn apply_profile_feature_projection(
    profile_projection: &ExportProfileSelectionProjection,
    selection: &mut ProjectPluginSelection,
) {
    if !profile_projection
        .selected_feature_ids
        .contains_key(&selection.id)
    {
        return;
    }
    for feature in &mut selection.features {
        feature.enabled =
            profile_projection.selects_feature(&selection.id, feature_short_or_full_id(feature));
    }
}

fn feature_short_or_full_id(feature: &ProjectPluginFeatureSelection) -> &str {
    feature.id.as_str()
}

#[cfg(test)]
mod tests {
    use crate::core::framework::platform::RuntimeTargetMode;
    use crate::core::framework::project::{ExportProfile, ExportTargetPlatform, RuntimeProfileId};

    use super::{
        begin_profile_projection_build_observation, ExportProfileSelectionProjection,
        SelectedProfileFeatureIds,
    };

    #[test]
    fn preallocated_profile_feature_indexes_preserve_contract() {
        let feature_ids = vec![
            "rendering.forward".to_string(),
            "deferred.fast".to_string(),
            "deferred".to_string(),
        ];
        let selected = SelectedProfileFeatureIds::from_feature_ids(&feature_ids);

        assert_eq!(selected.qualified.len(), 2);
        assert!(selected.qualified.contains("rendering.forward"));
        assert!(selected.qualified.contains("deferred.fast"));
        assert_eq!(selected.short.len(), 1);
        assert!(selected.short.contains("deferred"));
    }

    #[test]
    fn profile_selection_projection_build_and_lookup_counts_scale_linearly() {
        for plugin_count in [1, 100, 1_000] {
            for features_per_plugin in [1, 10, 100] {
                let plugin_ids = (0..plugin_count)
                    .map(|index| format!("plugin_{index}"))
                    .collect::<Vec<_>>();
                let mut profile = ExportProfile::new(
                    "linear-profile",
                    RuntimeTargetMode::ClientRuntime,
                    ExportTargetPlatform::Windows,
                    RuntimeProfileId::Client3d,
                )
                .with_selected_plugins(plugin_ids.iter().cloned());
                for plugin_id in &plugin_ids {
                    profile = profile.with_feature_selection(
                        plugin_id.clone(),
                        (0..features_per_plugin)
                            .map(|index| format!("{plugin_id}.feature_{index}")),
                    );
                }

                begin_profile_projection_build_observation();
                let projection = ExportProfileSelectionProjection::new(&profile);
                for plugin_id in &plugin_ids {
                    assert!(projection.selects_plugin(plugin_id));
                    for feature_index in 0..features_per_plugin {
                        assert!(projection.selects_feature(
                            plugin_id,
                            &format!("{plugin_id}.feature_{feature_index}")
                        ));
                    }
                }

                let feature_count = plugin_count * features_per_plugin;
                let metrics = projection.metrics();
                assert_eq!(metrics.projection_builds, 1);
                assert_eq!(metrics.selected_plugin_rows_indexed, plugin_count);
                assert_eq!(metrics.selected_feature_owner_rows_indexed, plugin_count);
                assert_eq!(metrics.selected_feature_rows_indexed, feature_count);
                assert_eq!(metrics.lookup_probes, plugin_count + feature_count);
            }
        }
    }

    #[test]
    fn feature_projection_search_does_not_allocate_normalized_ids() {
        let source = include_str!("profile_projection.rs");
        let allocating_helper = ["normalize_", "profile_feature_id"].concat();
        assert!(
            !source.contains(&allocating_helper),
            "feature matching should compare qualified and short ids without formatting a String"
        );
    }

    #[test]
    fn feature_projection_matches_short_and_qualified_ids_without_normalizing() {
        let profile = ExportProfile::new(
            "feature-matching",
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::Windows,
            RuntimeProfileId::Client3d,
        )
        .with_feature_selection(
            "rendering",
            [
                "rendering.forward".to_string(),
                "deferred.fast".to_string(),
                "deferred".to_string(),
            ],
        );
        let projection = ExportProfileSelectionProjection::new(&profile);

        assert!(projection.selects_feature("rendering", "rendering.deferred"));
        assert!(projection.selects_feature("rendering", "rendering.forward"));
        assert!(!projection.selects_feature("rendering", "rendering.deferred.fast"));
        assert!(!projection.selects_feature("rendering", "rendering.shadow"));
        assert!(!projection.selects_feature("rendering", "other.deferred"));
    }
}
