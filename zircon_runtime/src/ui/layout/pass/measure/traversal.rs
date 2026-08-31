use std::{collections::BTreeSet, time::Instant};

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{DesiredSize, UiContainerKind, UiSize},
    tree::{UiTemplateNodeMetadata, UiTree, UiTreeError},
};

use crate::ui::text::UiTextMeasureCache;

use super::super::{
    axis::desired_axis,
    slot::UiLayoutSlotIndex,
    workspace::{UiLayoutPassWorkspace, UiMeasurePostOrderEntry},
};
use super::measure_content_size;

pub(crate) fn measure_node(
    tree: &mut UiTree,
    node_id: UiNodeId,
    text_measure_cache: &mut UiTextMeasureCache,
    slot_index: &UiLayoutSlotIndex,
) -> Result<DesiredSize, UiTreeError> {
    let profile_layout = std::env::var_os("ZR_UI_LAYOUT_PROFILE").is_some();
    slot_index.with_measure_workspace(|workspace| {
        let result = measure_node_with_profile(
            tree,
            node_id,
            text_measure_cache,
            slot_index,
            profile_layout,
            workspace,
            None,
            None,
        );
        workspace.clear_transient_lengths();
        result.map(|(desired_size, _)| desired_size)
    })
}

pub(crate) fn measure_node_incremental(
    tree: &mut UiTree,
    node_id: UiNodeId,
    text_measure_cache: &mut UiTextMeasureCache,
    slot_index: &UiLayoutSlotIndex,
    required_node_ids: &BTreeSet<UiNodeId>,
    visited_node_ids: &mut BTreeSet<UiNodeId>,
) -> Result<(DesiredSize, usize), UiTreeError> {
    let profile_layout = std::env::var_os("ZR_UI_LAYOUT_PROFILE").is_some();
    slot_index.with_measure_workspace(|workspace| {
        let result = measure_node_with_profile(
            tree,
            node_id,
            text_measure_cache,
            slot_index,
            profile_layout,
            workspace,
            Some(required_node_ids),
            Some(visited_node_ids),
        );
        workspace.clear_transient_lengths();
        result
    })
}

fn measure_node_with_profile(
    tree: &mut UiTree,
    node_id: UiNodeId,
    text_measure_cache: &mut UiTextMeasureCache,
    slot_index: &UiLayoutSlotIndex,
    profile_layout: bool,
    workspace: &mut UiLayoutPassWorkspace,
    required_node_ids: Option<&BTreeSet<UiNodeId>>,
    visited_node_ids: Option<&mut BTreeSet<UiNodeId>>,
) -> Result<(DesiredSize, usize), UiTreeError> {
    let mut measurement_probe_node_count = 0;
    workspace.post_order.clear();
    plan_measurement_post_order(
        tree,
        node_id,
        false,
        false,
        required_node_ids,
        &mut workspace.post_order,
        &mut measurement_probe_node_count,
    )?;
    if let Some(visited_node_ids) = visited_node_ids {
        visited_node_ids.extend(workspace.post_order.iter().map(|entry| entry.node_id));
    }

    for entry_index in 0..workspace.post_order.len() {
        let entry = workspace.post_order[entry_index];
        if entry.collapsed {
            let node = tree
                .node_mut(entry.node_id)
                .ok_or(UiTreeError::MissingNode(entry.node_id))?;
            node.layout_cache.desired_size = DesiredSize::default();
            node.layout_cache.content_size = UiSize::default();
            node.layout_cache.virtual_window = None;
            node.layout_cache.invalidate_measure();
            continue;
        }

        let profile_started = profile_layout.then(Instant::now);
        workspace.child_desired.clear();
        let (layout_boundary, constraints, container, child_count) = {
            let node = tree
                .node(entry.node_id)
                .ok_or(UiTreeError::MissingNode(entry.node_id))?;
            (
                node.layout_boundary,
                node.constraints,
                node.container,
                node.children.len(),
            )
        };
        let ordered_children =
            slot_index.ordered_children_for_container(tree, entry.node_id, container);
        for child_id in ordered_children.iter().copied() {
            let child = tree
                .node(child_id)
                .ok_or(UiTreeError::MissingNode(child_id))?;
            if child.effective_visibility().occupies_layout() {
                workspace
                    .child_desired
                    .push((child_id, child.layout_cache.desired_size));
            }
        }

        let content_size = measure_content_size(
            tree,
            entry.node_id,
            container,
            &workspace.child_desired,
            slot_index,
            tree.node(entry.node_id)
                .and_then(|node| node.template_metadata.as_ref()),
            text_measure_cache,
            &mut workspace.container_scratch,
        );
        let desired = DesiredSize::new(
            desired_axis(layout_boundary, constraints.width, content_size.width),
            desired_axis(layout_boundary, constraints.height, content_size.height),
        );

        {
            let node = tree
                .node_mut(entry.node_id)
                .ok_or(UiTreeError::MissingNode(entry.node_id))?;
            node.layout_cache.desired_size = desired;
            node.layout_cache.content_size = content_size;
            node.layout_cache.complete_measure();
            if !node.container.is_scrollable() {
                node.layout_cache.virtual_window = None;
            }
        }

        emit_slow_measure_profile(
            profile_started,
            entry.node_id,
            child_count,
            container,
            tree.node(entry.node_id)
                .and_then(|node| node.template_metadata.as_ref()),
        );
    }

    tree.node(node_id)
        .map(|node| (node.layout_cache.desired_size, measurement_probe_node_count))
        .ok_or(UiTreeError::MissingNode(node_id))
}

fn plan_measurement_post_order(
    tree: &UiTree,
    node_id: UiNodeId,
    ancestor_collapsed: bool,
    force_subtree: bool,
    required_node_ids: Option<&BTreeSet<UiNodeId>>,
    post_order: &mut Vec<UiMeasurePostOrderEntry>,
    measurement_probe_node_count: &mut usize,
) -> Result<(), UiTreeError> {
    *measurement_probe_node_count = (*measurement_probe_node_count).saturating_add(1);
    let node = tree
        .node(node_id)
        .ok_or(UiTreeError::MissingNode(node_id))?;
    if !ancestor_collapsed
        && !force_subtree
        && required_node_ids.is_some_and(|required| !required.contains(&node_id))
        && node.layout_cache.measure_valid
    {
        return Ok(());
    }
    let collapsed = ancestor_collapsed || !node.effective_visibility().occupies_layout();
    // A zero-sized frame can still be a valid measurement (for example a zero viewport).
    // An invalid node outside the explicit dependency path must be measured, but it must not
    // force valid descendants. Collapsed state remains the only whole-subtree invalidation.
    let force_children = force_subtree || collapsed;
    for child_id in &node.children {
        plan_measurement_post_order(
            tree,
            *child_id,
            collapsed,
            force_children,
            required_node_ids,
            post_order,
            measurement_probe_node_count,
        )?;
    }
    post_order.push(UiMeasurePostOrderEntry { node_id, collapsed });
    Ok(())
}

fn emit_slow_measure_profile(
    started: Option<Instant>,
    node_id: UiNodeId,
    child_count: usize,
    container: UiContainerKind,
    metadata: Option<&UiTemplateNodeMetadata>,
) {
    let Some(started) = started else {
        return;
    };
    let elapsed_ms = started.elapsed().as_millis();
    if elapsed_ms < 10 {
        return;
    }
    let component = metadata
        .map(|metadata| metadata.component.as_str())
        .unwrap_or("<missing>");
    eprintln!(
        "ui-layout-profile stage=slow-measure elapsed_ms={elapsed_ms} node_id={node_id:?} component={component} container={container:?} children={child_count}"
    );
}
