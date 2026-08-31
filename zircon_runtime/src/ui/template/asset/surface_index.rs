use std::collections::{BTreeMap, BTreeSet};

use crate::ui::surface::UiSurface;
use crate::ui::template::UiCompiledDocument;
use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiTreeId};
use zircon_runtime_interface::ui::template::{
    UiCompiledBindingHandle, UiCompiledBindingProgram, UiCompiledNodeId,
};
use zircon_runtime_interface::ui::tree::{UiDirtyFlags, UiTree, UiTreeError};

use super::hot_reload_plan::{UiAssetHotReloadPlan, UiAssetHotReloadSurfaceDirtyReport};

mod node_resource_registration;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiAssetSurfaceIndex {
    // Forward and reverse maps describe which retained runtime surfaces currently
    // own or reference each UI asset. The index is registration-based because
    // surface nodes do not yet persist asset-to-node ownership metadata.
    assets_by_surface: BTreeMap<UiTreeId, Vec<String>>,
    surfaces_by_asset: BTreeMap<String, BTreeSet<UiTreeId>>,
    compiled_assets_by_surface: BTreeMap<UiTreeId, Vec<String>>,
    compiled_nodes_by_surface: BTreeMap<UiTreeId, Vec<(String, UiCompiledNodeId)>>,
    compiled_nodes_by_asset: BTreeMap<String, BTreeSet<UiAssetCompiledNodeTarget>>,
    bindings_by_surface: BTreeMap<UiTreeId, Vec<(String, UiCompiledBindingHandle)>>,
    bindings_by_asset: BTreeMap<String, BTreeSet<UiAssetBindingTarget>>,
    node_assets_by_surface: BTreeMap<UiTreeId, BTreeMap<UiNodeId, Vec<String>>>,
    nodes_by_asset: BTreeMap<String, BTreeSet<UiAssetNodeTarget>>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UiAssetNodeTarget {
    pub tree_id: UiTreeId,
    pub node_id: UiNodeId,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UiAssetCompiledNodeTarget {
    pub tree_id: UiTreeId,
    pub node_id: UiCompiledNodeId,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UiAssetBindingTarget {
    pub tree_id: UiTreeId,
    pub handle: UiCompiledBindingHandle,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiAssetSurfaceHotReloadTargets {
    pub template_rebuild_surfaces: Vec<UiTreeId>,
    pub removed_compiled_surfaces: Vec<UiTreeId>,
    pub theme_restyle_surfaces: Vec<UiTreeId>,
    pub resource_damage_surfaces: Vec<UiTreeId>,
    pub dirty: UiDirtyFlags,
    pub rebuild_required: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiAssetNodeHotReloadTargets {
    pub template_rebuild_nodes: Vec<UiAssetNodeTarget>,
    pub removed_compiled_nodes: Vec<UiAssetNodeTarget>,
    pub theme_restyle_nodes: Vec<UiAssetNodeTarget>,
    pub resource_damage_nodes: Vec<UiAssetNodeTarget>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiAssetHotReloadNodeDirtyReport {
    pub nodes_marked: Vec<UiNodeId>,
    pub missing_nodes: Vec<UiNodeId>,
    pub dirty: UiDirtyFlags,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiAssetSurfaceNodeResourceRegistrationReport {
    pub tree_id: UiTreeId,
    pub nodes_registered: usize,
    pub resource_uris_registered: usize,
    pub nodes_without_resources: Vec<UiNodeId>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiAssetSurfaceHotReloadApplyReport {
    pub targets: UiAssetSurfaceHotReloadTargets,
    pub node_targets: UiAssetNodeHotReloadTargets,
    pub dirty_reports: BTreeMap<UiTreeId, UiAssetHotReloadSurfaceDirtyReport>,
    pub node_dirty_reports: BTreeMap<UiTreeId, UiAssetHotReloadNodeDirtyReport>,
    pub missing_surfaces: Vec<UiTreeId>,
}

impl UiAssetSurfaceIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_surface_assets<I, S>(&mut self, tree_id: UiTreeId, asset_ids: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.remove_surface_assets(&tree_id);

        let assets = dedupe_asset_ids(asset_ids);
        for asset in &assets {
            self.surfaces_by_asset
                .entry(asset.clone())
                .or_default()
                .insert(tree_id.clone());
        }
        self.assets_by_surface.insert(tree_id, assets);
    }

    pub fn record_node_assets<I, S>(&mut self, tree_id: UiTreeId, node_id: UiNodeId, asset_ids: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.remove_node_assets(&tree_id, node_id);

        let assets = dedupe_asset_ids(asset_ids);
        if assets.is_empty() {
            return;
        }

        let target = UiAssetNodeTarget {
            tree_id: tree_id.clone(),
            node_id,
        };
        for asset in &assets {
            self.nodes_by_asset
                .entry(asset.clone())
                .or_default()
                .insert(target.clone());
        }
        self.node_assets_by_surface
            .entry(tree_id)
            .or_default()
            .insert(node_id, assets);
    }

    pub fn record_compiled_surface(&mut self, tree_id: UiTreeId, compiled: &UiCompiledDocument) {
        let mut assets = Vec::new();
        assets.push(compiled.asset.id.as_str());
        for dependency in compiled.resource_dependencies() {
            assets.push(dependency.reference.uri.as_str());
            if let Some(fallback_uri) = dependency.reference.fallback.uri.as_deref() {
                assets.push(fallback_uri);
            }
        }
        self.record_surface_assets(tree_id.clone(), assets);
        self.record_binding_program(tree_id, compiled.template_instance().binding_program());
    }

    pub fn record_binding_program(
        &mut self,
        tree_id: UiTreeId,
        program: &UiCompiledBindingProgram,
    ) {
        self.remove_compiled_ownership(&tree_id);

        let mut assets = BTreeSet::new();
        if let Some(asset_id) = program.asset_id() {
            assets.insert(asset_id.to_string());
        }
        let mut node_owners = Vec::with_capacity(program.node_count());
        for (node_id, _) in program.iter_nodes() {
            let Some(asset_id) = program.node_asset_id(node_id) else {
                continue;
            };
            assets.insert(asset_id.to_string());
            let target = UiAssetCompiledNodeTarget {
                tree_id: tree_id.clone(),
                node_id,
            };
            self.compiled_nodes_by_asset
                .entry(asset_id.to_string())
                .or_default()
                .insert(target);
            node_owners.push((asset_id.to_string(), node_id));
        }

        let mut binding_owners = Vec::with_capacity(program.binding_count());
        for binding in program.iter_bindings() {
            let Some(asset_id) = program.binding_asset_id(binding.handle) else {
                continue;
            };
            assets.insert(asset_id.to_string());
            let target = UiAssetBindingTarget {
                tree_id: tree_id.clone(),
                handle: binding.handle,
            };
            self.bindings_by_asset
                .entry(asset_id.to_string())
                .or_default()
                .insert(target);
            binding_owners.push((asset_id.to_string(), binding.handle));
        }

        let assets = assets.into_iter().collect::<Vec<_>>();
        for asset_id in &assets {
            self.surfaces_by_asset
                .entry(asset_id.clone())
                .or_default()
                .insert(tree_id.clone());
        }
        self.compiled_assets_by_surface
            .insert(tree_id.clone(), assets);
        self.compiled_nodes_by_surface
            .insert(tree_id.clone(), node_owners);
        self.bindings_by_surface.insert(tree_id, binding_owners);
    }

    pub fn remove_surface(&mut self, tree_id: &UiTreeId) -> Option<Vec<String>> {
        self.remove_surface_node_assets(tree_id);
        let assets = self.remove_surface_assets(tree_id);
        self.remove_compiled_ownership(tree_id);
        assets
    }

    fn remove_surface_assets(&mut self, tree_id: &UiTreeId) -> Option<Vec<String>> {
        let assets = self.assets_by_surface.remove(tree_id)?;
        for asset in &assets {
            let compiled_owns_asset = self
                .compiled_assets_by_surface
                .get(tree_id)
                .is_some_and(|assets| assets.contains(asset));
            let remove_asset = if compiled_owns_asset {
                false
            } else if let Some(surfaces) = self.surfaces_by_asset.get_mut(asset.as_str()) {
                surfaces.remove(tree_id);
                surfaces.is_empty()
            } else {
                false
            };
            if remove_asset {
                self.surfaces_by_asset.remove(asset.as_str());
            }
        }
        Some(assets)
    }

    fn remove_compiled_ownership(&mut self, tree_id: &UiTreeId) -> Option<Vec<String>> {
        if let Some(nodes) = self.compiled_nodes_by_surface.remove(tree_id) {
            for (asset_id, node_id) in nodes {
                let target = UiAssetCompiledNodeTarget {
                    tree_id: tree_id.clone(),
                    node_id,
                };
                let remove_asset = self
                    .compiled_nodes_by_asset
                    .get_mut(asset_id.as_str())
                    .is_some_and(|targets| {
                        targets.remove(&target);
                        targets.is_empty()
                    });
                if remove_asset {
                    self.compiled_nodes_by_asset.remove(asset_id.as_str());
                }
            }
        }
        if let Some(bindings) = self.bindings_by_surface.remove(tree_id) {
            for (asset_id, handle) in bindings {
                let target = UiAssetBindingTarget {
                    tree_id: tree_id.clone(),
                    handle,
                };
                let remove_asset = self
                    .bindings_by_asset
                    .get_mut(asset_id.as_str())
                    .is_some_and(|targets| {
                        targets.remove(&target);
                        targets.is_empty()
                    });
                if remove_asset {
                    self.bindings_by_asset.remove(asset_id.as_str());
                }
            }
        }

        let assets = self.compiled_assets_by_surface.remove(tree_id)?;
        for asset_id in &assets {
            let registered_owns_asset = self
                .assets_by_surface
                .get(tree_id)
                .is_some_and(|assets| assets.contains(asset_id));
            let remove_asset = if registered_owns_asset {
                false
            } else if let Some(surfaces) = self.surfaces_by_asset.get_mut(asset_id.as_str()) {
                surfaces.remove(tree_id);
                surfaces.is_empty()
            } else {
                false
            };
            if remove_asset {
                self.surfaces_by_asset.remove(asset_id.as_str());
            }
        }
        Some(assets)
    }

    pub fn remove_node_assets(
        &mut self,
        tree_id: &UiTreeId,
        node_id: UiNodeId,
    ) -> Option<Vec<String>> {
        let assets = {
            let nodes = self.node_assets_by_surface.get_mut(tree_id)?;
            let assets = nodes.remove(&node_id)?;
            if nodes.is_empty() {
                self.node_assets_by_surface.remove(tree_id);
            }
            assets
        };

        let target = UiAssetNodeTarget {
            tree_id: tree_id.clone(),
            node_id,
        };
        for asset in &assets {
            let remove_asset = if let Some(nodes) = self.nodes_by_asset.get_mut(asset.as_str()) {
                nodes.remove(&target);
                nodes.is_empty()
            } else {
                false
            };
            if remove_asset {
                self.nodes_by_asset.remove(asset.as_str());
            }
        }
        Some(assets)
    }

    pub fn assets_for_surface(&self, tree_id: &UiTreeId) -> &[String] {
        self.assets_by_surface
            .get(tree_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn compiled_assets_for_surface(&self, tree_id: &UiTreeId) -> &[String] {
        self.compiled_assets_by_surface
            .get(tree_id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn assets_for_node(&self, tree_id: &UiTreeId, node_id: UiNodeId) -> &[String] {
        self.node_assets_by_surface
            .get(tree_id)
            .and_then(|nodes| nodes.get(&node_id))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn surfaces_for_asset<'a>(&'a self, asset_id: &str) -> impl Iterator<Item = &'a UiTreeId> {
        self.surfaces_by_asset
            .get(asset_id)
            .into_iter()
            .flat_map(|surfaces| surfaces.iter())
    }

    pub fn nodes_for_asset<'a>(
        &'a self,
        asset_id: &str,
    ) -> impl Iterator<Item = &'a UiAssetNodeTarget> {
        self.nodes_by_asset
            .get(asset_id)
            .into_iter()
            .flat_map(|nodes| nodes.iter())
    }

    pub fn compiled_nodes_for_asset<'a>(
        &'a self,
        asset_id: &str,
    ) -> impl Iterator<Item = &'a UiAssetCompiledNodeTarget> {
        self.compiled_nodes_by_asset
            .get(asset_id)
            .into_iter()
            .flat_map(|nodes| nodes.iter())
    }

    pub fn bindings_for_asset<'a>(
        &'a self,
        asset_id: &str,
    ) -> impl Iterator<Item = &'a UiAssetBindingTarget> {
        self.bindings_by_asset
            .get(asset_id)
            .into_iter()
            .flat_map(|bindings| bindings.iter())
    }

    pub fn target_surfaces_for_plan(
        &self,
        plan: &UiAssetHotReloadPlan,
    ) -> UiAssetSurfaceHotReloadTargets {
        let mut targets = UiAssetSurfaceHotReloadTargets {
            dirty: plan.dirty,
            rebuild_required: plan.rebuild_required,
            ..Default::default()
        };
        let mut seen_template_surfaces = BTreeSet::new();
        let mut seen_removed_surfaces = BTreeSet::new();
        let mut seen_theme_surfaces = BTreeSet::new();
        let mut seen_resource_surfaces = BTreeSet::new();

        self.collect_surfaces_for_assets(
            plan.template_rebuild_targets.iter().map(String::as_str),
            &mut targets.template_rebuild_surfaces,
            &mut seen_template_surfaces,
        );
        self.collect_surfaces_for_assets(
            plan.removed_compiled_assets.iter().map(String::as_str),
            &mut targets.removed_compiled_surfaces,
            &mut seen_removed_surfaces,
        );
        self.collect_surfaces_for_assets(
            plan.theme_restyle_assets
                .iter()
                .chain(plan.theme_restyle_targets.iter())
                .map(String::as_str),
            &mut targets.theme_restyle_surfaces,
            &mut seen_theme_surfaces,
        );
        self.collect_surfaces_for_assets(
            plan.resource_refresh_assets
                .iter()
                .chain(plan.resource_damage_targets.iter())
                .map(String::as_str),
            &mut targets.resource_damage_surfaces,
            &mut seen_resource_surfaces,
        );

        targets
    }

    pub fn target_nodes_for_plan(
        &self,
        plan: &UiAssetHotReloadPlan,
    ) -> UiAssetNodeHotReloadTargets {
        let mut targets = UiAssetNodeHotReloadTargets::default();
        let mut seen_template_nodes = BTreeSet::new();
        let mut seen_removed_nodes = BTreeSet::new();
        let mut seen_theme_nodes = BTreeSet::new();
        let mut seen_resource_nodes = BTreeSet::new();

        self.collect_nodes_for_assets(
            plan.template_rebuild_targets.iter().map(String::as_str),
            &mut targets.template_rebuild_nodes,
            &mut seen_template_nodes,
        );
        self.collect_nodes_for_assets(
            plan.removed_compiled_assets.iter().map(String::as_str),
            &mut targets.removed_compiled_nodes,
            &mut seen_removed_nodes,
        );
        self.collect_nodes_for_assets(
            plan.theme_restyle_assets
                .iter()
                .chain(plan.theme_restyle_targets.iter())
                .map(String::as_str),
            &mut targets.theme_restyle_nodes,
            &mut seen_theme_nodes,
        );
        self.collect_nodes_for_assets(
            plan.resource_refresh_assets
                .iter()
                .chain(plan.resource_damage_targets.iter())
                .map(String::as_str),
            &mut targets.resource_damage_nodes,
            &mut seen_resource_nodes,
        );

        targets
    }

    pub fn mark_target_surfaces_dirty(
        &self,
        plan: &UiAssetHotReloadPlan,
        surfaces: &mut BTreeMap<UiTreeId, UiSurface>,
    ) -> Result<UiAssetSurfaceHotReloadApplyReport, UiTreeError> {
        let targets = self.target_surfaces_for_plan(plan);
        let node_targets = self.target_nodes_for_plan(plan);
        let mut dirty_reports = BTreeMap::new();
        let mut node_dirty_reports = BTreeMap::new();
        let mut missing_surfaces = Vec::new();

        for tree_id in targets.all_target_surfaces() {
            let Some(surface) = surfaces.get_mut(&tree_id) else {
                missing_surfaces.push(tree_id);
                continue;
            };

            if !plan.rebuild_required {
                let node_target_report = self.node_dirty_report_for_surface(
                    &tree_id,
                    &node_targets,
                    plan.dirty,
                    surface,
                )?;
                if let Some(report) = node_target_report {
                    node_dirty_reports.insert(tree_id, report);
                    continue;
                }
            }

            let root_report = plan.mark_surface_roots_dirty(surface)?;
            if root_report.roots_marked > 0 || root_report.dirty.any() {
                dirty_reports.insert(tree_id, root_report);
            }
        }

        Ok(UiAssetSurfaceHotReloadApplyReport {
            targets,
            node_targets,
            dirty_reports,
            node_dirty_reports,
            missing_surfaces,
        })
    }

    pub fn surface_count(&self) -> usize {
        self.assets_by_surface.len()
            + self
                .compiled_assets_by_surface
                .keys()
                .filter(|tree_id| !self.assets_by_surface.contains_key(*tree_id))
                .count()
    }

    pub fn asset_count(&self) -> usize {
        self.surfaces_by_asset.len()
    }

    pub fn node_asset_count(&self) -> usize {
        self.nodes_by_asset.len()
    }

    pub fn compiled_node_asset_count(&self) -> usize {
        self.compiled_nodes_by_asset.len()
    }

    pub fn binding_asset_count(&self) -> usize {
        self.bindings_by_asset.len()
    }

    pub fn is_empty(&self) -> bool {
        self.assets_by_surface.is_empty()
            && self.compiled_assets_by_surface.is_empty()
            && self.node_assets_by_surface.is_empty()
            && self.compiled_nodes_by_surface.is_empty()
            && self.bindings_by_surface.is_empty()
    }

    fn collect_surfaces_for_assets<'a>(
        &'a self,
        asset_ids: impl IntoIterator<Item = &'a str>,
        output: &mut Vec<UiTreeId>,
        seen: &mut BTreeSet<&'a UiTreeId>,
    ) {
        for asset_id in asset_ids {
            for surface in self.surfaces_for_asset(asset_id) {
                if seen.insert(surface) {
                    output.push(surface.clone());
                }
            }
        }
    }

    fn collect_nodes_for_assets<'a>(
        &'a self,
        asset_ids: impl IntoIterator<Item = &'a str>,
        output: &mut Vec<UiAssetNodeTarget>,
        seen: &mut BTreeSet<&'a UiAssetNodeTarget>,
    ) {
        for asset_id in asset_ids {
            for target in self.nodes_for_asset(asset_id) {
                if seen.insert(target) {
                    output.push(target.clone());
                }
            }
        }
    }

    fn node_dirty_report_for_surface(
        &self,
        tree_id: &UiTreeId,
        targets: &UiAssetNodeHotReloadTargets,
        dirty: UiDirtyFlags,
        surface: &mut UiSurface,
    ) -> Result<Option<UiAssetHotReloadNodeDirtyReport>, UiTreeError> {
        let mut target_nodes = Vec::new();
        let mut seen = BTreeSet::new();
        let mut expected_target_count = push_nodes_for_surface(
            &mut target_nodes,
            &mut seen,
            tree_id,
            &targets.theme_restyle_nodes,
        );
        expected_target_count += push_nodes_for_surface(
            &mut target_nodes,
            &mut seen,
            tree_id,
            &targets.resource_damage_nodes,
        );

        if target_nodes.is_empty() {
            return Ok(None);
        }

        if target_nodes.len() != expected_target_count {
            return Ok(None);
        }

        let mut nodes_marked = Vec::new();
        let mut missing_nodes = Vec::new();
        for node_id in target_nodes {
            match surface.mark_node_dirty(node_id, dirty) {
                Ok(()) => nodes_marked.push(node_id),
                Err(UiTreeError::MissingNode(missing)) => missing_nodes.push(missing),
                Err(error) => return Err(error),
            }
        }

        if nodes_marked.is_empty() && missing_nodes.is_empty() {
            return Ok(None);
        }

        Ok(Some(UiAssetHotReloadNodeDirtyReport {
            nodes_marked,
            missing_nodes,
            dirty,
        }))
    }

    fn remove_surface_node_assets(
        &mut self,
        tree_id: &UiTreeId,
    ) -> Option<BTreeMap<UiNodeId, Vec<String>>> {
        let node_assets = self.node_assets_by_surface.remove(tree_id)?;
        for (node_id, assets) in &node_assets {
            let target = UiAssetNodeTarget {
                tree_id: tree_id.clone(),
                node_id: *node_id,
            };
            for asset in assets {
                let remove_asset = if let Some(nodes) = self.nodes_by_asset.get_mut(asset.as_str())
                {
                    nodes.remove(&target);
                    nodes.is_empty()
                } else {
                    false
                };
                if remove_asset {
                    self.nodes_by_asset.remove(asset.as_str());
                }
            }
        }
        Some(node_assets)
    }
}

impl UiAssetSurfaceHotReloadTargets {
    pub fn is_empty(&self) -> bool {
        self.template_rebuild_surfaces.is_empty()
            && self.removed_compiled_surfaces.is_empty()
            && self.theme_restyle_surfaces.is_empty()
            && self.resource_damage_surfaces.is_empty()
            && !self.dirty.any()
            && !self.rebuild_required
    }

    pub fn all_target_surfaces(&self) -> Vec<UiTreeId> {
        let mut targets = Vec::new();
        let mut seen = BTreeSet::new();
        push_unique_surfaces(&mut targets, &mut seen, &self.template_rebuild_surfaces);
        push_unique_surfaces(&mut targets, &mut seen, &self.removed_compiled_surfaces);
        push_unique_surfaces(&mut targets, &mut seen, &self.theme_restyle_surfaces);
        push_unique_surfaces(&mut targets, &mut seen, &self.resource_damage_surfaces);
        targets
    }
}

impl UiAssetNodeHotReloadTargets {
    pub fn is_empty(&self) -> bool {
        self.template_rebuild_nodes.is_empty()
            && self.removed_compiled_nodes.is_empty()
            && self.theme_restyle_nodes.is_empty()
            && self.resource_damage_nodes.is_empty()
    }

    pub fn all_target_nodes(&self) -> Vec<UiAssetNodeTarget> {
        let mut targets = Vec::new();
        let mut seen = BTreeSet::new();
        push_unique_nodes(&mut targets, &mut seen, &self.template_rebuild_nodes);
        push_unique_nodes(&mut targets, &mut seen, &self.removed_compiled_nodes);
        push_unique_nodes(&mut targets, &mut seen, &self.theme_restyle_nodes);
        push_unique_nodes(&mut targets, &mut seen, &self.resource_damage_nodes);
        targets
    }
}

fn dedupe_asset_ids<I, S>(asset_ids: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut seen = BTreeSet::new();
    let mut assets = Vec::new();
    for asset_id in asset_ids {
        let asset_id = asset_id.as_ref().trim();
        if !asset_id.is_empty() && seen.insert(asset_id.to_string()) {
            assets.push(asset_id.to_string());
        }
    }
    assets
}

fn push_unique_surfaces<'a>(
    targets: &mut Vec<UiTreeId>,
    seen: &mut BTreeSet<&'a UiTreeId>,
    surfaces: &'a [UiTreeId],
) {
    for surface in surfaces {
        if seen.insert(surface) {
            targets.push(surface.clone());
        }
    }
}

fn push_nodes_for_surface(
    targets: &mut Vec<UiNodeId>,
    seen: &mut BTreeSet<UiNodeId>,
    tree_id: &UiTreeId,
    nodes: &[UiAssetNodeTarget],
) -> usize {
    let mut matched = 0;
    for target in nodes {
        if &target.tree_id == tree_id {
            matched += 1;
            if seen.insert(target.node_id) {
                targets.push(target.node_id);
            }
        }
    }
    matched
}

fn push_unique_nodes<'a>(
    targets: &mut Vec<UiAssetNodeTarget>,
    seen: &mut BTreeSet<&'a UiAssetNodeTarget>,
    nodes: &'a [UiAssetNodeTarget],
) {
    for node in nodes {
        if seen.insert(node) {
            targets.push(node.clone());
        }
    }
}
