use super::super::taffy_bridge::{
    compute_taffy_child_frames, taffy_main_axis, taffy_supports_axis_constraint_priority,
    taffy_supports_child_layout_values, taffy_supports_parent_layout_values,
    taffy_supports_slot_alignment, taffy_supports_slot_layout_values, taffy_supports_slot_padding,
    TaffyChildLayoutInput,
};
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{Pivot, Position, UiAxis, UiContainerKind, UiFrame, UiLayoutEngineFallbackReason},
    tree::{UiTree, UiTreeError},
};

use super::arrange::{arrange_node, hide_subtree_layout};
use super::engine::UiLayoutPassEngineContext;
use super::slot::{slot_for_container_child, UiLayoutSlotIndex};
use super::workspace::{
    recycle_taffy_arrange_scratch, take_taffy_arrange_scratch, UiTaffyArrangeScratch,
};

pub(super) fn try_arrange_taffy_owned_children(
    tree: &mut UiTree,
    parent_id: UiNodeId,
    children: &[UiNodeId],
    frame: UiFrame,
    inherited_clip: Option<UiFrame>,
    slot_index: &UiLayoutSlotIndex,
    engine_context: &mut UiLayoutPassEngineContext,
) -> Result<bool, UiTreeError> {
    let container = tree
        .node(parent_id)
        .ok_or(UiTreeError::MissingNode(parent_id))?
        .container;
    let Some(axis) = taffy_main_axis(container) else {
        return Ok(false);
    };
    if !taffy_supports_parent_layout_values(container, frame) {
        engine_context.record_taffy_fallback(
            parent_id,
            container,
            UiLayoutEngineFallbackReason::InvalidLayoutValue,
            None,
        );
        return Ok(false);
    }

    let mut scratch = take_taffy_arrange_scratch();
    let result = try_arrange_taffy_owned_children_with_scratch(
        tree,
        parent_id,
        children,
        frame,
        inherited_clip,
        slot_index,
        engine_context,
        container,
        axis,
        &mut scratch,
    );
    recycle_taffy_arrange_scratch(scratch);
    result
}

#[allow(clippy::too_many_arguments)]
fn try_arrange_taffy_owned_children_with_scratch(
    tree: &mut UiTree,
    parent_id: UiNodeId,
    children: &[UiNodeId],
    frame: UiFrame,
    inherited_clip: Option<UiFrame>,
    slot_index: &UiLayoutSlotIndex,
    engine_context: &mut UiLayoutPassEngineContext,
    container: UiContainerKind,
    axis: Option<UiAxis>,
    scratch: &mut UiTaffyArrangeScratch,
) -> Result<bool, UiTreeError> {
    taffy_layout_children(tree, children, container, scratch)?;
    if let Some(reason) = taffy_child_contracts_unsupported(
        tree,
        slot_index,
        parent_id,
        &scratch.layout_children,
        container,
        axis,
    )? {
        engine_context.record_taffy_fallback(parent_id, container, reason, None);
        return Ok(false);
    }

    scratch.bridge.begin_children(container);
    for child_id in scratch.layout_children.iter().copied() {
        let child = tree
            .node(child_id)
            .ok_or(UiTreeError::MissingNode(child_id))?;
        let slot = slot_for_container_child(tree, slot_index, parent_id, child_id, container);
        if let Err(error) = scratch.bridge.push_child(
            container,
            axis,
            TaffyChildLayoutInput {
                node_id: child_id,
                node: child,
                slot,
            },
        ) {
            engine_context.record_taffy_fallback(
                parent_id,
                container,
                error.fallback_reason(),
                Some(error.tree_build()),
            );
            return Ok(false);
        }
    }
    let outcome = compute_taffy_child_frames(container, frame, &mut scratch.bridge);
    let outcome = match outcome {
        Ok(outcome) => outcome,
        Err(error) => {
            engine_context.record_taffy_fallback(
                parent_id,
                container,
                error.fallback_reason(),
                Some(error.tree_build()),
            );
            return Ok(false);
        }
    };

    engine_context.record_taffy_native(parent_id, container, outcome.tree_build);
    for child_id in scratch.hidden_children.iter().copied() {
        hide_subtree_layout(tree, child_id, slot_index, engine_context)?;
    }
    for child_frame in scratch.bridge.child_frames().iter().copied() {
        arrange_node(
            tree,
            child_frame.node_id,
            child_frame.frame,
            inherited_clip,
            slot_index,
            engine_context,
        )?;
    }

    Ok(true)
}

fn taffy_layout_children(
    tree: &mut UiTree,
    children: &[UiNodeId],
    container: UiContainerKind,
    scratch: &mut UiTaffyArrangeScratch,
) -> Result<(), UiTreeError> {
    scratch.layout_children.clear();
    scratch.hidden_children.clear();
    for child_id in children.iter().copied() {
        let child = tree
            .node(child_id)
            .ok_or(UiTreeError::MissingNode(child_id))?;
        if child.effective_visibility().occupies_layout() {
            scratch.layout_children.push(child_id);
        } else if matches!(container, UiContainerKind::GridBox(_)) {
            scratch.layout_children.clear();
            scratch.layout_children.extend_from_slice(children);
            scratch.hidden_children.clear();
            return Ok(());
        } else {
            scratch.hidden_children.push(child_id);
        }
    }
    Ok(())
}

fn taffy_child_contracts_unsupported(
    tree: &UiTree,
    slot_index: &UiLayoutSlotIndex,
    parent_id: UiNodeId,
    children: &[UiNodeId],
    container: UiContainerKind,
    parent_axis: Option<UiAxis>,
) -> Result<Option<UiLayoutEngineFallbackReason>, UiTreeError> {
    for child_id in children.iter().copied() {
        let child = tree
            .node(child_id)
            .ok_or(UiTreeError::MissingNode(child_id))?;
        if !child.effective_visibility().occupies_layout() {
            return Ok(Some(
                UiLayoutEngineFallbackReason::UnsupportedChildVisibility,
            ));
        }
        // Template metadata carries render/event descriptors only; Taffy eligibility is decided by
        // authored placement and slot policies so v2 template assets can use the shared layout pass.
        if child.anchor != Default::default()
            || child.pivot != Pivot::default()
            || child.position != Position::default()
        {
            return Ok(Some(UiLayoutEngineFallbackReason::ChildPlacementPolicy));
        }
        if !taffy_supports_axis_constraint_priority(child, parent_axis) {
            return Ok(Some(UiLayoutEngineFallbackReason::AxisConstraintPriority));
        }
        if !taffy_supports_child_layout_values(child) {
            return Ok(Some(UiLayoutEngineFallbackReason::InvalidLayoutValue));
        }

        let slot = slot_for_container_child(tree, slot_index, parent_id, child_id, container);
        if let Some(slot) = slot {
            if slot.canvas_placement.is_some() {
                return Ok(Some(UiLayoutEngineFallbackReason::SlotCanvasPlacement));
            }
            if !taffy_supports_slot_layout_values(slot, container) {
                return Ok(Some(UiLayoutEngineFallbackReason::InvalidLayoutValue));
            }
            if !taffy_supports_slot_padding(slot.padding) {
                return Ok(Some(UiLayoutEngineFallbackReason::SlotFramePolicy));
            }
            if !taffy_supports_slot_alignment(child, slot, container) {
                return Ok(Some(UiLayoutEngineFallbackReason::SlotFramePolicy));
            }
        }
    }

    Ok(None)
}
