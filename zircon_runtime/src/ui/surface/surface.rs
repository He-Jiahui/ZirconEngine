use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
};

use serde::{Deserialize, Serialize};

use crate::ui::v2::UiV2RuntimeStyleIndex;
use crate::ui::{
    layout::UiLayoutSlotIndex,
    tree::{UiHitTestIndex, UiHitTestResult, UiRuntimeTreeRoutingExt},
};
use zircon_runtime_interface::ui::accessibility::UiAccessibilityTreeSnapshot;
use zircon_runtime_interface::ui::tree::{UiDirtyFlags, UiTree, UiTreeError};
use zircon_runtime_interface::ui::{
    component::UiValue,
    dispatch::UiTransientDismissalTarget,
    event_ui::{UiNodeId, UiReflectorSnapshot, UiTreeId},
    focus::{UiFocusChangeEvent, UiFocusChangeReason},
    layout::{UiFrame, UiLayoutEngineSelectionReport, UiPoint},
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{
        UiArrangedNode, UiArrangedTree, UiFocusPath, UiFocusState, UiHitTestDebugDump,
        UiHitTestQuery, UiNavigationState, UiRenderCommand, UiRenderCommandKind, UiRenderExtract,
        UiRenderList, UiSurfaceDebugOptions, UiSurfaceDebugSnapshot, UiSurfaceWindowState,
    },
};

use super::{
    arranged_focus_path,
    component_state::{property_may_affect_runtime_pseudo_state, UiSurfaceComponentStateStore},
    control_index::UiSurfaceControlIndex,
    debug_hit_test_surface_frame, debug_surface_frame, debug_surface_frame_for_pick,
    debug_surface_frame_for_selection, debug_surface_frame_with_options,
    frame_hit_test::{hit_test_projected_grid_with_query, UiProjectedHitTestIndex},
    input::UiSurfaceInputState,
    invalidation::{
        UiInvalidationCommit, UiInvalidationGenerations, UiInvalidationTransaction,
        UiSurfaceInvalidationApplyError, UiSurfaceInvalidationState,
    },
    node_pool::{UiSurfaceNodePool, UiSurfaceNodePoolReport},
    property_mutation::{
        mutate_tree_property, UiPropertyMutationReport, UiPropertyMutationRequest,
        UiPropertyMutationStatus,
    },
    reflector_snapshot,
    render::{popup_base_z, UiSurfaceRenderCache},
};
use crate::ui::text::UiTextMeasureCache;

mod default_interactions;
mod event_routing;
mod frame_publication;
mod interaction_state;
mod pointer_component_events;
mod rebuild;

use frame_publication::UiSurfaceFramePublication;
pub use rebuild::UiSurfaceRebuildReport;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiSurface {
    pub tree: UiTree,
    pub arranged_tree: UiArrangedTree,
    #[serde(default, skip)]
    pub(super) arranged_node_indices: BTreeMap<UiNodeId, usize>,
    #[serde(default, skip)]
    pub(super) arranged_slot_indices: BTreeMap<UiNodeId, usize>,
    #[serde(default, skip)]
    pub(super) layout_slot_index: UiLayoutSlotIndex,
    pub hit_test: UiHitTestIndex,
    #[serde(default, skip)]
    pub(super) projected_hit_test: UiProjectedHitTestIndex,
    pub focus: UiFocusState,
    #[serde(default)]
    pub input: UiSurfaceInputState,
    #[serde(default)]
    pub component_states: UiSurfaceComponentStateStore,
    #[serde(default, skip)]
    pub(super) control_index: UiSurfaceControlIndex,
    #[serde(default, skip)]
    pub(crate) runtime_style: UiV2RuntimeStyleIndex,
    pub navigation: UiNavigationState,
    pub render_extract: UiRenderExtract,
    #[serde(default)]
    pub window_state: UiSurfaceWindowState,
    #[serde(default)]
    pub render_cache: UiSurfaceRenderCache,
    #[serde(default, skip)]
    pub(crate) text_measure_cache: UiTextMeasureCache,
    #[serde(default)]
    pub node_pool: UiSurfaceNodePool,
    #[serde(default)]
    pub(super) invalidation: UiSurfaceInvalidationState,
    #[serde(default, skip)]
    pub(super) last_layout_geometry_changed_node_ids: BTreeSet<UiNodeId>,
    #[serde(default, skip)]
    pub(super) dirty_node_ids: BTreeSet<UiNodeId>,
    #[serde(default, skip)]
    pub(super) dirty_index_initialized: bool,
    #[serde(default, skip)]
    pub(super) last_layout_root_size: Option<zircon_runtime_interface::ui::layout::UiSize>,
    pub last_rebuild_report: UiSurfaceRebuildReport,
    #[serde(default)]
    pub layout_engine_report: UiLayoutEngineSelectionReport,
    #[serde(default, skip)]
    pub(super) layout_engine_selection_indices: BTreeMap<UiNodeId, usize>,
    #[serde(default)]
    pub(super) pending_pool_report: UiSurfaceNodePoolReport,
    #[serde(default, skip)]
    pub(super) frame_publication: RefCell<UiSurfaceFramePublication>,
}

impl UiSurface {
    /// Returns the generation used by UI text measurement and shaping caches.
    pub fn shared_font_database_generation() -> u64 {
        crate::text::font::shared_font_database_generation()
    }

    pub fn new(tree_id: UiTreeId) -> Self {
        Self {
            tree: UiTree::new(tree_id.clone()),
            arranged_tree: UiArrangedTree {
                tree_id: tree_id.clone(),
                ..Default::default()
            },
            arranged_node_indices: BTreeMap::new(),
            arranged_slot_indices: BTreeMap::new(),
            layout_slot_index: UiLayoutSlotIndex::default(),
            hit_test: UiHitTestIndex::default(),
            projected_hit_test: UiProjectedHitTestIndex::default(),
            focus: UiFocusState::default(),
            input: UiSurfaceInputState::default(),
            component_states: UiSurfaceComponentStateStore::default(),
            control_index: UiSurfaceControlIndex::default(),
            runtime_style: UiV2RuntimeStyleIndex::default(),
            navigation: UiNavigationState::default(),
            render_extract: UiRenderExtract {
                tree_id,
                list: UiRenderList::default(),
                raster_scale: 1.0,
            },
            window_state: UiSurfaceWindowState::default(),
            render_cache: UiSurfaceRenderCache::default(),
            text_measure_cache: UiTextMeasureCache::default(),
            node_pool: UiSurfaceNodePool::default(),
            invalidation: UiSurfaceInvalidationState::default(),
            last_layout_geometry_changed_node_ids: BTreeSet::new(),
            dirty_node_ids: BTreeSet::new(),
            dirty_index_initialized: true,
            last_layout_root_size: None,
            last_rebuild_report: UiSurfaceRebuildReport::default(),
            layout_engine_report: UiLayoutEngineSelectionReport::default(),
            layout_engine_selection_indices: BTreeMap::new(),
            pending_pool_report: UiSurfaceNodePoolReport::default(),
            frame_publication: RefCell::new(UiSurfaceFramePublication::default()),
        }
    }

    pub fn component_state(
        &self,
        node_id: UiNodeId,
    ) -> Option<&zircon_runtime_interface::ui::component::UiComponentState> {
        self.component_states.get(node_id)
    }

    pub fn arranged_node(&self, node_id: UiNodeId) -> Option<&UiArrangedNode> {
        self.arranged_node_indices
            .get(&node_id)
            .and_then(|index| self.arranged_tree.nodes.get(*index))
            .filter(|node| node.node_id == node_id)
            .or_else(|| self.arranged_tree.get(node_id))
    }

    pub const fn invalidation_generations(&self) -> UiInvalidationGenerations {
        self.invalidation.generations()
    }

    pub fn last_layout_geometry_changed_node_ids(&self) -> &BTreeSet<UiNodeId> {
        &self.last_layout_geometry_changed_node_ids
    }

    pub fn last_invalidation_commit(&self) -> Option<&UiInvalidationCommit> {
        self.invalidation.last_commit()
    }

    pub fn pending_invalidation_changed_node_count(&self) -> usize {
        self.invalidation.pending_changed_node_count()
    }

    pub fn pending_rebuild_node_ids(&self) -> BTreeSet<UiNodeId> {
        let mut node_ids = self.dirty_node_ids.clone();
        node_ids.extend(self.invalidation.pending_changed_node_ids());
        node_ids.extend(self.tree.pending_mutation_node_ids().iter().copied());
        node_ids
    }

    pub fn begin_invalidation_transaction(&self) -> UiInvalidationTransaction {
        self.invalidation.begin_transaction()
    }

    pub fn apply_invalidation_transaction(
        &mut self,
        transaction: UiInvalidationTransaction,
    ) -> Result<(), UiSurfaceInvalidationApplyError> {
        self.invalidation.validate_transaction(&transaction)?;
        for change in transaction.changes() {
            if !self.tree.nodes.contains_key(&change.node_id) {
                return Err(UiTreeError::MissingNode(change.node_id).into());
            }
        }

        for change in transaction.changes().cloned().collect::<Vec<_>>() {
            self.mark_node_dirty(change.node_id, change.dirty)?;
            self.invalidation.record_change(&change);
        }
        Ok(())
    }

    pub(crate) fn set_runtime_style_index(&mut self, runtime_style: UiV2RuntimeStyleIndex) {
        self.runtime_style = runtime_style;
    }

    pub(crate) fn seed_component_states_from_tree_metadata(&mut self) {
        self.component_states.seed_from_tree_metadata(&self.tree);
        self.seed_popup_stack_from_tree_metadata();
    }

    pub(crate) fn apply_runtime_state_style_all(
        &mut self,
        mark_dirty: bool,
    ) -> Result<usize, UiTreeError> {
        if !self.runtime_style.has_runtime_rules() {
            return Ok(0);
        }
        let roots = self.tree.roots.clone();
        let mut changed = 0;
        for root in roots {
            changed += self.apply_runtime_state_style_subtree(root, mark_dirty)?;
        }
        Ok(changed)
    }

    pub(crate) fn apply_runtime_state_style_subtree(
        &mut self,
        root_id: UiNodeId,
        mark_dirty: bool,
    ) -> Result<usize, UiTreeError> {
        let report = self.runtime_style.apply_to_tree_subtree(
            &mut self.tree,
            &self.component_states,
            root_id,
            mark_dirty,
        )?;
        let changed_count = report.changed_count();
        if mark_dirty {
            for (node_id, dirty) in report.changed_nodes {
                self.mark_node_dirty(node_id, dirty)?;
            }
        }
        Ok(changed_count)
    }

    pub(crate) fn apply_runtime_state_style_node(
        &mut self,
        node_id: UiNodeId,
        mark_dirty: bool,
    ) -> Result<usize, UiTreeError> {
        let report = self.runtime_style.apply_to_tree_node(
            &mut self.tree,
            &self.component_states,
            node_id,
            mark_dirty,
        )?;
        let changed_count = report.changed_count();
        if mark_dirty {
            for (changed_node_id, dirty) in report.changed_nodes {
                self.mark_node_dirty(changed_node_id, dirty)?;
            }
        }
        Ok(changed_count)
    }

    pub fn hit_test(&self, point: UiPoint) -> UiHitTestResult {
        self.hit_test_with_query(UiHitTestQuery::new(point))
    }

    pub fn hit_test_with_query(&self, query: UiHitTestQuery) -> UiHitTestResult {
        hit_test_projected_grid_with_query(
            self.projected_hit_test
                .authoritative_grid(&self.hit_test.grid),
            &self.arranged_tree,
            query,
        )
    }

    pub(super) fn rendered_popup_background(
        &self,
        node_id: UiNodeId,
        arranged: &UiArrangedNode,
    ) -> Option<(usize, &UiRenderCommand)> {
        let (command_start, commands) = self
            .render_cache
            .commands_for_node(&self.render_extract, node_id)?;
        commands
            .iter()
            .enumerate()
            .filter(|(_, command)| {
                command.node_id == node_id
                    && command.kind == UiRenderCommandKind::Quad
                    && command.z_index == popup_base_z(arranged.z_index)
                    && command.style.painter_family == UiPainterFamily::Dropdown
                    && command.style.painter_state == UiPainterResolvedState::Open
            })
            .min_by_key(|(command_index, command)| (command.z_index, *command_index))
            .map(|(command_index, command)| (command_start + command_index, command))
    }

    pub fn accessibility_snapshot(&self) -> UiAccessibilityTreeSnapshot {
        crate::ui::accessibility::accessibility_snapshot(self)
    }

    pub fn debug_hit_test(&self, point: UiPoint) -> UiHitTestDebugDump {
        debug_hit_test_surface_frame(&self.surface_frame(), point)
    }

    pub fn debug_snapshot(&self) -> UiSurfaceDebugSnapshot {
        debug_surface_frame(&self.surface_frame())
    }

    pub fn debug_snapshot_with_options(
        &self,
        options: &UiSurfaceDebugOptions,
    ) -> UiSurfaceDebugSnapshot {
        debug_surface_frame_with_options(&self.surface_frame(), options)
    }

    pub fn debug_snapshot_for_pick(
        &self,
        query: UiHitTestQuery,
        options: &UiSurfaceDebugOptions,
    ) -> UiSurfaceDebugSnapshot {
        debug_surface_frame_for_pick(&self.surface_frame(), query, options)
    }

    pub fn debug_snapshot_for_selection(
        &self,
        selected_node: UiNodeId,
        options: &UiSurfaceDebugOptions,
    ) -> UiSurfaceDebugSnapshot {
        debug_surface_frame_for_selection(&self.surface_frame(), selected_node, options)
    }

    pub fn debug_snapshot_json(
        &self,
        options: &UiSurfaceDebugOptions,
    ) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.debug_snapshot_with_options(options))
    }

    pub fn mutate_property(
        &mut self,
        request: UiPropertyMutationRequest,
    ) -> Result<UiPropertyMutationReport, UiTreeError> {
        self.mutate_property_with_popup_branch_close(request, true)
    }

    pub(crate) fn dismiss_transient_ui(
        &mut self,
        target: UiTransientDismissalTarget,
    ) -> Result<Option<UiNodeId>, UiTreeError> {
        let popup_closures = matches!(
            target,
            UiTransientDismissalTarget::All | UiTransientDismissalTarget::PopupStack
        )
        .then(|| self.declarative_popup_closures())
        .unwrap_or_default();
        let route_owner = self.input.dismiss_transient_ui(target);
        for (popup_node_id, property) in popup_closures {
            let _ = self.mutate_property_with_popup_branch_close(
                UiPropertyMutationRequest::new(popup_node_id, property, UiValue::Bool(false)),
                false,
            )?;
        }
        Ok(route_owner)
    }

    pub(crate) fn dismiss_popup_by_id(
        &mut self,
        popup_id: &str,
    ) -> Result<Option<UiNodeId>, UiTreeError> {
        let fallback_route_owner = self.input.popup_owner(popup_id);
        let popup_node_id = self
            .input
            .popup_stack
            .iter()
            .find(|popup| popup.popup_id == popup_id)
            .and_then(|popup| popup.popup_node)
            .or_else(|| {
                self.unique_popup_state_for_id(popup_id)
                    .map(|(node_id, _, _)| node_id)
            });
        let Some(popup_node_id) = popup_node_id else {
            self.input.close_popup(popup_id);
            return Ok(fallback_route_owner);
        };
        let route_owner = self.popup_route_owner_for_node(popup_node_id).or_else(|| {
            (!self.is_popup_stack_node(popup_node_id))
                .then_some(fallback_route_owner)
                .flatten()
        });
        let mut popup_closures = self.popup_branch_closures(popup_node_id);
        let Some((property, open)) = self.popup_state_for_node(popup_node_id) else {
            self.input.close_popup_with_node(popup_node_id, popup_id);
            return Ok(route_owner);
        };
        if !open {
            self.input.close_popup_with_node(popup_node_id, popup_id);
            return Ok(route_owner);
        }
        popup_closures.push((popup_node_id, property.to_string()));
        for (popup_node_id, property) in popup_closures {
            let _ = self.mutate_property_with_popup_branch_close(
                UiPropertyMutationRequest::new(popup_node_id, property, UiValue::Bool(false)),
                false,
            )?;
        }
        Ok(route_owner)
    }

    pub(crate) fn set_declarative_popup_open_by_id(
        &mut self,
        popup_id: &str,
        open: bool,
    ) -> Result<bool, UiTreeError> {
        let Some((popup_node_id, property, current_open)) =
            self.unique_popup_state_for_id(popup_id)
        else {
            return Ok(false);
        };
        if current_open == open {
            if open {
                self.synchronize_open_popup_state(popup_node_id, property)?;
            }
            return Ok(true);
        }
        let _ = self.mutate_property(UiPropertyMutationRequest::new(
            popup_node_id,
            property,
            UiValue::Bool(open),
        ))?;
        Ok(true)
    }

    fn mutate_property_with_popup_branch_close(
        &mut self,
        request: UiPropertyMutationRequest,
        close_popup_branch: bool,
    ) -> Result<UiPropertyMutationReport, UiTreeError> {
        let node_id = request.node_id;
        let property = request.property.clone();
        let value = request.value.clone();
        let popup_close_descendants = (close_popup_branch
            && matches!(property.as_str(), "open" | "popup_open")
            && matches!(&value, UiValue::Bool(false))
            && self.is_popup_stack_node(node_id))
        .then(|| self.popup_branch_closures(node_id))
        .unwrap_or_default();
        for (descendant_id, descendant_property) in popup_close_descendants {
            let _ = self.mutate_property_with_popup_branch_close(
                UiPropertyMutationRequest::new(
                    descendant_id,
                    descendant_property,
                    UiValue::Bool(false),
                ),
                false,
            )?;
        }
        let mut report = mutate_tree_property(&mut self.tree, request)?;
        let previous_component_value =
            if matches!(report.status, UiPropertyMutationStatus::Accepted) {
                self.component_states
                    .get(node_id)
                    .and_then(|state| state.value(&property).cloned())
            } else {
                None
            };
        if matches!(report.status, UiPropertyMutationStatus::Accepted) {
            if let Some(attribute_value) = self
                .tree
                .nodes
                .get(&node_id)
                .and_then(|node| node.template_metadata.as_ref())
                .and_then(|metadata| metadata.attributes.get(&property))
                .cloned()
            {
                let _ = self.runtime_style.set_base_attribute(
                    node_id,
                    property.clone(),
                    attribute_value,
                );
            }
        }
        let component_state_changed = if matches!(report.status, UiPropertyMutationStatus::Accepted)
        {
            self.component_states
                .sync_from_property(node_id, &property, &value)
        } else {
            false
        };
        let popup_open_alias_state_changed =
            if matches!(report.status, UiPropertyMutationStatus::Accepted) {
                self.sync_popup_open_alias_state(node_id, &property, &value)
            } else {
                false
            };
        if matches!(report.status, UiPropertyMutationStatus::Accepted) {
            if component_state_changed || popup_open_alias_state_changed {
                self.mark_component_state_render_dirty(node_id)?;
                report.mark_render_dirty();
                report.record_component_state_value_update(
                    node_id,
                    property.clone(),
                    previous_component_value,
                    value.clone(),
                );
            } else if property_may_affect_runtime_pseudo_state(&property) {
                let changed = self.apply_runtime_state_style_subtree(node_id, true)?;
                if changed > 0 {
                    report.mark_render_dirty();
                }
            }
        }
        if matches!(report.status, UiPropertyMutationStatus::Accepted)
            && matches!(
                property.as_str(),
                "disabled" | "enabled" | "visible" | "visibility" | "focusable"
            )
        {
            let reason = focus_reconcile_reason(&property, &self.tree, node_id);
            report.focus_change = self.reconcile_focus_after_tree_change(reason);
        }
        if matches!(report.status, UiPropertyMutationStatus::Accepted)
            && matches!(property.as_str(), "open" | "popup_open")
        {
            if let UiValue::Bool(open) = value {
                let popup_stack_node = self.is_popup_stack_node(node_id);
                if !open && popup_stack_node {
                    self.reset_popup_open_state(node_id, property.as_str())?;
                }
                if open {
                    if self.synchronize_open_popup_state(node_id, property.as_str())? {
                        report.mark_render_dirty();
                    }
                } else {
                    let control_anchored_popup = self.popup_uses_control_anchor(node_id);
                    let popup_owner = self.sync_popup_stack_for_node(node_id, false);
                    report.focus_change = self.apply_mui_modal_focus_transition(
                        node_id,
                        false,
                        control_anchored_popup.then_some(popup_owner).flatten(),
                    )?;
                }
            }
        }
        if matches!(report.status, UiPropertyMutationStatus::Accepted) {
            self.invalidation
                .record_dirty(node_id, report.invalidation.dirty);
        }
        Ok(report)
    }

    fn synchronize_open_popup_state(
        &mut self,
        node_id: UiNodeId,
        property: &str,
    ) -> Result<bool, UiTreeError> {
        let control_anchored_popup = self.popup_uses_control_anchor(node_id);
        let popup_owner = self.sync_popup_stack_for_node(node_id, true);
        if control_anchored_popup && popup_owner.is_none() {
            let _ = self.reject_control_anchored_popup(node_id, property)?;
            return Ok(true);
        }
        let _ = self.apply_mui_modal_focus_transition(
            node_id,
            true,
            control_anchored_popup.then_some(popup_owner).flatten(),
        )?;
        Ok(false)
    }

    fn sync_popup_open_alias_state(
        &mut self,
        node_id: UiNodeId,
        property: &str,
        value: &UiValue,
    ) -> bool {
        if !matches!(value, UiValue::Bool(_)) || !self.is_popup_stack_node(node_id) {
            return false;
        }
        let alias = match property {
            "open" => "popup_open",
            "popup_open" => "open",
            _ => return false,
        };
        let Some(attribute_value) = self
            .tree
            .nodes
            .get(&node_id)
            .and_then(|node| node.template_metadata.as_ref())
            .and_then(|metadata| metadata.attributes.get(alias))
            .cloned()
        else {
            return false;
        };
        let _ = self
            .runtime_style
            .set_base_attribute(node_id, alias.to_string(), attribute_value);
        self.component_states
            .sync_from_property(node_id, alias, value)
    }

    pub(crate) fn reject_control_anchored_popup(
        &mut self,
        node_id: UiNodeId,
        property: &str,
    ) -> Result<Option<UiFocusChangeEvent>, UiTreeError> {
        self.reset_popup_open_state(node_id, property)?;
        self.apply_mui_modal_focus_transition(node_id, false, None)
    }

    fn reset_popup_open_state(
        &mut self,
        node_id: UiNodeId,
        property: &str,
    ) -> Result<(), UiTreeError> {
        let value = UiValue::Bool(false);
        let properties = if let Some(metadata) = self
            .tree
            .nodes
            .get_mut(&node_id)
            .and_then(|node| node.template_metadata.as_mut())
        {
            let properties = ["open", "popup_open"]
                .into_iter()
                .filter(|candidate| {
                    *candidate == property || metadata.attributes.contains_key(*candidate)
                })
                .collect::<Vec<_>>();
            for property in &properties {
                metadata
                    .attributes
                    .insert((*property).to_string(), toml::Value::Boolean(false));
            }
            properties
        } else {
            Vec::new()
        };
        if properties.is_empty() {
            return Ok(());
        }
        let mut component_state_changed = false;
        for property in properties {
            let _ = self.runtime_style.set_base_attribute(
                node_id,
                property.to_string(),
                toml::Value::Boolean(false),
            );
            component_state_changed |= self
                .component_states
                .sync_from_property(node_id, property, &value);
        }
        if component_state_changed {
            self.mark_component_state_render_dirty(node_id)?;
        }
        self.mark_node_dirty(
            node_id,
            UiDirtyFlags {
                layout: true,
                hit_test: true,
                render: true,
                input: true,
                ..UiDirtyFlags::default()
            },
        )?;
        Ok(())
    }

    pub fn reflector_snapshot(&self, query: Option<UiHitTestQuery>) -> UiReflectorSnapshot {
        reflector_snapshot(self, query)
    }

    pub fn bubble_route(&self, node_id: UiNodeId) -> Result<Vec<UiNodeId>, UiTreeError> {
        self.tree.bubble_route(node_id)
    }

    pub fn focus_path(&self) -> UiFocusPath {
        arranged_focus_path(&self.arranged_tree, self.focus.focused)
    }

    pub fn focused_route(&self) -> Vec<UiNodeId> {
        self.focus_path().bubble_route
    }
}

fn focus_reconcile_reason(property: &str, tree: &UiTree, node_id: UiNodeId) -> UiFocusChangeReason {
    match property {
        "disabled" | "enabled" | "focusable" => UiFocusChangeReason::Disabled,
        "visible" => UiFocusChangeReason::Hidden,
        "visibility" => tree
            .nodes
            .get(&node_id)
            .map(|node| {
                if node.is_render_visible() {
                    UiFocusChangeReason::Disabled
                } else {
                    UiFocusChangeReason::Hidden
                }
            })
            .unwrap_or(UiFocusChangeReason::Hidden),
        _ => UiFocusChangeReason::Clear,
    }
}
