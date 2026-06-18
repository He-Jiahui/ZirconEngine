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
use super::slot::{ordered_children_for_container, slot_for_container_child};

pub(super) fn try_arrange_taffy_owned_children(
    tree: &mut UiTree,
    parent_id: UiNodeId,
    children: &[UiNodeId],
    frame: UiFrame,
    inherited_clip: Option<UiFrame>,
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

    let (layout_children, hidden_children) =
        taffy_layout_children(tree, parent_id, children, container)?;
    if let Some(reason) =
        taffy_child_contracts_unsupported(tree, parent_id, &layout_children, container, axis)?
    {
        engine_context.record_taffy_fallback(parent_id, container, reason, None);
        return Ok(false);
    }

    let outcome = {
        let mut child_inputs = Vec::with_capacity(layout_children.len());
        for child_id in &layout_children {
            let child = tree
                .node(*child_id)
                .ok_or(UiTreeError::MissingNode(*child_id))?;
            let slot = slot_for_container_child(tree, parent_id, *child_id, container);
            child_inputs.push(TaffyChildLayoutInput {
                node_id: *child_id,
                node: child,
                slot,
            });
        }
        compute_taffy_child_frames(container, frame, &child_inputs)
    };
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
    for child_id in hidden_children {
        hide_subtree_layout(tree, child_id)?;
    }
    for child_frame in outcome.child_frames {
        arrange_node(
            tree,
            child_frame.node_id,
            child_frame.frame,
            inherited_clip,
            engine_context,
        )?;
    }

    Ok(true)
}

fn taffy_layout_children(
    tree: &mut UiTree,
    parent_id: UiNodeId,
    children: &[UiNodeId],
    container: UiContainerKind,
) -> Result<(Vec<UiNodeId>, Vec<UiNodeId>), UiTreeError> {
    let ordered_children = ordered_children_for_container(tree, parent_id, children, container);
    let mut layout_children = Vec::with_capacity(ordered_children.len());
    let mut hidden_children = Vec::new();
    for child_id in ordered_children {
        let child = tree
            .node(child_id)
            .ok_or(UiTreeError::MissingNode(child_id))?;
        if child.effective_visibility().occupies_layout() {
            layout_children.push(child_id);
        } else if matches!(container, UiContainerKind::GridBox(_)) {
            return Ok((children.to_vec(), Vec::new()));
        } else {
            hidden_children.push(child_id);
        }
    }
    Ok((layout_children, hidden_children))
}

fn taffy_child_contracts_unsupported(
    tree: &UiTree,
    parent_id: UiNodeId,
    children: &[UiNodeId],
    container: UiContainerKind,
    parent_axis: Option<UiAxis>,
) -> Result<Option<UiLayoutEngineFallbackReason>, UiTreeError> {
    for child_id in children {
        let child = tree
            .node(*child_id)
            .ok_or(UiTreeError::MissingNode(*child_id))?;
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

        let slot = slot_for_container_child(tree, parent_id, *child_id, container);
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
