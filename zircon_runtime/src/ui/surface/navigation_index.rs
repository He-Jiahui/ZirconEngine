use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
};

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    navigation::{
        UiDirectionalNavigation, UiDirectionalNavigationTarget, UiNavigationGroup,
        UiNavigationGroupId, UiTabIndex,
    },
    surface::{UiArrangedTree, UiHitTestGrid, UiNavigationEventKind},
    tree::{UiTemplateNodeMetadata, UiTree, UiTreeError},
    widget::UiWidgetBehavior,
};

use super::{arranged_node_indexed, frame_hit_test::UiProjectedHitTestIndex, surface::UiSurface};

#[cfg(feature = "profiling")]
mod profile;
#[cfg(feature = "profiling")]
use profile::record_navigation_rebuild_profile;
mod geometry_patch;
mod semantics;

#[derive(Clone, Debug, Default)]
pub(super) struct UiSurfaceNavigationIndex {
    initialized: bool,
    build_generation: u64,
    build_error: Option<UiTreeError>,
    nodes: BTreeMap<UiNodeId, UiNavigationIndexNode>,
    geometry_authority_node_ids: BTreeSet<UiNodeId>,
    referenced_modal_root_node_ids: BTreeSet<UiNodeId>,
    tab_base: Vec<UiNodeId>,
    tab_groups: BTreeMap<UiNavigationGroupId, Vec<UiNodeId>>,
    tab_mui_roots: BTreeMap<UiNodeId, Vec<UiNodeId>>,
    tab_base_positions: BTreeMap<UiNodeId, usize>,
    tab_group_positions: BTreeMap<UiNavigationGroupId, BTreeMap<UiNodeId, usize>>,
    tab_mui_root_positions: BTreeMap<UiNodeId, BTreeMap<UiNodeId, usize>>,
    spatial_all: Vec<UiNodeId>,
    spatial_groups: BTreeMap<UiNavigationGroupId, Vec<UiNodeId>>,
    spatial_mui_roots: BTreeMap<UiNodeId, Vec<UiNodeId>>,
    first_candidate_by_group: BTreeMap<UiNavigationGroupId, UiNodeId>,
    declared_modal_scope: Option<UiRankedNavigationScope>,
}

// The index is a rebuild-owned cache and does not change UiSurface value equality.
impl PartialEq for UiSurfaceNavigationIndex {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[derive(Clone, Debug)]
struct UiNavigationIndexNode {
    frame: UiFrame,
    z_index: i32,
    paint_order: u64,
    focus_candidate: bool,
    tab_order: i32,
    tabbable: bool,
    group_order: i32,
    group_id: Option<UiNavigationGroupId>,
    modal_group_id: Option<UiNavigationGroupId>,
    mui_modal_root: Option<UiNodeId>,
    local_modal_scope: Option<UiRankedNavigationScope>,
    directional: Option<UiDirectionalNavigation>,
    subtree_navigation_authority: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct UiRankedNavigationScope {
    rank: (i32, u64, UiNodeId),
    scope: UiNavigationScope,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum UiNavigationScope {
    Group { group_id: UiNavigationGroupId },
    MuiOverlay(UiNodeId),
}

#[derive(Clone, Copy)]
struct UiNavigationGeometry {
    frame: UiFrame,
    z_index: i32,
    paint_order: u64,
}

impl UiSurfaceNavigationIndex {
    pub(super) fn rebuild(
        &mut self,
        tree: &UiTree,
        arranged_tree: &UiArrangedTree,
        arranged_node_indices: &BTreeMap<UiNodeId, usize>,
        projected_hit_test: &UiProjectedHitTestIndex,
        base_hit_grid: &UiHitTestGrid,
    ) {
        #[cfg(feature = "profiling")]
        let rebuild_start = std::time::Instant::now();
        self.clear_for_rebuild();
        self.build_generation = self.build_generation.saturating_add(1);

        for root_id in &tree.roots {
            if let Err(error) = self.collect_node(
                tree,
                arranged_tree,
                arranged_node_indices,
                projected_hit_test,
                base_hit_grid,
                *root_id,
                None,
                None,
                None,
            ) {
                self.build_error = Some(error);
                self.initialized = true;
                #[cfg(feature = "profiling")]
                record_navigation_rebuild_profile(self, rebuild_start);
                return;
            }
        }

        self.finish_lists();
        self.initialized = true;
        #[cfg(feature = "profiling")]
        record_navigation_rebuild_profile(self, rebuild_start);
    }

    pub(super) fn next_navigation_target(
        &self,
        current: Option<UiNodeId>,
        kind: UiNavigationEventKind,
    ) -> Result<Option<UiNodeId>, UiTreeError> {
        if let Some(error) = &self.build_error {
            return Err(error.clone());
        }
        if !self.initialized {
            return Ok(None);
        }
        #[cfg(feature = "profiling")]
        crate::core::diagnostics::profiling::record_counter(
            "runtime",
            "ui.navigation_index.query_count",
            1.0,
        );
        match kind {
            UiNavigationEventKind::Next | UiNavigationEventKind::Previous => {
                Ok(self.next_tab_target(current, kind))
            }
            UiNavigationEventKind::Up
            | UiNavigationEventKind::Down
            | UiNavigationEventKind::Left
            | UiNavigationEventKind::Right => self.next_directional_target(current, kind),
            UiNavigationEventKind::Activate
            | UiNavigationEventKind::Cancel
            | UiNavigationEventKind::Home
            | UiNavigationEventKind::End => Ok(None),
        }
    }

    pub(super) fn has_navigation_candidate(&self) -> bool {
        self.initialized && self.build_error.is_none() && !self.spatial_all.is_empty()
    }

    #[cfg(test)]
    pub(super) const fn build_generation(&self) -> u64 {
        self.build_generation
    }

    fn clear_for_rebuild(&mut self) {
        self.build_error = None;
        self.nodes.clear();
        self.geometry_authority_node_ids.clear();
        self.referenced_modal_root_node_ids.clear();
        self.tab_base.clear();
        self.tab_groups.clear();
        self.tab_mui_roots.clear();
        self.tab_base_positions.clear();
        self.tab_group_positions.clear();
        self.tab_mui_root_positions.clear();
        self.spatial_all.clear();
        self.spatial_groups.clear();
        self.spatial_mui_roots.clear();
        self.first_candidate_by_group.clear();
        self.declared_modal_scope = None;
    }

    #[allow(clippy::too_many_arguments)]
    fn collect_node<'a>(
        &mut self,
        tree: &'a UiTree,
        arranged_tree: &UiArrangedTree,
        arranged_node_indices: &BTreeMap<UiNodeId, usize>,
        projected_hit_test: &UiProjectedHitTestIndex,
        base_hit_grid: &UiHitTestGrid,
        node_id: UiNodeId,
        inherited_group: Option<&'a UiNavigationGroup>,
        inherited_modal_group: Option<(UiNodeId, &'a UiNavigationGroup)>,
        inherited_mui_modal_root: Option<UiNodeId>,
    ) -> Result<bool, UiTreeError> {
        let node = tree
            .nodes
            .get(&node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        let geometry = navigation_geometry(
            node_id,
            node.layout_cache.frame,
            node.z_index,
            node.paint_order,
            arranged_tree,
            arranged_node_indices,
            projected_hit_test,
            base_hit_grid,
        );
        let group = node.navigation.group.as_ref().or(inherited_group);
        let modal_group = node
            .navigation
            .group
            .as_ref()
            .filter(|group| group.modal)
            .map(|group| (node_id, group))
            .or(inherited_modal_group);
        let mui_modal_root = is_active_mui_modal_focus_scope(node)
            .then_some(node_id)
            .or(inherited_mui_modal_root);
        if node.is_focus_candidate() || is_active_mui_modal_focus_scope(node) {
            self.geometry_authority_node_ids.insert(node_id);
        }
        if let Some(group_root) = node
            .navigation
            .group
            .as_ref()
            .filter(|group| group.modal)
            .map(|group| group.root.unwrap_or(node_id))
        {
            self.geometry_authority_node_ids.insert(group_root);
            self.referenced_modal_root_node_ids.insert(group_root);
        }

        let mut local_modal_scope = modal_group.and_then(|(owner, group)| {
            active_modal_navigation_group_scope(
                tree,
                arranged_tree,
                arranged_node_indices,
                projected_hit_test,
                base_hit_grid,
                owner,
                group,
            )
        });
        if let Some(root) = mui_modal_root {
            retain_topmost_scope(
                &mut local_modal_scope,
                ranked_mui_scope(
                    tree,
                    arranged_tree,
                    arranged_node_indices,
                    projected_hit_test,
                    base_hit_grid,
                    root,
                ),
            );
        }

        if let Some(scope) = declared_modal_scope_for_node(
            tree,
            arranged_tree,
            arranged_node_indices,
            projected_hit_test,
            base_hit_grid,
            node_id,
        ) {
            retain_topmost_scope(&mut self.declared_modal_scope, Some(scope));
        }

        let tab_index = node.navigation.tab_index.unwrap_or_else(|| UiTabIndex {
            order: geometry.paint_order.min(i32::MAX as u64) as i32,
            tabbable: node.is_focus_candidate(),
        });
        self.nodes.insert(
            node_id,
            UiNavigationIndexNode {
                frame: geometry.frame,
                z_index: geometry.z_index,
                paint_order: geometry.paint_order,
                focus_candidate: node.is_focus_candidate(),
                tab_order: tab_index.order,
                tabbable: tab_index.tabbable,
                group_order: group.map_or(0, |group| group.order),
                group_id: group.map(|group| group.group_id.clone()),
                modal_group_id: modal_group.map(|(_, group)| group.group_id.clone()),
                mui_modal_root,
                local_modal_scope,
                directional: node.navigation.directional.clone(),
                subtree_navigation_authority: false,
            },
        );

        let mut subtree_navigation_authority = node.is_focus_candidate()
            || node.navigation.tab_index.is_some()
            || node.navigation.group.is_some()
            || node.navigation.directional.is_some()
            || is_active_mui_modal_focus_scope(node);
        for child_id in &node.children {
            subtree_navigation_authority |= self.collect_node(
                tree,
                arranged_tree,
                arranged_node_indices,
                projected_hit_test,
                base_hit_grid,
                *child_id,
                group,
                modal_group,
                mui_modal_root,
            )?;
        }
        self.nodes
            .get_mut(&node_id)
            .expect("collected navigation node must remain indexed")
            .subtree_navigation_authority = subtree_navigation_authority;
        Ok(subtree_navigation_authority)
    }

    fn finish_lists(&mut self) {
        let candidate_ids: Vec<_> = self
            .nodes
            .iter()
            .filter_map(|(node_id, node)| node.focus_candidate.then_some(*node_id))
            .collect();
        for node_id in candidate_ids {
            let node = self
                .nodes
                .get(&node_id)
                .expect("candidate id must resolve in navigation index");
            self.spatial_all.push(node_id);
            if let Some(group_id) = &node.modal_group_id {
                self.spatial_groups
                    .entry(group_id.clone())
                    .or_default()
                    .push(node_id);
                if node.tabbable {
                    self.tab_groups
                        .entry(group_id.clone())
                        .or_default()
                        .push(node_id);
                }
            }
            if let Some(root) = node.mui_modal_root {
                self.spatial_mui_roots
                    .entry(root)
                    .or_default()
                    .push(node_id);
                if node.tabbable {
                    self.tab_mui_roots.entry(root).or_default().push(node_id);
                }
            }
            if node.modal_group_id.is_none() && node.mui_modal_root.is_none() && node.tabbable {
                self.tab_base.push(node_id);
            }
        }

        let nodes = &self.nodes;
        self.spatial_all
            .sort_by(|left, right| compare_spatial_nodes(nodes, *left, *right));
        for candidates in self.spatial_groups.values_mut() {
            candidates.sort_by(|left, right| compare_spatial_nodes(nodes, *left, *right));
        }
        for candidates in self.spatial_mui_roots.values_mut() {
            candidates.sort_by(|left, right| compare_spatial_nodes(nodes, *left, *right));
        }
        self.tab_base
            .sort_by(|left, right| compare_tab_nodes(nodes, *left, *right));
        for candidates in self.tab_groups.values_mut() {
            candidates.sort_by(|left, right| compare_tab_nodes(nodes, *left, *right));
        }
        for candidates in self.tab_mui_roots.values_mut() {
            candidates.sort_by(|left, right| compare_tab_nodes(nodes, *left, *right));
        }

        self.tab_base_positions = position_map(&self.tab_base);
        self.tab_group_positions = self
            .tab_groups
            .iter()
            .map(|(group_id, candidates)| (group_id.clone(), position_map(candidates)))
            .collect();
        self.tab_mui_root_positions = self
            .tab_mui_roots
            .iter()
            .map(|(root, candidates)| (*root, position_map(candidates)))
            .collect();

        let mut group_candidates: BTreeMap<UiNavigationGroupId, Vec<UiNodeId>> = BTreeMap::new();
        for (node_id, node) in &self.nodes {
            if node.focus_candidate {
                if let Some(group_id) = &node.group_id {
                    group_candidates
                        .entry(group_id.clone())
                        .or_default()
                        .push(*node_id);
                }
            }
        }
        for (group_id, candidates) in &mut group_candidates {
            candidates.sort_by(|left, right| compare_tab_nodes(nodes, *left, *right));
            if let Some(first) = candidates.first() {
                self.first_candidate_by_group
                    .insert(group_id.clone(), *first);
            }
        }
    }

    fn next_tab_target(
        &self,
        current: Option<UiNodeId>,
        kind: UiNavigationEventKind,
    ) -> Option<UiNodeId> {
        let scope = self.active_modal_scope(current);
        let (candidates, current_index) = match scope.map(|scope| &scope.scope) {
            Some(UiNavigationScope::Group { group_id }) => {
                let candidates = self
                    .tab_groups
                    .get(group_id)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                let index = current.and_then(|node_id| {
                    self.tab_group_positions
                        .get(group_id)
                        .and_then(|positions| positions.get(&node_id))
                        .copied()
                });
                (candidates, index)
            }
            Some(UiNavigationScope::MuiOverlay(root)) => {
                let candidates = self
                    .tab_mui_roots
                    .get(root)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                let index = current.and_then(|node_id| {
                    self.tab_mui_root_positions
                        .get(root)
                        .and_then(|positions| positions.get(&node_id))
                        .copied()
                });
                (candidates, index)
            }
            None => (
                self.tab_base.as_slice(),
                current.and_then(|node_id| self.tab_base_positions.get(&node_id).copied()),
            ),
        };
        if candidates.is_empty() {
            return None;
        }
        let next_index = match (kind, current_index) {
            (UiNavigationEventKind::Next, Some(index)) => (index + 1) % candidates.len(),
            (UiNavigationEventKind::Previous, Some(0)) => candidates.len() - 1,
            (UiNavigationEventKind::Previous, Some(index)) => index - 1,
            (UiNavigationEventKind::Next, None) => 0,
            (UiNavigationEventKind::Previous, None) => candidates.len() - 1,
            _ => return None,
        };
        candidates.get(next_index).copied()
    }

    fn next_directional_target(
        &self,
        current: Option<UiNodeId>,
        kind: UiNavigationEventKind,
    ) -> Result<Option<UiNodeId>, UiTreeError> {
        let scope = self.active_modal_scope(current);
        if let Some(current) = current {
            let node = self
                .nodes
                .get(&current)
                .ok_or(UiTreeError::MissingNode(current))?;
            if let Some(target) = self.manual_direction_target(node, kind) {
                return Ok(self.filter_modal_target(target, scope));
            }
        }
        let candidates = self.spatial_candidates(scope);
        if candidates.is_empty() {
            return Ok(None);
        }
        let Some(current) = current else {
            return Ok(match kind {
                UiNavigationEventKind::Up | UiNavigationEventKind::Left => {
                    candidates.last().copied()
                }
                UiNavigationEventKind::Down | UiNavigationEventKind::Right => {
                    candidates.first().copied()
                }
                _ => None,
            });
        };
        self.nearest_candidate(current, kind, candidates)
    }

    fn manual_direction_target(
        &self,
        node: &UiNavigationIndexNode,
        kind: UiNavigationEventKind,
    ) -> Option<Option<UiNodeId>> {
        let directional = node.directional.as_ref()?;
        match target_for_direction(directional, kind) {
            UiDirectionalNavigationTarget::Auto => None,
            UiDirectionalNavigationTarget::Blocked => Some(None),
            UiDirectionalNavigationTarget::Node(node_id) => Some(
                self.nodes
                    .get(node_id)
                    .filter(|node| node.focus_candidate)
                    .map(|_| *node_id),
            ),
            UiDirectionalNavigationTarget::Group(group_id) => {
                Some(self.first_candidate_by_group.get(group_id).copied())
            }
        }
    }

    fn filter_modal_target(
        &self,
        target: Option<UiNodeId>,
        modal_scope: Option<&UiRankedNavigationScope>,
    ) -> Option<UiNodeId> {
        let Some(scope) = modal_scope.map(|scope| &scope.scope) else {
            return target;
        };
        target.filter(|target| {
            let Some(node) = self.nodes.get(target) else {
                return false;
            };
            match scope {
                UiNavigationScope::Group { group_id } => {
                    node.modal_group_id.as_ref() == Some(group_id)
                }
                UiNavigationScope::MuiOverlay(root) => node.mui_modal_root == Some(*root),
            }
        })
    }

    fn active_modal_scope(&self, current: Option<UiNodeId>) -> Option<&UiRankedNavigationScope> {
        let local = current
            .and_then(|current| self.nodes.get(&current))
            .and_then(|node| node.local_modal_scope.as_ref());
        match (local, self.declared_modal_scope.as_ref()) {
            (Some(local), Some(declared)) if local.rank >= declared.rank => Some(local),
            (Some(_), Some(declared)) => Some(declared),
            (Some(local), None) => Some(local),
            (None, declared) => declared,
        }
    }

    fn spatial_candidates(&self, scope: Option<&UiRankedNavigationScope>) -> &[UiNodeId] {
        match scope.map(|scope| &scope.scope) {
            Some(UiNavigationScope::Group { group_id }) => self
                .spatial_groups
                .get(group_id)
                .map(Vec::as_slice)
                .unwrap_or(&[]),
            Some(UiNavigationScope::MuiOverlay(root)) => self
                .spatial_mui_roots
                .get(root)
                .map(Vec::as_slice)
                .unwrap_or(&[]),
            None => &self.spatial_all,
        }
    }

    fn nearest_candidate(
        &self,
        current: UiNodeId,
        kind: UiNavigationEventKind,
        candidates: &[UiNodeId],
    ) -> Result<Option<UiNodeId>, UiTreeError> {
        const DIRECTION_EPSILON: f32 = 0.0001;

        let origin = self
            .nodes
            .get(&current)
            .ok_or(UiTreeError::MissingNode(current))?
            .frame
            .center();
        let mut best: Option<(UiNodeId, f32, f32)> = None;
        for candidate_id in candidates {
            if *candidate_id == current {
                continue;
            }
            let Some(candidate) = self.nodes.get(candidate_id) else {
                continue;
            };
            let center = candidate.frame.center();
            let delta_x = center.x - origin.x;
            let delta_y = center.y - origin.y;
            let (primary, lateral) = match kind {
                UiNavigationEventKind::Right => (delta_x, delta_y.abs()),
                UiNavigationEventKind::Left => (-delta_x, delta_y.abs()),
                UiNavigationEventKind::Down => (delta_y, delta_x.abs()),
                UiNavigationEventKind::Up => (-delta_y, delta_x.abs()),
                _ => continue,
            };
            if primary <= DIRECTION_EPSILON {
                continue;
            }
            let slope = lateral / primary;
            let distance_sq = delta_x * delta_x + delta_y * delta_y;
            if best
                .as_ref()
                .is_none_or(|(_, best_slope, best_distance_sq)| {
                    slope < *best_slope - DIRECTION_EPSILON
                        || ((slope - *best_slope).abs() <= DIRECTION_EPSILON
                            && distance_sq < *best_distance_sq)
                })
            {
                best = Some((*candidate_id, slope, distance_sq));
            }
        }
        Ok(best.map(|(node_id, _, _)| node_id))
    }
}

impl UiSurface {
    pub(super) fn rebuild_navigation_index(&mut self) {
        self.navigation_index.rebuild(
            &self.tree,
            &self.arranged_tree,
            &self.arranged_node_indices,
            &self.projected_hit_test,
            &self.hit_test.grid,
        );
    }

    pub(super) fn next_navigation_target(
        &self,
        current: Option<UiNodeId>,
        kind: UiNavigationEventKind,
    ) -> Result<Option<UiNodeId>, UiTreeError> {
        self.navigation_index.next_navigation_target(current, kind)
    }

    pub(super) fn navigation_index_patch_changed_geometry(
        &mut self,
        changed_node_ids: &BTreeSet<UiNodeId>,
        removed_node_ids: &BTreeSet<UiNodeId>,
    ) -> Result<usize, ()> {
        self.navigation_index.patch_changed_geometry(
            &self.tree,
            &self.arranged_tree,
            &self.arranged_node_indices,
            &self.projected_hit_test,
            &self.hit_test.grid,
            changed_node_ids,
            removed_node_ids,
        )
    }

    pub(super) fn navigation_index_needs_semantics_rebuild(
        &self,
        changed_node_ids: &BTreeSet<UiNodeId>,
        removed_node_ids: &BTreeSet<UiNodeId>,
    ) -> bool {
        self.navigation_index.needs_semantics_rebuild(
            &self.tree,
            &self.arranged_tree,
            &self.arranged_node_indices,
            &self.projected_hit_test,
            &self.hit_test.grid,
            changed_node_ids,
            removed_node_ids,
        )
    }

    pub(super) fn navigation_index_patch_projected_geometry(&mut self) -> Result<usize, ()> {
        self.navigation_index.patch_projected_geometry(
            &self.tree,
            &self.arranged_tree,
            &self.arranged_node_indices,
            &self.projected_hit_test,
            &self.hit_test.grid,
        )
    }

    #[cfg(test)]
    pub(crate) fn navigation_index_build_generation(&self) -> u64 {
        self.navigation_index.build_generation()
    }
}

#[allow(clippy::too_many_arguments)]
fn navigation_geometry(
    node_id: UiNodeId,
    fallback_frame: UiFrame,
    fallback_z_index: i32,
    fallback_paint_order: u64,
    arranged_tree: &UiArrangedTree,
    arranged_node_indices: &BTreeMap<UiNodeId, usize>,
    projected_hit_test: &UiProjectedHitTestIndex,
    base_hit_grid: &UiHitTestGrid,
) -> UiNavigationGeometry {
    if let Some(entry) = projected_hit_test.authoritative_entry(base_hit_grid, node_id) {
        return UiNavigationGeometry {
            frame: entry.frame,
            z_index: entry.z_index,
            paint_order: entry.paint_order,
        };
    }
    arranged_node_indexed(arranged_tree, arranged_node_indices, node_id)
        .map(|node| UiNavigationGeometry {
            frame: node.frame,
            z_index: node.z_index,
            paint_order: node.paint_order,
        })
        .unwrap_or(UiNavigationGeometry {
            frame: fallback_frame,
            z_index: fallback_z_index,
            paint_order: fallback_paint_order,
        })
}

#[allow(clippy::too_many_arguments)]
fn active_modal_navigation_group_scope(
    tree: &UiTree,
    arranged_tree: &UiArrangedTree,
    arranged_node_indices: &BTreeMap<UiNodeId, usize>,
    projected_hit_test: &UiProjectedHitTestIndex,
    base_hit_grid: &UiHitTestGrid,
    owner: UiNodeId,
    group: &UiNavigationGroup,
) -> Option<UiRankedNavigationScope> {
    let root = group.root.unwrap_or(owner);
    let root_node = tree.nodes.get(&root)?;
    if !root_node.state_flags.enabled || !root_node.is_render_visible() {
        return None;
    }
    if let Some(metadata) = root_node.template_metadata.as_ref() {
        if bool_attribute_any(metadata, &["disable_enforce_focus", "disableEnforceFocus"])
            || metadata_declares_open(metadata)
                && !(bool_attribute(metadata, "open") || bool_attribute(metadata, "popup_open"))
        {
            return None;
        }
    }
    let geometry = navigation_geometry(
        root,
        root_node.layout_cache.frame,
        root_node.z_index,
        root_node.paint_order,
        arranged_tree,
        arranged_node_indices,
        projected_hit_test,
        base_hit_grid,
    );
    Some(UiRankedNavigationScope {
        rank: (geometry.z_index, geometry.paint_order, root),
        scope: UiNavigationScope::Group {
            group_id: group.group_id.clone(),
        },
    })
}

#[allow(clippy::too_many_arguments)]
fn ranked_mui_scope(
    tree: &UiTree,
    arranged_tree: &UiArrangedTree,
    arranged_node_indices: &BTreeMap<UiNodeId, usize>,
    projected_hit_test: &UiProjectedHitTestIndex,
    base_hit_grid: &UiHitTestGrid,
    root: UiNodeId,
) -> Option<UiRankedNavigationScope> {
    let root_node = tree.nodes.get(&root)?;
    let geometry = navigation_geometry(
        root,
        root_node.layout_cache.frame,
        root_node.z_index,
        root_node.paint_order,
        arranged_tree,
        arranged_node_indices,
        projected_hit_test,
        base_hit_grid,
    );
    Some(UiRankedNavigationScope {
        rank: (geometry.z_index, geometry.paint_order, root),
        scope: UiNavigationScope::MuiOverlay(root),
    })
}

#[allow(clippy::too_many_arguments)]
fn declared_modal_scope_for_node(
    tree: &UiTree,
    arranged_tree: &UiArrangedTree,
    arranged_node_indices: &BTreeMap<UiNodeId, usize>,
    projected_hit_test: &UiProjectedHitTestIndex,
    base_hit_grid: &UiHitTestGrid,
    node_id: UiNodeId,
) -> Option<UiRankedNavigationScope> {
    let node = tree.nodes.get(&node_id)?;
    if is_active_mui_modal_focus_scope(node) {
        return ranked_mui_scope(
            tree,
            arranged_tree,
            arranged_node_indices,
            projected_hit_test,
            base_hit_grid,
            node_id,
        );
    }
    let group = node.navigation.group.as_ref().filter(|group| group.modal)?;
    let root = group.root.unwrap_or(node_id);
    let root_node = tree.nodes.get(&root)?;
    let metadata = root_node.template_metadata.as_ref()?;
    if !metadata_declares_open(metadata)
        || !(bool_attribute(metadata, "open") || bool_attribute(metadata, "popup_open"))
        || bool_attribute_any(metadata, &["disable_enforce_focus", "disableEnforceFocus"])
        || !root_node.state_flags.enabled
        || !root_node.is_render_visible()
    {
        return None;
    }
    active_modal_navigation_group_scope(
        tree,
        arranged_tree,
        arranged_node_indices,
        projected_hit_test,
        base_hit_grid,
        node_id,
        group,
    )
}

fn retain_topmost_scope(
    best: &mut Option<UiRankedNavigationScope>,
    candidate: Option<UiRankedNavigationScope>,
) {
    let Some(candidate) = candidate else {
        return;
    };
    if best
        .as_ref()
        .is_none_or(|current| candidate.rank > current.rank)
    {
        *best = Some(candidate);
    }
}

fn compare_tab_nodes(
    nodes: &BTreeMap<UiNodeId, UiNavigationIndexNode>,
    left: UiNodeId,
    right: UiNodeId,
) -> Ordering {
    let left_node = nodes
        .get(&left)
        .expect("tab candidate must resolve in navigation index");
    let right_node = nodes
        .get(&right)
        .expect("tab candidate must resolve in navigation index");
    left_node
        .group_order
        .cmp(&right_node.group_order)
        .then_with(|| left_node.tab_order.cmp(&right_node.tab_order))
        .then_with(|| left_node.paint_order.cmp(&right_node.paint_order))
        .then_with(|| left.cmp(&right))
}

fn compare_spatial_nodes(
    nodes: &BTreeMap<UiNodeId, UiNavigationIndexNode>,
    left: UiNodeId,
    right: UiNodeId,
) -> Ordering {
    let left_node = nodes
        .get(&left)
        .expect("spatial candidate must resolve in navigation index");
    let right_node = nodes
        .get(&right)
        .expect("spatial candidate must resolve in navigation index");
    left_node
        .paint_order
        .cmp(&right_node.paint_order)
        .then_with(|| left.cmp(&right))
}

fn position_map(candidates: &[UiNodeId]) -> BTreeMap<UiNodeId, usize> {
    candidates
        .iter()
        .enumerate()
        .map(|(index, node_id)| (*node_id, index))
        .collect()
}

fn target_for_direction(
    directional: &UiDirectionalNavigation,
    kind: UiNavigationEventKind,
) -> &UiDirectionalNavigationTarget {
    match kind {
        UiNavigationEventKind::Up => &directional.up,
        UiNavigationEventKind::Down => &directional.down,
        UiNavigationEventKind::Left => &directional.left,
        UiNavigationEventKind::Right => &directional.right,
        UiNavigationEventKind::Activate
        | UiNavigationEventKind::Cancel
        | UiNavigationEventKind::Home
        | UiNavigationEventKind::End
        | UiNavigationEventKind::Next
        | UiNavigationEventKind::Previous => &UiDirectionalNavigationTarget::Auto,
    }
}

fn is_active_mui_modal_focus_scope(node: &zircon_runtime_interface::ui::tree::UiTreeNode) -> bool {
    let Some(metadata) = node.template_metadata.as_ref() else {
        return false;
    };
    (is_mui_modal_focus_component(metadata.component.as_str())
        || metadata.widget.resolved_behavior(&metadata.component) == UiWidgetBehavior::Popup)
        && node.state_flags.enabled
        && node.is_render_visible()
        && (bool_attribute(metadata, "open") || bool_attribute(metadata, "popup_open"))
        && !bool_attribute_any(metadata, &["disable_enforce_focus", "disableEnforceFocus"])
}

fn is_navigation_geometry_authority(
    node_id: UiNodeId,
    node: &zircon_runtime_interface::ui::tree::UiTreeNode,
) -> bool {
    node.is_focus_candidate()
        || is_active_mui_modal_focus_scope(node)
        || node
            .navigation
            .group
            .as_ref()
            .is_some_and(|group| group.modal && group.root.unwrap_or(node_id) == node_id)
}

fn metadata_declares_open(metadata: &UiTemplateNodeMetadata) -> bool {
    metadata.attributes.contains_key("open") || metadata.attributes.contains_key("popup_open")
}

fn is_mui_modal_focus_component(component: &str) -> bool {
    matches!(
        component,
        "Dialog" | "ConfirmDialog" | "Modal" | "Popover" | "Menu"
    )
}

fn bool_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> bool {
    metadata
        .attributes
        .get(key)
        .and_then(toml::Value::as_bool)
        .unwrap_or(false)
}

fn bool_attribute_any(metadata: &UiTemplateNodeMetadata, keys: &[&str]) -> bool {
    keys.iter().any(|key| bool_attribute(metadata, key))
}

#[cfg(test)]
mod tests;
