use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
};

use serde::{Deserialize, Serialize};

use crate::ui::tree::{UiHitTestIndex, UiHitTestResult, UiRuntimeTreeRoutingExt};
use crate::ui::v2::UiV2RuntimeStyleIndex;
use zircon_runtime_interface::ui::accessibility::UiAccessibilityTreeSnapshot;
use zircon_runtime_interface::ui::tree::{UiTree, UiTreeError};
use zircon_runtime_interface::ui::{
    component::UiValue,
    event_ui::{UiNodeId, UiReflectorSnapshot, UiTreeId},
    focus::UiFocusChangeReason,
    layout::{UiLayoutEngineSelectionReport, UiPoint},
    surface::{
        UiArrangedTree, UiFocusPath, UiFocusState, UiHitTestDebugDump, UiHitTestQuery,
        UiNavigationState, UiRenderExtract, UiRenderList, UiSurfaceDebugOptions,
        UiSurfaceDebugSnapshot, UiSurfaceWindowState,
    },
};

use super::{
    arranged_focus_path,
    component_state::{property_may_affect_runtime_pseudo_state, UiSurfaceComponentStateStore},
    debug_hit_test_surface_frame, debug_surface_frame, debug_surface_frame_for_pick,
    debug_surface_frame_for_selection, debug_surface_frame_with_options,
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
    render::UiSurfaceRenderCache,
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
    pub hit_test: UiHitTestIndex,
    pub focus: UiFocusState,
    #[serde(default)]
    pub input: UiSurfaceInputState,
    #[serde(default)]
    pub component_states: UiSurfaceComponentStateStore,
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
            hit_test: UiHitTestIndex::default(),
            focus: UiFocusState::default(),
            input: UiSurfaceInputState::default(),
            component_states: UiSurfaceComponentStateStore::default(),
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
        self.runtime_style.apply_to_tree_subtree(
            &mut self.tree,
            &self.component_states,
            root_id,
            mark_dirty,
        )
    }

    pub fn hit_test(&self, point: UiPoint) -> UiHitTestResult {
        self.hit_test.hit_test_arranged(&self.arranged_tree, point)
    }

    pub fn hit_test_with_query(&self, query: UiHitTestQuery) -> UiHitTestResult {
        self.hit_test
            .hit_test_arranged_with_query(&self.arranged_tree, query)
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
        let node_id = request.node_id;
        let property = request.property.clone();
        let value = request.value.clone();
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
        if matches!(report.status, UiPropertyMutationStatus::Accepted) {
            if component_state_changed {
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
                self.sync_popup_stack_for_node(node_id, open);
                report.focus_change = self.apply_mui_modal_focus_transition(node_id, open)?;
            }
        }
        if matches!(report.status, UiPropertyMutationStatus::Accepted) {
            self.invalidation
                .record_dirty(node_id, report.invalidation.dirty);
        }
        Ok(report)
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
