use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use super::{
    arranged_focus_path_indexed,
    arranged_visibility::UiArrangedVisibilityIndex,
    clipboard_transfers::UiSurfaceClipboardTransferStore,
    component_state::{UiSurfaceComponentStateMigrationReport, UiSurfaceComponentStateStore},
    control_index::UiSurfaceControlIndex,
    debug_hit_test_surface_frame, debug_surface_frame_for_pick_with_ecs_projection,
    debug_surface_frame_for_selection_with_ecs_projection, debug_surface_frame_with_ecs_projection,
    debug_surface_frame_with_options_and_ecs_projection,
    frame_hit_test::UiProjectedHitTestIndex,
    input::{self, UiSurfaceInputState},
    invalidation::{
        UiInvalidationCommit, UiInvalidationGenerations, UiInvalidationTransaction,
        UiSurfaceInvalidationApplyError, UiSurfaceInvalidationState,
    },
    navigation_index::UiSurfaceNavigationIndex,
    node_pool::{UiSurfaceNodePool, UiSurfaceNodePoolReport},
    reflector_snapshot,
    render::{UiSurfaceRenderCache, popup_base_z},
    secure_text_values::UiSurfaceSecureTextValueStore,
    session_identity::{UiSurfaceSessionIdentity, UiSurfaceSessionIdentityHandle},
    virtual_list_materialization::UiVirtualListMaterializationIndex,
    virtual_list_prototype_pool::UiVirtualListPrototypePoolIndex,
};
use crate::text::{
    RichSemanticProjection, RichTextFormat,
    font::{FontCollectionService, shared_font_collection_service},
};
use crate::ui::text::UiTextMeasureCache;
use crate::ui::v2::UiV2RuntimeStyleIndex;
use crate::ui::{
    layout::UiLayoutSlotIndex,
    tree::{UiHitTestIndex, UiHitTestResult, UiRuntimeTreeRoutingExt},
};
use zircon_runtime_interface::ui::accessibility::UiAccessibilityTreeSnapshot;
use zircon_runtime_interface::ui::tree::{UiTree, UiTreeError};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiReflectorSnapshot, UiTreeId},
    layout::{UiFrame, UiLayoutEngineSelectionReport, UiPoint},
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{
        UiArrangedNode, UiArrangedTree, UiFocusPath, UiFocusState, UiHitTestDebugDump,
        UiHitTestQuery, UiNavigationState, UiRenderCommand, UiRenderCommandKind, UiRenderExtract,
        UiRenderList, UiSurfaceDebugOptions, UiSurfaceDebugSnapshot, UiSurfaceWindowState,
    },
    template::UiCompiledBindingProgram,
};

mod compiled_binding_event_index;
mod default_interactions;
mod event_routing;
mod font_generation;
mod frame_publication;
mod interaction_state;
mod pointer_component_events;
mod property_transaction;
mod rebuild;
mod virtual_window;

use compiled_binding_event_index::UiCompiledBindingEventIndex;
use frame_publication::UiSurfaceFramePublication;
pub use rebuild::{
    UiAuthoredGeometryFallbackReason, UiAuthoredGeometryPublication, UiSurfaceRebuildReport,
};
use virtual_window::UiVirtualWindowState;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiSurface {
    pub tree: UiTree,
    #[serde(default, skip)]
    pub(super) session_identity: UiSurfaceSessionIdentity,
    #[serde(default)]
    pub(super) compiled_bindings: UiCompiledBindingProgram,
    #[serde(default, skip)]
    pub(super) compiled_binding_event_index: UiCompiledBindingEventIndex,
    pub arranged_tree: UiArrangedTree,
    #[serde(default, skip)]
    pub(super) arranged_node_indices: BTreeMap<UiNodeId, usize>,
    #[serde(default, skip)]
    pub(super) arranged_visibility: UiArrangedVisibilityIndex,
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
    pub(super) secure_text_values: UiSurfaceSecureTextValueStore,
    #[serde(default, skip)]
    pub(super) clipboard_transfers: UiSurfaceClipboardTransferStore,
    #[serde(default, skip)]
    pub(super) control_index: UiSurfaceControlIndex,
    #[serde(default, skip)]
    pub(crate) runtime_style: UiV2RuntimeStyleIndex,
    pub navigation: UiNavigationState,
    #[serde(default, skip)]
    pub(super) navigation_index: UiSurfaceNavigationIndex,
    pub render_extract: UiRenderExtract,
    #[serde(default)]
    pub window_state: UiSurfaceWindowState,
    #[serde(default, skip)]
    pub render_cache: UiSurfaceRenderCache,
    #[serde(default, skip)]
    pub(crate) text_measure_cache: UiTextMeasureCache,
    #[serde(default, skip)]
    pub(super) observed_text_font_generation: u64,
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
    #[serde(default, skip)]
    pub(super) virtual_list_materialization: UiVirtualListMaterializationIndex,
    #[serde(default, skip)]
    pub(super) virtual_list_prototype_pool: UiVirtualListPrototypePoolIndex,
}

impl UiSurface {
    /// Returns the process-owner generation used by Editor-host and standalone UI caches.
    /// Core-owned Runtime surfaces track the generation of their injected collection internally.
    pub fn shared_font_database_generation() -> u64 {
        crate::text::font::shared_font_database_generation()
    }

    pub(crate) fn install_compiled_binding_program(&mut self, program: UiCompiledBindingProgram) {
        self.control_index
            .install_compiled_controls(&self.tree, &program);
        self.compiled_binding_event_index = UiCompiledBindingEventIndex::from_program(&program);
        self.compiled_bindings = program;
    }

    pub fn binding_program(&self) -> &UiCompiledBindingProgram {
        &self.compiled_bindings
    }

    /// Resolves an unambiguous retained control through the incremental surface index.
    pub fn unique_control_node_id(&self, control_id: &str) -> Option<UiNodeId> {
        self.control_index
            .unique_node_id_for_surface(&self.tree, control_id)
    }

    /// Builds an Editor-host or standalone surface from the process-owner font collection.
    /// Core-owned Runtime surfaces use `new_with_font_collection` through their owner-aware
    /// builders so layout and rendering share one font collection revision.
    pub fn new(tree_id: UiTreeId) -> Self {
        Self::new_with_font_collection(tree_id, shared_font_collection_service())
    }

    pub(crate) fn text_font_asset_dependencies(&self) -> Vec<String> {
        super::render::text_font_asset_dependencies(&self.tree)
    }

    pub(crate) fn new_with_font_collection(
        tree_id: UiTreeId,
        font_collection: Arc<FontCollectionService>,
    ) -> Self {
        let observed_text_font_generation = font_collection.generation();
        Self {
            tree: UiTree::new(tree_id.clone()),
            session_identity: UiSurfaceSessionIdentity::default(),
            compiled_bindings: UiCompiledBindingProgram::default(),
            compiled_binding_event_index: UiCompiledBindingEventIndex::default(),
            arranged_tree: UiArrangedTree {
                tree_id: tree_id.clone(),
                ..Default::default()
            },
            arranged_node_indices: BTreeMap::new(),
            arranged_visibility: UiArrangedVisibilityIndex::default(),
            arranged_slot_indices: BTreeMap::new(),
            layout_slot_index: UiLayoutSlotIndex::default(),
            hit_test: UiHitTestIndex::default(),
            projected_hit_test: UiProjectedHitTestIndex::default(),
            focus: UiFocusState::default(),
            input: UiSurfaceInputState::default(),
            component_states: UiSurfaceComponentStateStore::default(),
            secure_text_values: UiSurfaceSecureTextValueStore::default(),
            clipboard_transfers: UiSurfaceClipboardTransferStore::default(),
            control_index: UiSurfaceControlIndex::default(),
            runtime_style: UiV2RuntimeStyleIndex::default(),
            navigation: UiNavigationState::default(),
            navigation_index: UiSurfaceNavigationIndex::default(),
            render_extract: UiRenderExtract {
                tree_id,
                list: UiRenderList::default(),
                raster_scale: 1.0,
            },
            window_state: UiSurfaceWindowState::default(),
            render_cache: UiSurfaceRenderCache::default(),
            text_measure_cache: UiTextMeasureCache::new_with_font_collection(font_collection),
            observed_text_font_generation,
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
            virtual_list_materialization: UiVirtualListMaterializationIndex::default(),
            virtual_list_prototype_pool: UiVirtualListPrototypePoolIndex::default(),
        }
    }

    pub(crate) fn session_identity(&self) -> UiSurfaceSessionIdentityHandle {
        self.session_identity.handle()
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

        for change in transaction.into_changes() {
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

    pub(crate) fn adopt_hot_reload_state_from(
        &mut self,
        previous: &Self,
    ) -> UiSurfaceComponentStateMigrationReport {
        let report = self.component_states.migrate_stable_from(
            &previous.component_states,
            &previous.tree,
            &self.tree,
        );
        self.input = UiSurfaceInputState::default();
        self.focus = UiFocusState::default();
        self.navigation = UiNavigationState::default();
        self.control_index = UiSurfaceControlIndex::default();
        self.window_state = previous.window_state.clone();
        report
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
        self.hit_test_published_surface_frame_with_query(query)
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

    pub(crate) fn current_render_commands_for_node(
        &self,
        node_id: UiNodeId,
    ) -> Option<&[UiRenderCommand]> {
        self.render_cache
            .commands_for_node(&self.render_extract, node_id)
            .map(|(_, commands)| commands)
    }

    pub(crate) fn compile_rich_semantic_projection(
        &self,
        source_markup: &str,
        format: RichTextFormat,
    ) -> Option<RichSemanticProjection> {
        self.text_measure_cache
            .compile_rich_semantic_projection(source_markup, format)
    }

    pub fn accessibility_snapshot(&self) -> UiAccessibilityTreeSnapshot {
        crate::ui::accessibility::accessibility_snapshot(self)
    }

    pub(crate) fn accessibility_snapshot_bounded(
        &self,
        budget: &mut crate::ui::accessibility::AccessibilityBuildBudget,
    ) -> Result<
        UiAccessibilityTreeSnapshot,
        crate::ui::accessibility::AccessibilitySnapshotBudgetError,
    > {
        crate::ui::accessibility::accessibility_snapshot_bounded(self, budget)
    }

    pub(crate) fn accessibility_source_node_count(&self) -> usize {
        self.tree.nodes.len()
    }

    pub fn debug_hit_test(&self, point: UiPoint) -> UiHitTestDebugDump {
        debug_hit_test_surface_frame(&self.surface_frame(), point)
    }

    pub fn debug_snapshot(&self) -> UiSurfaceDebugSnapshot {
        let surface_frame = self.surface_frame();
        let ecs_projection = self.ui_ecs_projection();
        debug_surface_frame_with_ecs_projection(&surface_frame, &ecs_projection)
    }

    pub fn debug_snapshot_with_options(
        &self,
        options: &UiSurfaceDebugOptions,
    ) -> UiSurfaceDebugSnapshot {
        let surface_frame = self.surface_frame();
        let ecs_projection = self.ui_ecs_projection();
        debug_surface_frame_with_options_and_ecs_projection(
            &surface_frame,
            &ecs_projection,
            options,
        )
    }

    pub fn debug_snapshot_for_pick(
        &self,
        query: UiHitTestQuery,
        options: &UiSurfaceDebugOptions,
    ) -> UiSurfaceDebugSnapshot {
        let surface_frame = self.surface_frame();
        let ecs_projection = self.ui_ecs_projection();
        debug_surface_frame_for_pick_with_ecs_projection(
            &surface_frame,
            &ecs_projection,
            query,
            options,
        )
    }

    pub fn debug_snapshot_for_selection(
        &self,
        selected_node: UiNodeId,
        options: &UiSurfaceDebugOptions,
    ) -> UiSurfaceDebugSnapshot {
        let surface_frame = self.surface_frame();
        let ecs_projection = self.ui_ecs_projection();
        debug_surface_frame_for_selection_with_ecs_projection(
            &surface_frame,
            &ecs_projection,
            selected_node,
            options,
        )
    }

    pub fn debug_snapshot_json(
        &self,
        options: &UiSurfaceDebugOptions,
    ) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.debug_snapshot_with_options(options))
    }

    pub fn reflector_snapshot(&self, query: Option<UiHitTestQuery>) -> UiReflectorSnapshot {
        reflector_snapshot(self, query)
    }

    pub fn bubble_route(&self, node_id: UiNodeId) -> Result<Vec<UiNodeId>, UiTreeError> {
        self.tree.bubble_route(node_id)
    }

    pub fn focus_path(&self) -> UiFocusPath {
        arranged_focus_path_indexed(
            &self.arranged_tree,
            &self.arranged_node_indices,
            self.focus.focused,
        )
    }

    pub fn focused_route(&self) -> Vec<UiNodeId> {
        self.focus_path().bubble_route
    }
}
