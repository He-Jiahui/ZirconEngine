use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{
        UiContainerKind, UiFrame, UiLayoutEngineBackend, UiLayoutEngineCapability,
        UiLayoutEngineFallbackReason, UiLayoutEngineRequest, UiLayoutEngineSelection,
        UiLayoutEngineSelectionReport, UiLayoutEngineSupport, UiLayoutEngineTaffyTreeBuildStats,
    },
    tree::UiTree,
};

use crate::ui::layout::taffy_bridge::UiTaffyChildContractScope;

#[derive(Debug, Default)]
pub(super) struct UiLayoutPassEngineContext {
    selections: Vec<UiLayoutEngineSelection>,
    reuse_geometry: bool,
    required_node_ids: BTreeSet<UiNodeId>,
    required_children_by_parent: BTreeMap<UiNodeId, Vec<UiNodeId>>,
    // A propagated ancestor may be required without changing its own layout contract.
    layout_source_node_ids: BTreeSet<UiNodeId>,
    mutation_source_node_ids: BTreeSet<UiNodeId>,
    visited_node_ids: BTreeSet<UiNodeId>,
    geometry_changed_node_ids: BTreeSet<UiNodeId>,
    arrangement_probe_node_count: usize,
}

impl UiLayoutPassEngineContext {
    pub(super) fn incremental(required_node_ids: BTreeSet<UiNodeId>) -> Self {
        Self::incremental_with_sources(required_node_ids, BTreeSet::new())
    }

    pub(super) fn incremental_with_sources(
        required_node_ids: BTreeSet<UiNodeId>,
        layout_source_node_ids: BTreeSet<UiNodeId>,
    ) -> Self {
        Self {
            reuse_geometry: true,
            required_node_ids,
            layout_source_node_ids,
            ..Self::default()
        }
    }

    pub(super) fn index_required_children(&mut self, tree: &UiTree) {
        self.required_children_by_parent.clear();
        for node_id in &self.required_node_ids {
            let Some(node) = tree.node(*node_id) else {
                continue;
            };
            let Some(parent_id) = node.parent else {
                continue;
            };
            self.required_children_by_parent
                .entry(parent_id)
                .or_default()
                .push(*node_id);
        }
        self.mutation_source_node_ids = tree.pending_mutation_node_ids().clone();
        for node_id in tree.pending_mutation_node_ids().iter().copied() {
            let mut current = node_id;
            while let Some(parent_id) = tree.node(current).and_then(|node| node.parent) {
                let children = self
                    .required_children_by_parent
                    .entry(parent_id)
                    .or_default();
                if !children.contains(&current) {
                    children.push(current);
                }
                current = parent_id;
            }
        }
    }

    pub(super) fn copy_required_children(
        &self,
        tree: &UiTree,
        parent_id: UiNodeId,
        output: &mut Vec<UiNodeId>,
    ) {
        // Independent containers do not need tree-order traversal: free child geometry is
        // determined per child, while the retained tree remains the paint-order authority.
        output.clear();
        if tree.node(parent_id).is_none() {
            return;
        }
        if let Some(required_children) = self.required_children_by_parent.get(&parent_id) {
            output.extend(required_children.iter().copied());
        }
    }

    pub(super) fn can_sparse_arrange_independent_children(
        &self,
        node_id: UiNodeId,
        container: UiContainerKind,
        structure_dirty: bool,
        previous_frame: UiFrame,
        previous_clip_frame: Option<UiFrame>,
        next_frame: UiFrame,
        next_clip_frame: Option<UiFrame>,
    ) -> bool {
        self.reuse_geometry
            && matches!(
                container,
                UiContainerKind::Free
                    | UiContainerKind::Canvas
                    | UiContainerKind::Container
                    | UiContainerKind::Overlay
                    | UiContainerKind::SizeBox(_)
                    | UiContainerKind::Space
            )
            && !structure_dirty
            && !self.layout_source_node_ids.contains(&node_id)
            && previous_frame == next_frame
            && previous_clip_frame == next_clip_frame
    }

    pub(super) fn taffy_child_contract_scope(
        &self,
        parent_id: UiNodeId,
        structure_dirty: bool,
    ) -> UiTaffyChildContractScope {
        if !self.reuse_geometry
            || structure_dirty
            || self.layout_source_node_ids.contains(&parent_id)
            || self.mutation_source_node_ids.contains(&parent_id)
        {
            return UiTaffyChildContractScope::Full;
        }
        UiTaffyChildContractScope::Exact
    }

    pub(super) fn can_reuse_geometry(
        &self,
        node_id: UiNodeId,
        previous_frame: UiFrame,
        previous_clip_frame: Option<UiFrame>,
        next_frame: UiFrame,
        next_clip_frame: Option<UiFrame>,
    ) -> bool {
        self.reuse_geometry
            && !self.required_node_ids.contains(&node_id)
            && previous_frame == next_frame
            && previous_clip_frame == next_clip_frame
    }

    pub(super) fn record_arrangement_probe(&mut self) {
        self.arrangement_probe_node_count = self.arrangement_probe_node_count.saturating_add(1);
    }

    pub(super) fn record_geometry(
        &mut self,
        node_id: UiNodeId,
        previous_frame: UiFrame,
        previous_clip_frame: Option<UiFrame>,
        next_frame: UiFrame,
        next_clip_frame: Option<UiFrame>,
    ) {
        self.visited_node_ids.insert(node_id);
        if previous_frame != next_frame || previous_clip_frame != next_clip_frame {
            self.geometry_changed_node_ids.insert(node_id);
        }
    }

    pub(super) fn record_taffy_native(
        &mut self,
        node_id: UiNodeId,
        container: UiContainerKind,
        taffy_tree_build: Option<UiLayoutEngineTaffyTreeBuildStats>,
    ) {
        let selection = UiLayoutEngineSelection::select(
            &UiLayoutEngineRequest::from_container_kind(container),
            &UiLayoutEngineCapability::taffy_flex_grid_wrap_block(),
            &UiLayoutEngineCapability::zircon(),
        )
        .with_node_id(node_id);
        self.selections.push(match taffy_tree_build {
            Some(tree_build) => selection.with_taffy_tree_build(tree_build),
            None => selection,
        });
    }

    pub(super) fn record_taffy_fallback(
        &mut self,
        node_id: UiNodeId,
        container: UiContainerKind,
        reason: UiLayoutEngineFallbackReason,
        taffy_tree_build: Option<UiLayoutEngineTaffyTreeBuildStats>,
    ) {
        self.selections.push(UiLayoutEngineSelection {
            node_id: Some(node_id),
            request: UiLayoutEngineRequest::from_container_kind(container),
            requested_backend: UiLayoutEngineBackend::Taffy,
            selected_backend: UiLayoutEngineBackend::Zircon,
            support: UiLayoutEngineSupport::Fallback,
            fallback_reason: Some(reason),
            taffy_tree_build,
        });
    }

    pub(super) fn record_zircon_owned(&mut self, node_id: UiNodeId, container: UiContainerKind) {
        self.selections.push(
            UiLayoutEngineSelection::select(
                &UiLayoutEngineRequest::from_container_kind(container),
                &UiLayoutEngineCapability::taffy_flex_grid_wrap_block(),
                &UiLayoutEngineCapability::zircon(),
            )
            .with_node_id(node_id),
        );
    }

    pub(super) fn finish(self) -> UiLayoutEngineSelectionReport {
        UiLayoutEngineSelectionReport::from_selections(self.selections)
    }

    pub(super) fn finish_incremental(
        self,
    ) -> (
        UiLayoutEngineSelectionReport,
        BTreeSet<UiNodeId>,
        BTreeSet<UiNodeId>,
        usize,
    ) {
        (
            UiLayoutEngineSelectionReport::from_selections(self.selections),
            self.visited_node_ids,
            self.geometry_changed_node_ids,
            self.arrangement_probe_node_count,
        )
    }
}
