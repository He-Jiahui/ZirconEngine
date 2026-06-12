use std::collections::{BTreeMap, BTreeSet};

use crate::ui::surface::UiSurface;
use crate::ui::template::UiCompiledDocument;
use toml::Value;
use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiTreeId};
use zircon_runtime_interface::ui::template::{
    UiResourceFallbackMode, UiResourceFallbackPolicy, UiResourceKind,
};
use zircon_runtime_interface::ui::tree::{UiDirtyFlags, UiTree, UiTreeError};

use super::hot_reload_plan::{UiAssetHotReloadPlan, UiAssetHotReloadSurfaceDirtyReport};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiAssetSurfaceIndex {
    // Forward and reverse maps describe which retained runtime surfaces currently
    // own or reference each UI asset. The index is registration-based because
    // surface nodes do not yet persist asset-to-node ownership metadata.
    assets_by_surface: BTreeMap<UiTreeId, Vec<String>>,
    surfaces_by_asset: BTreeMap<String, BTreeSet<UiTreeId>>,
    node_assets_by_surface: BTreeMap<UiTreeId, BTreeMap<UiNodeId, Vec<String>>>,
    nodes_by_asset: BTreeMap<String, BTreeSet<UiAssetNodeTarget>>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UiAssetNodeTarget {
    pub tree_id: UiTreeId,
    pub node_id: UiNodeId,
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
        self.record_surface_assets(tree_id, assets);
    }

    pub fn record_tree_node_resources(
        &mut self,
        tree: &UiTree,
    ) -> UiAssetSurfaceNodeResourceRegistrationReport {
        let tree_id = tree.tree_id.clone();
        self.remove_surface_node_assets(&tree_id);
        let mut report = UiAssetSurfaceNodeResourceRegistrationReport {
            tree_id: tree_id.clone(),
            ..Default::default()
        };

        for (node_id, node) in &tree.nodes {
            let mut collector = NodeResourceCollector::default();
            if let Some(metadata) = &node.template_metadata {
                collector.collect_map(&metadata.attributes, "attributes");
                collector.collect_map(&metadata.slot_attributes, "slot_attributes");
                collector.collect_map(&metadata.style_overrides, "style_overrides");
            }

            let resources = collector.finish();
            if resources.is_empty() {
                report.nodes_without_resources.push(*node_id);
                continue;
            }

            report.nodes_registered += 1;
            report.resource_uris_registered += resources.len();
            self.record_node_assets(tree_id.clone(), *node_id, resources);
        }

        report
    }

    pub fn remove_surface(&mut self, tree_id: &UiTreeId) -> Option<Vec<String>> {
        self.remove_surface_node_assets(tree_id);
        self.remove_surface_assets(tree_id)
    }

    fn remove_surface_assets(&mut self, tree_id: &UiTreeId) -> Option<Vec<String>> {
        let assets = self.assets_by_surface.remove(tree_id)?;
        for asset in &assets {
            let remove_asset =
                if let Some(surfaces) = self.surfaces_by_asset.get_mut(asset.as_str()) {
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
    }

    pub fn asset_count(&self) -> usize {
        self.surfaces_by_asset.len()
    }

    pub fn node_asset_count(&self) -> usize {
        self.nodes_by_asset.len()
    }

    pub fn is_empty(&self) -> bool {
        self.assets_by_surface.is_empty() && self.node_assets_by_surface.is_empty()
    }

    fn collect_surfaces_for_assets<'a>(
        &'a self,
        asset_ids: impl IntoIterator<Item = &'a str>,
        output: &mut Vec<UiTreeId>,
        seen: &mut BTreeSet<UiTreeId>,
    ) {
        for asset_id in asset_ids {
            for surface in self.surfaces_for_asset(asset_id) {
                if seen.insert(surface.clone()) {
                    output.push(surface.clone());
                }
            }
        }
    }

    fn collect_nodes_for_assets<'a>(
        &'a self,
        asset_ids: impl IntoIterator<Item = &'a str>,
        output: &mut Vec<UiAssetNodeTarget>,
        seen: &mut BTreeSet<UiAssetNodeTarget>,
    ) {
        for asset_id in asset_ids {
            for target in self.nodes_for_asset(asset_id) {
                if seen.insert(target.clone()) {
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
        push_nodes_for_surface(
            &mut target_nodes,
            &mut seen,
            tree_id,
            &targets.theme_restyle_nodes,
        );
        push_nodes_for_surface(
            &mut target_nodes,
            &mut seen,
            tree_id,
            &targets.resource_damage_nodes,
        );

        if target_nodes.is_empty() {
            return Ok(None);
        }

        let expected_target_count = target_surface_count(tree_id, &targets.theme_restyle_nodes)
            + target_surface_count(tree_id, &targets.resource_damage_nodes);
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

fn push_unique_surfaces(
    targets: &mut Vec<UiTreeId>,
    seen: &mut BTreeSet<UiTreeId>,
    surfaces: &[UiTreeId],
) {
    for surface in surfaces {
        if seen.insert(surface.clone()) {
            targets.push(surface.clone());
        }
    }
}

fn push_nodes_for_surface(
    targets: &mut Vec<UiNodeId>,
    seen: &mut BTreeSet<UiNodeId>,
    tree_id: &UiTreeId,
    nodes: &[UiAssetNodeTarget],
) {
    for target in nodes {
        if &target.tree_id == tree_id && seen.insert(target.node_id) {
            targets.push(target.node_id);
        }
    }
}

fn target_surface_count(tree_id: &UiTreeId, nodes: &[UiAssetNodeTarget]) -> usize {
    nodes
        .iter()
        .filter(|target| &target.tree_id == tree_id)
        .count()
}

fn push_unique_nodes(
    targets: &mut Vec<UiAssetNodeTarget>,
    seen: &mut BTreeSet<UiAssetNodeTarget>,
    nodes: &[UiAssetNodeTarget],
) {
    for node in nodes {
        if seen.insert(node.clone()) {
            targets.push(node.clone());
        }
    }
}

#[derive(Default)]
struct NodeResourceCollector {
    uris: Vec<String>,
    seen: BTreeSet<String>,
}

impl NodeResourceCollector {
    fn collect_map(&mut self, values: &BTreeMap<String, Value>, root: &str) {
        for (key, value) in values {
            self.collect_value(value, &format!("{root}.{key}"));
        }
    }

    fn collect_value(&mut self, value: &Value, path: &str) {
        match value {
            Value::String(uri) if has_supported_resource_scheme(uri) => {
                let kind = UiResourceKind::infer_from_path_and_uri(path, uri);
                self.push_resource_uri(kind, uri, &UiResourceFallbackPolicy::default());
            }
            Value::Array(values) => {
                for (index, value) in values.iter().enumerate() {
                    self.collect_value(value, &format!("{path}[{index}]"));
                }
            }
            Value::Table(table) if is_resource_table(table) => {
                self.collect_resource_table(table, path);
            }
            Value::Table(table) => {
                for (key, value) in table {
                    self.collect_value(value, &format!("{path}.{key}"));
                }
            }
            _ => {}
        }
    }

    fn collect_resource_table(&mut self, table: &toml::map::Map<String, Value>, path: &str) {
        let Some(Value::String(uri)) = table.get("uri") else {
            return;
        };
        if !has_supported_resource_scheme(uri) {
            return;
        }
        let kind = table
            .get("kind")
            .and_then(Value::as_str)
            .and_then(resource_kind_from_name)
            .unwrap_or_else(|| UiResourceKind::infer_from_path_and_uri(path, uri));
        let fallback = fallback_policy_from_table(table);
        self.push_resource_uri(kind, uri, &fallback);
    }

    fn push_resource_uri(
        &mut self,
        kind: UiResourceKind,
        uri: &str,
        fallback: &UiResourceFallbackPolicy,
    ) {
        let trimmed = uri.trim();
        if !trimmed.is_empty() && self.seen.insert(trimmed.to_string()) {
            self.uris.push(trimmed.to_string());
        }
        if let Some(fallback_uri) = fallback.uri.as_deref() {
            let fallback_uri = fallback_uri.trim();
            if !fallback_uri.is_empty()
                && has_supported_resource_scheme(fallback_uri)
                && (fallback.mode != UiResourceFallbackMode::Placeholder
                    || UiResourceKind::infer_from_path_and_uri("", fallback_uri) == kind)
                && self.seen.insert(fallback_uri.to_string())
            {
                self.uris.push(fallback_uri.to_string());
            }
        }
    }

    fn finish(self) -> Vec<String> {
        self.uris
    }
}

fn is_resource_table(table: &toml::map::Map<String, Value>) -> bool {
    table.contains_key("uri")
        || table.get("kind").is_some_and(
            |kind| matches!(kind, Value::String(kind) if resource_kind_from_name(kind).is_some()),
        )
        || matches!(table.get("fallback"), Some(Value::Table(_)))
}

fn fallback_policy_from_table(table: &toml::map::Map<String, Value>) -> UiResourceFallbackPolicy {
    let Some(Value::Table(fallback)) = table.get("fallback") else {
        return UiResourceFallbackPolicy::default();
    };

    let mode = fallback
        .get("mode")
        .and_then(Value::as_str)
        .map(fallback_mode_from_name)
        .unwrap_or_default();
    let uri = fallback
        .get("uri")
        .and_then(Value::as_str)
        .map(str::to_string);
    UiResourceFallbackPolicy { mode, uri }
}

fn fallback_mode_from_name(value: &str) -> UiResourceFallbackMode {
    match value {
        "placeholder" => UiResourceFallbackMode::Placeholder,
        "optional" => UiResourceFallbackMode::Optional,
        _ => UiResourceFallbackMode::None,
    }
}

fn resource_kind_from_name(value: &str) -> Option<UiResourceKind> {
    match value {
        "font" => Some(UiResourceKind::Font),
        "image" => Some(UiResourceKind::Image),
        "media" => Some(UiResourceKind::Media),
        "generic_asset" => Some(UiResourceKind::GenericAsset),
        _ => None,
    }
}

fn has_supported_resource_scheme(uri: &str) -> bool {
    uri.starts_with("res://") || uri.starts_with("asset://") || uri.starts_with("project://")
}
