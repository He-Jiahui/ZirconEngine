use std::collections::BTreeSet;

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{UiFrame, UiLayoutEngineSelectionReport, UiSize},
    tree::{UiTree, UiTreeError},
};

use crate::ui::text::UiTextMeasureCache;

use super::{
    arrange::{arrange_node, arrange_resized_root},
    child_frame::free_child_frame,
    engine::UiLayoutPassEngineContext,
    inline_widgets::arrange_inline_widget_children,
    measure::measure_node_incremental,
    pipeline::{assert_layout_pass_stage, UiLayoutPassStage},
    responsive_mui::{apply_mui_responsive_layout_for_nodes, apply_mui_responsive_layout_indexed},
    slot::{slot_for_container_child, UiLayoutSlotIndex},
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct UiIncrementalLayoutStats {
    pub visited_node_count: usize,
    pub visited_node_ids: BTreeSet<UiNodeId>,
    pub layout_engine_route_node_ids: BTreeSet<UiNodeId>,
    pub removed_node_ids: BTreeSet<UiNodeId>,
    pub geometry_changed_node_count: usize,
    pub geometry_changed_node_ids: BTreeSet<UiNodeId>,
    pub skipped_node_count: usize,
    pub layout_measure_probe_node_count: usize,
    pub layout_arrange_probe_node_count: usize,
    pub layout_engine_report: UiLayoutEngineSelectionReport,
}

pub(crate) fn compute_incremental_layout_tree_with_text_measure_cache(
    tree: &mut UiTree,
    root_size: UiSize,
    text_measure_cache: &mut UiTextMeasureCache,
    dirty_node_ids: &BTreeSet<UiNodeId>,
    root_size_changed: bool,
    layout_slot_index: &UiLayoutSlotIndex,
) -> Result<UiIncrementalLayoutStats, UiTreeError> {
    assert_layout_pass_stage(UiLayoutPassStage::ResponsiveStyleResolution, 0);
    layout_slot_index.ensure_initialized(tree);
    let mut layout_dirty_node_ids = dirty_node_ids.clone();
    layout_dirty_node_ids.extend(tree.pending_mutation_node_ids().iter().copied());
    layout_slot_index.synchronize_responsive_candidates(tree, &layout_dirty_node_ids);
    if root_size_changed {
        apply_mui_responsive_layout_indexed(tree, root_size, layout_slot_index)?;
    } else {
        apply_mui_responsive_layout_for_nodes(tree, root_size, dirty_node_ids, layout_slot_index)?;
    }
    layout_dirty_node_ids.extend(tree.pending_mutation_node_ids().iter().copied());
    layout_slot_index.synchronize_ordered_children(tree, &layout_dirty_node_ids);
    layout_slot_index.synchronize_parent_size_dependencies(tree, &layout_dirty_node_ids);
    let removed_node_ids = layout_dirty_node_ids
        .iter()
        .copied()
        .filter(|node_id| !tree.nodes.contains_key(node_id))
        .collect::<BTreeSet<_>>();

    let measurement_roots = incremental_layout_roots(tree, &layout_dirty_node_ids)?;
    let required_node_ids =
        layout_dependency_paths(tree, &measurement_roots, &layout_dirty_node_ids)?;
    let mut visited = BTreeSet::new();
    let mut layout_measure_probe_node_count = 0usize;
    let mut engine_context = UiLayoutPassEngineContext::incremental_with_sources(
        required_node_ids.clone(),
        tree.pending_layout_source_node_ids().clone(),
    );
    engine_context.index_required_children(tree);

    assert_layout_pass_stage(UiLayoutPassStage::Measurement, 1);
    for root_id in &measurement_roots {
        let (_, measurement_probe_node_count) = measure_node_incremental(
            tree,
            *root_id,
            &mut *text_measure_cache,
            layout_slot_index,
            &required_node_ids,
            &mut visited,
        )?;
        layout_measure_probe_node_count =
            layout_measure_probe_node_count.saturating_add(measurement_probe_node_count);
    }

    assert_layout_pass_stage(UiLayoutPassStage::BackendSelection, 2);
    assert_layout_pass_stage(UiLayoutPassStage::TaffyBridgeArrangement, 3);
    assert_layout_pass_stage(UiLayoutPassStage::ZirconFallbackArrangement, 4);
    assert_layout_pass_stage(UiLayoutPassStage::ClipAndVirtualWindowPropagation, 5);
    let mut arrangement_roots = measurement_roots;
    if root_size_changed {
        arrangement_roots.extend(tree.roots.iter().copied());
        arrangement_roots.sort_unstable();
        arrangement_roots.dedup();
    }
    let pure_root_resize = root_size_changed && required_node_ids.is_empty();
    for root_id in arrangement_roots.iter().copied() {
        arrange_layout_root(
            tree,
            root_id,
            root_size,
            layout_slot_index,
            &mut engine_context,
            pure_root_resize,
        )?;
    }
    arrange_inline_widget_children(
        tree,
        &arrangement_roots,
        text_measure_cache,
        layout_slot_index,
        &mut engine_context,
    )?;

    let (
        layout_engine_report,
        layout_engine_route_node_ids,
        geometry_changed_node_ids,
        layout_arrange_probe_node_count,
    ) = engine_context.finish_incremental();
    visited.extend(layout_engine_route_node_ids.iter().copied());
    let geometry_changed_node_count = geometry_changed_node_ids.len();

    let visited_node_count = visited.len();
    let skipped_node_count = tree.nodes.len().saturating_sub(visited_node_count);

    assert_layout_pass_stage(UiLayoutPassStage::SelectionReport, 6);
    Ok(UiIncrementalLayoutStats {
        visited_node_count,
        visited_node_ids: visited,
        layout_engine_route_node_ids,
        removed_node_ids,
        geometry_changed_node_count,
        geometry_changed_node_ids,
        skipped_node_count,
        layout_measure_probe_node_count,
        layout_arrange_probe_node_count,
        layout_engine_report,
    })
}

fn layout_dependency_paths(
    tree: &UiTree,
    roots: &[UiNodeId],
    dirty_node_ids: &BTreeSet<UiNodeId>,
) -> Result<BTreeSet<UiNodeId>, UiTreeError> {
    let roots = roots.iter().copied().collect::<BTreeSet<_>>();
    let mut required = BTreeSet::new();
    for node_id in dirty_node_ids.iter().copied() {
        let Some(node) = tree.node(node_id) else {
            continue;
        };
        if !(node.dirty.layout || node.dirty.style || node.dirty.text || node.dirty.visible_range) {
            continue;
        }

        let mut current = node_id;
        loop {
            required.insert(current);
            if roots.contains(&current) {
                break;
            }
            let Some(parent_id) = tree
                .node(current)
                .ok_or(UiTreeError::MissingNode(current))?
                .parent
            else {
                break;
            };
            current = parent_id;
        }
    }
    Ok(required)
}

fn incremental_layout_roots(
    tree: &UiTree,
    dirty_node_ids: &BTreeSet<UiNodeId>,
) -> Result<Vec<UiNodeId>, UiTreeError> {
    let candidates = dirty_node_ids
        .iter()
        .filter_map(|node_id| tree.nodes.get(node_id))
        .filter(|node| {
            node.dirty.layout || node.dirty.style || node.dirty.text || node.dirty.visible_range
        })
        .map(|node| propagated_layout_root(tree, node.node_id))
        .collect::<Result<BTreeSet<_>, _>>()?;

    let mut roots = Vec::new();
    for candidate in candidates.iter().copied() {
        if !has_ancestor_in(candidate, &candidates, tree)? {
            roots.push(candidate);
        }
    }
    Ok(roots)
}

fn propagated_layout_root(tree: &UiTree, node_id: UiNodeId) -> Result<UiNodeId, UiTreeError> {
    let mut current = node_id;
    let mut root = node_id;
    while let Some(parent_id) = tree
        .node(current)
        .ok_or(UiTreeError::MissingNode(current))?
        .parent
    {
        let parent = tree
            .node(parent_id)
            .ok_or(UiTreeError::MissingParent(parent_id))?;
        if !(parent
            .layout_boundary
            .propagates_child_layout_invalidation()
            || parent.container.is_auto_layout_container())
        {
            break;
        }
        root = parent_id;
        current = parent_id;
    }
    Ok(root)
}

fn has_ancestor_in(
    node_id: UiNodeId,
    roots: &BTreeSet<UiNodeId>,
    tree: &UiTree,
) -> Result<bool, UiTreeError> {
    let mut current = node_id;
    while let Some(parent_id) = tree
        .node(current)
        .ok_or(UiTreeError::MissingNode(current))?
        .parent
    {
        if roots.contains(&parent_id) {
            return Ok(true);
        }
        current = parent_id;
    }
    Ok(false)
}

fn arrange_layout_root(
    tree: &mut UiTree,
    root_id: UiNodeId,
    root_size: UiSize,
    slot_index: &UiLayoutSlotIndex,
    engine_context: &mut UiLayoutPassEngineContext,
    pure_root_resize: bool,
) -> Result<(), UiTreeError> {
    let parent_id = tree
        .node(root_id)
        .ok_or(UiTreeError::MissingNode(root_id))?
        .parent;
    let Some(parent_id) = parent_id else {
        if pure_root_resize {
            return arrange_resized_root(
                tree,
                root_id,
                root_frame(root_size),
                slot_index,
                engine_context,
            );
        }
        return arrange_node(
            tree,
            root_id,
            root_frame(root_size),
            None,
            slot_index,
            engine_context,
        );
    };

    let parent = tree
        .node(parent_id)
        .ok_or(UiTreeError::MissingParent(parent_id))?;
    let parent_frame = parent.layout_cache.frame;
    let inherited_clip = parent.layout_cache.clip_frame;
    let parent_container = parent.container;
    let child_frame = free_child_frame(
        tree,
        root_id,
        parent_frame,
        slot_for_container_child(tree, slot_index, parent_id, root_id, parent_container),
    )?;

    arrange_node(
        tree,
        root_id,
        child_frame,
        inherited_clip,
        slot_index,
        engine_context,
    )
}

fn root_frame(root_size: UiSize) -> UiFrame {
    UiFrame::new(
        0.0,
        0.0,
        root_size.width.max(0.0),
        root_size.height.max(0.0),
    )
}
