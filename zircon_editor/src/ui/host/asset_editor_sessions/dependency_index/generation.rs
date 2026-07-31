use std::collections::{BTreeMap, BTreeSet};

use crate::ui::host::project_access::normalize_ui_asset_asset_id;
use crate::ui::workbench::view::ViewInstanceId;

use super::UiAssetDependencyImpact;

/// Derived authoring projection for open UI documents. Runtime remains the
/// project-asset inventory authority; this generation only routes refreshes.
#[derive(Clone, Debug, Default)]
pub(crate) struct UiAssetDependencyGeneration {
    generation: u64,
    direct_by_asset_id: BTreeMap<String, BTreeSet<ViewInstanceId>>,
    importers_by_asset_id: BTreeMap<String, BTreeSet<ViewInstanceId>>,
    route_by_instance: BTreeMap<ViewInstanceId, String>,
    dependencies_by_instance: BTreeMap<ViewInstanceId, BTreeSet<String>>,
}

impl UiAssetDependencyGeneration {
    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }

    pub(crate) fn register_route(&mut self, instance_id: ViewInstanceId, asset_id: &str) -> bool {
        let asset_id = normalize_ui_asset_asset_id(asset_id).to_string();
        if self.route_by_instance.get(&instance_id) == Some(&asset_id) {
            return false;
        }
        if let Some(previous) = self
            .route_by_instance
            .insert(instance_id.clone(), asset_id.clone())
        {
            remove_reverse_edge(&mut self.direct_by_asset_id, &previous, &instance_id);
        }
        self.direct_by_asset_id
            .entry(asset_id)
            .or_default()
            .insert(instance_id);
        self.bump_generation();
        true
    }

    pub(crate) fn replace_dependencies<I, S>(
        &mut self,
        instance_id: ViewInstanceId,
        dependencies: I,
    ) -> bool
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let next = dependencies
            .into_iter()
            .map(|dependency| normalize_ui_asset_asset_id(dependency.as_ref()).to_string())
            .collect::<BTreeSet<_>>();
        if self.dependencies_by_instance.get(&instance_id) == Some(&next) {
            return false;
        }
        if let Some(previous) = self
            .dependencies_by_instance
            .insert(instance_id.clone(), next.clone())
        {
            for dependency in previous {
                remove_reverse_edge(&mut self.importers_by_asset_id, &dependency, &instance_id);
            }
        }
        for dependency in next {
            self.importers_by_asset_id
                .entry(dependency)
                .or_default()
                .insert(instance_id.clone());
        }
        self.bump_generation();
        true
    }

    pub(crate) fn remove(&mut self, instance_id: &ViewInstanceId) -> bool {
        let mut changed = false;
        if let Some(route) = self.route_by_instance.remove(instance_id) {
            remove_reverse_edge(&mut self.direct_by_asset_id, &route, instance_id);
            changed = true;
        }
        if let Some(dependencies) = self.dependencies_by_instance.remove(instance_id) {
            for dependency in dependencies {
                remove_reverse_edge(&mut self.importers_by_asset_id, &dependency, instance_id);
            }
            changed = true;
        }
        if changed {
            self.bump_generation();
        }
        changed
    }

    pub(crate) fn clear(&mut self) -> bool {
        if self.route_by_instance.is_empty() && self.dependencies_by_instance.is_empty() {
            return false;
        }
        self.direct_by_asset_id.clear();
        self.importers_by_asset_id.clear();
        self.route_by_instance.clear();
        self.dependencies_by_instance.clear();
        self.bump_generation();
        true
    }

    pub(crate) fn impact<I, S>(&self, changed_asset_ids: I) -> UiAssetDependencyImpact
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let changed_asset_ids = changed_asset_ids
            .into_iter()
            .map(|asset_id| normalize_ui_asset_asset_id(asset_id.as_ref()).to_string())
            .collect::<BTreeSet<_>>();
        let mut direct_instances = BTreeSet::new();
        let mut import_instances = BTreeSet::new();
        for asset_id in &changed_asset_ids {
            if let Some(instances) = self.direct_by_asset_id.get(asset_id) {
                direct_instances.extend(instances.iter().cloned());
            }
            if let Some(instances) = self.importers_by_asset_id.get(asset_id) {
                import_instances.extend(instances.iter().cloned());
            }
        }
        import_instances.retain(|instance_id| !direct_instances.contains(instance_id));
        UiAssetDependencyImpact {
            generation: self.generation,
            changed_asset_ids,
            direct_instances,
            import_instances,
        }
    }

    fn bump_generation(&mut self) {
        self.generation = self.generation.saturating_add(1);
    }
}

fn remove_reverse_edge(
    reverse: &mut BTreeMap<String, BTreeSet<ViewInstanceId>>,
    asset_id: &str,
    instance_id: &ViewInstanceId,
) {
    let remove_key = reverse.get_mut(asset_id).is_some_and(|instances| {
        instances.remove(instance_id);
        instances.is_empty()
    });
    if remove_key {
        reverse.remove(asset_id);
    }
}
