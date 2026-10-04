//! 一代清单共用的身份、重复项与 provider 索引；profile 投影后刷新启用状态，诊断及清理仍引用原行位置。
#[cfg(test)]
use std::cell::Cell;
use std::collections::{HashMap, HashSet};

use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::{
    ProjectPluginFeatureSelection, ProjectPluginManifest, ProjectPluginSelection,
};

use super::identity::{project_plugin_feature_id_is_valid, project_plugin_package_id_is_valid};

#[cfg(test)]
#[path = "projection/tests/capacity_tests.rs"]
mod capacity_tests;

/// Per-generation facts for a plugin manifest.  The projection owns its keys so callers may
/// sanitize a clone of the manifest after validation without retaining a borrow into it.
///
/// `refresh` is deliberately the only place that observes enabled/target state.  Profile
/// projection may change those flags, but it must not rebuild separate duplicate or provider
/// caches for every downstream export consumer.
pub(in crate::plugin::export_build_plan) struct ProjectPluginManifestValidationProjection {
    target: RuntimeTargetMode,
    selections: Vec<SelectionProjection>,
    selection_indices: HashMap<String, Vec<usize>>,
    feature_locations: HashMap<String, Vec<FeatureLocation>>,
    enabled_provider_package_ids: HashSet<String>,
    #[cfg(test)]
    selection_rows_indexed: usize,
    #[cfg(test)]
    feature_rows_indexed: usize,
    #[cfg(test)]
    selection_rows_refreshed: Cell<usize>,
    #[cfg(test)]
    feature_rows_refreshed: Cell<usize>,
    #[cfg(test)]
    lookup_probes: Cell<usize>,
}

struct SelectionProjection {
    package_id_valid: bool,
    consumed_by_target: bool,
    duplicate_first_required: Option<bool>,
    feature_indices: HashMap<String, Vec<usize>>,
    feature_ids: HashSet<String>,
    short_feature_ids: HashSet<String>,
    features: Vec<FeatureProjection>,
}

struct FeatureProjection {
    id_valid: bool,
    consumed_by_target: bool,
    duplicate_first_required: Option<bool>,
}

#[derive(Clone, Copy)]
struct FeatureLocation {
    selection_index: usize,
    feature_index: usize,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ProjectPluginManifestValidationMetrics {
    projection_builds: usize,
    selection_rows_indexed: usize,
    feature_rows_indexed: usize,
    selection_rows_refreshed: usize,
    feature_rows_refreshed: usize,
    lookup_probes: usize,
}

#[cfg(test)]
std::thread_local! {
    static OBSERVED_PROJECTION_BUILDS: Cell<usize> = const { Cell::new(0) };
}

#[cfg(test)]
pub(in crate::plugin::export_build_plan) fn begin_projection_build_observation() {
    OBSERVED_PROJECTION_BUILDS.with(|builds| builds.set(0));
}

#[cfg(test)]
fn observe_projection_build() {
    OBSERVED_PROJECTION_BUILDS.with(|builds| builds.set(builds.get().saturating_add(1)));
}

#[cfg(test)]
pub(in crate::plugin::export_build_plan) fn observed_projection_builds() -> usize {
    OBSERVED_PROJECTION_BUILDS.with(Cell::get)
}

impl ProjectPluginManifestValidationProjection {
    pub(in crate::plugin::export_build_plan) fn new(
        manifest: &ProjectPluginManifest,
        target: RuntimeTargetMode,
    ) -> Self {
        #[cfg(test)]
        observe_projection_build();

        let mut selections = Vec::with_capacity(manifest.selections.len());
        let mut selection_indices =
            HashMap::<String, Vec<usize>>::with_capacity(manifest.selections.len());
        let mut feature_locations = HashMap::<String, Vec<FeatureLocation>>::new();

        for (selection_index, selection) in manifest.selections.iter().enumerate() {
            selection_indices
                .entry(selection.id.clone())
                .or_default()
                .push(selection_index);

            let mut feature_indices =
                HashMap::<String, Vec<usize>>::with_capacity(selection.features.len());
            let mut feature_ids = HashSet::with_capacity(selection.features.len());
            let mut short_feature_ids = HashSet::new();
            let mut features = Vec::with_capacity(selection.features.len());
            for (feature_index, feature) in selection.features.iter().enumerate() {
                feature_indices
                    .entry(feature.id.clone())
                    .or_default()
                    .push(feature_index);
                feature_ids.insert(feature.id.clone());
                if let Some(short_id) = feature
                    .id
                    .strip_prefix(&selection.id)
                    .and_then(|suffix| suffix.strip_prefix('.'))
                {
                    short_feature_ids.insert(short_id.to_string());
                }
                feature_locations
                    .entry(feature.id.clone())
                    .or_default()
                    .push(FeatureLocation {
                        selection_index,
                        feature_index,
                    });
                features.push(FeatureProjection {
                    id_valid: project_plugin_feature_id_is_valid(&selection.id, &feature.id),
                    consumed_by_target: false,
                    duplicate_first_required: None,
                });
            }
            selections.push(SelectionProjection {
                package_id_valid: project_plugin_package_id_is_valid(&selection.id),
                consumed_by_target: false,
                duplicate_first_required: None,
                feature_indices,
                feature_ids,
                short_feature_ids,
                features,
            });
        }

        let mut projection = Self {
            target,
            selections,
            selection_indices,
            feature_locations,
            enabled_provider_package_ids: HashSet::with_capacity(manifest.selections.len()),
            #[cfg(test)]
            selection_rows_indexed: manifest.selections.len(),
            #[cfg(test)]
            feature_rows_indexed: manifest
                .selections
                .iter()
                .map(|selection| selection.features.len())
                .sum(),
            #[cfg(test)]
            selection_rows_refreshed: Cell::new(0),
            #[cfg(test)]
            feature_rows_refreshed: Cell::new(0),
            #[cfg(test)]
            lookup_probes: Cell::new(0),
        };
        projection.refresh(manifest);
        projection
    }

    /// Recomputes only dynamic target/enable facts after profile application.  Static lookup
    /// topology and identity validation remain owned by this same generation projection.
    /// profile 可能改写 enabled，但不会改行拓扑；刷新目标消费与 provider 资格后再做诊断和特征解析。
    pub(in crate::plugin::export_build_plan) fn refresh(
        &mut self,
        manifest: &ProjectPluginManifest,
    ) {
        debug_assert_eq!(self.selections.len(), manifest.selections.len());
        self.enabled_provider_package_ids.clear();
        let mut seen_selection_ids =
            HashMap::<&str, bool>::with_capacity(manifest.selections.len());

        for (selection_index, selection) in manifest.selections.iter().enumerate() {
            #[cfg(test)]
            self.selection_rows_refreshed
                .set(self.selection_rows_refreshed.get().saturating_add(1));
            let Some(selection_projection) = self.selections.get_mut(selection_index) else {
                break;
            };
            selection_projection.consumed_by_target =
                selection.enabled && selection.supports_target(self.target);
            selection_projection.duplicate_first_required = None;
            if selection_projection.consumed_by_target {
                if let Some(first_required) = seen_selection_ids.get(selection.id.as_str()) {
                    selection_projection.duplicate_first_required = Some(*first_required);
                } else {
                    seen_selection_ids.insert(&selection.id, selection.required);
                }
                self.enabled_provider_package_ids
                    .insert(selection.id.clone());
            }

            let mut seen_feature_ids =
                HashMap::<&str, bool>::with_capacity(selection.features.len());
            for (feature_index, feature) in selection.features.iter().enumerate() {
                #[cfg(test)]
                self.feature_rows_refreshed
                    .set(self.feature_rows_refreshed.get().saturating_add(1));
                let Some(feature_projection) = selection_projection.features.get_mut(feature_index)
                else {
                    break;
                };
                feature_projection.consumed_by_target = selection_projection.consumed_by_target
                    && feature.enabled
                    && feature.supports_target(self.target);
                feature_projection.duplicate_first_required = None;
                if feature_projection.consumed_by_target {
                    if let Some(first_required) = seen_feature_ids.get(feature.id.as_str()) {
                        feature_projection.duplicate_first_required = Some(*first_required);
                    } else {
                        seen_feature_ids.insert(&feature.id, feature.required);
                    }
                }
            }
        }
    }

    #[cfg(test)]
    fn metrics(&self) -> ProjectPluginManifestValidationMetrics {
        ProjectPluginManifestValidationMetrics {
            projection_builds: observed_projection_builds(),
            selection_rows_indexed: self.selection_rows_indexed,
            feature_rows_indexed: self.feature_rows_indexed,
            selection_rows_refreshed: self.selection_rows_refreshed.get(),
            feature_rows_refreshed: self.feature_rows_refreshed.get(),
            lookup_probes: self.lookup_probes.get(),
        }
    }

    #[cfg(test)]
    fn observe_lookup_probe(&self) {
        self.lookup_probes
            .set(self.lookup_probes.get().saturating_add(1));
    }

    pub(in crate::plugin::export_build_plan) fn first_selection_index(
        &self,
        selection_id: &str,
    ) -> Option<usize> {
        #[cfg(test)]
        self.observe_lookup_probe();
        self.selection_indices
            .get(selection_id)
            .and_then(|indices| indices.first().copied())
    }

    pub(super) fn first_feature_index(
        &self,
        selection_id: &str,
        feature_id: &str,
    ) -> Option<usize> {
        #[cfg(test)]
        self.observe_lookup_probe();
        self.first_selection_index(selection_id)
            .and_then(|selection_index| {
                self.selections
                    .get(selection_index)
                    .and_then(|selection| selection.feature_indices.get(feature_id))
                    .and_then(|indices| indices.first().copied())
            })
    }

    pub(super) fn provider_is_enabled_for_target(&self, provider_package_id: &str) -> bool {
        #[cfg(test)]
        self.observe_lookup_probe();
        self.enabled_provider_package_ids
            .contains(provider_package_id)
    }

    pub(super) fn duplicate_selection_first_required(
        &self,
        selection_index: usize,
    ) -> Option<bool> {
        #[cfg(test)]
        self.observe_lookup_probe();
        self.selections
            .get(selection_index)
            .and_then(|selection| selection.duplicate_first_required)
    }

    pub(super) fn duplicate_feature_first_required(
        &self,
        selection_index: usize,
        feature_index: usize,
    ) -> Option<bool> {
        #[cfg(test)]
        self.observe_lookup_probe();
        self.selections
            .get(selection_index)
            .and_then(|selection| selection.features.get(feature_index))
            .and_then(|feature| feature.duplicate_first_required)
    }

    pub(in crate::plugin::export_build_plan) fn selection_is_consumed_by_target(
        &self,
        selection_index: usize,
    ) -> bool {
        #[cfg(test)]
        self.observe_lookup_probe();
        self.selections
            .get(selection_index)
            .is_some_and(|selection| selection.consumed_by_target)
    }

    pub(super) fn feature_is_consumed_by_target(
        &self,
        selection_index: usize,
        feature_index: usize,
    ) -> bool {
        #[cfg(test)]
        self.observe_lookup_probe();
        self.selections
            .get(selection_index)
            .and_then(|selection| selection.features.get(feature_index))
            .is_some_and(|feature| feature.consumed_by_target)
    }

    pub(super) fn selection_package_id_is_valid(&self, selection_index: usize) -> bool {
        #[cfg(test)]
        self.observe_lookup_probe();
        self.selections
            .get(selection_index)
            .is_some_and(|selection| selection.package_id_valid)
    }

    pub(super) fn feature_id_is_valid(&self, selection_index: usize, feature_index: usize) -> bool {
        #[cfg(test)]
        self.observe_lookup_probe();
        self.selections
            .get(selection_index)
            .and_then(|selection| selection.features.get(feature_index))
            .is_some_and(|feature| feature.id_valid)
    }

    pub(super) fn selection_retained_after_identity_sanitize(
        &self,
        selection_index: usize,
    ) -> bool {
        #[cfg(test)]
        self.observe_lookup_probe();
        self.selections
            .get(selection_index)
            .is_some_and(|selection| {
                !selection.consumed_by_target
                    || (selection.package_id_valid && selection.duplicate_first_required.is_none())
            })
    }

    pub(super) fn feature_retained_after_identity_sanitize(
        &self,
        selection_index: usize,
        feature_index: usize,
    ) -> bool {
        #[cfg(test)]
        self.observe_lookup_probe();
        self.selections
            .get(selection_index)
            .and_then(|selection| selection.features.get(feature_index))
            .is_some_and(|feature| {
                !feature.consumed_by_target
                    || (feature.id_valid && feature.duplicate_first_required.is_none())
            })
    }

    pub(in crate::plugin::export_build_plan) fn selection_contains_profile_feature(
        &self,
        selection_index: usize,
        selected_feature_id: &str,
    ) -> bool {
        #[cfg(test)]
        self.observe_lookup_probe();
        self.selections
            .get(selection_index)
            .is_some_and(|selection| {
                if selected_feature_id.contains('.') {
                    selection.feature_ids.contains(selected_feature_id)
                } else {
                    selection.short_feature_ids.contains(selected_feature_id)
                }
            })
    }

    pub(in crate::plugin::export_build_plan) fn feature_selection<'a>(
        &self,
        manifest: &'a ProjectPluginManifest,
        feature_id: &str,
    ) -> Option<(
        &'a ProjectPluginSelection,
        &'a ProjectPluginFeatureSelection,
    )> {
        #[cfg(test)]
        self.observe_lookup_probe();
        self.feature_locations
            .get(feature_id)
            .and_then(|locations| {
                locations.iter().find_map(|location| {
                    let selection = manifest.selections.get(location.selection_index)?;
                    let feature = selection.features.get(location.feature_index)?;
                    Some((selection, feature))
                })
            })
    }

    /// 仅返回目标消费、身份合法且 provider 也启用的外部特征；建计划据此链接或收集原生包。
    pub(in crate::plugin::export_build_plan) fn external_feature_selections<'a>(
        &self,
        manifest: &'a ProjectPluginManifest,
    ) -> Vec<(
        &'a ProjectPluginSelection,
        &'a ProjectPluginFeatureSelection,
    )> {
        let mut selections = Vec::new();
        for (selection_index, selection) in manifest.selections.iter().enumerate() {
            for (feature_index, feature) in selection.features.iter().enumerate() {
                if self.external_feature_matches(manifest, selection_index, feature_index) {
                    selections.push((selection, feature));
                }
            }
        }
        selections
    }

    pub(in crate::plugin::export_build_plan) fn external_feature_selection<'a>(
        &self,
        manifest: &'a ProjectPluginManifest,
        feature_id: &str,
    ) -> Option<(
        &'a ProjectPluginSelection,
        &'a ProjectPluginFeatureSelection,
    )> {
        #[cfg(test)]
        self.observe_lookup_probe();
        self.feature_locations
            .get(feature_id)
            .and_then(|locations| {
                locations.iter().find_map(|location| {
                    self.external_feature_matches(
                        manifest,
                        location.selection_index,
                        location.feature_index,
                    )
                    .then(|| {
                        let selection = manifest.selections.get(location.selection_index)?;
                        let feature = selection.features.get(location.feature_index)?;
                        Some((selection, feature))
                    })
                    .flatten()
                })
            })
    }

    fn external_feature_matches(
        &self,
        manifest: &ProjectPluginManifest,
        selection_index: usize,
        feature_index: usize,
    ) -> bool {
        if !self.selection_is_consumed_by_target(selection_index)
            || !self.feature_is_consumed_by_target(selection_index, feature_index)
            || !self.selection_package_id_is_valid(selection_index)
            || !self.feature_id_is_valid(selection_index, feature_index)
        {
            return false;
        }
        let Some(selection) = manifest.selections.get(selection_index) else {
            return false;
        };
        let Some(feature) = selection.features.get(feature_index) else {
            return false;
        };
        let Some(provider_package_id) = feature.external_provider_package_id(&selection.id) else {
            return false;
        };
        project_plugin_package_id_is_valid(provider_package_id)
            && self.provider_is_enabled_for_target(provider_package_id)
    }
}

#[cfg(test)]
#[path = "tests/projection.rs"]
mod tests;
