//! 把 profile 的插件和特征选择投影到清单；先报告排除 required 项与不存在的选择，再修改可导出视图。
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

/// 在修改清单前对原 profile 逐项报错；required 排除和未知选择会阻断导出。
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

/// 把 profile 选择应用于克隆清单；调用后须刷新 manifest projection 的动态启用状态。
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
#[path = "tests/profile_projection.rs"]
mod tests;
