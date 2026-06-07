use zircon_runtime_interface::ui::{
    dispatch::{UiDispatchEffect, UiDragDropEffectKind},
    event_ui::UiNodeId,
};

use super::super::super::surface::UiSurface;
use super::super::require_valid_input_owner;
use super::node::require_node;

pub(super) fn apply_drag_drop_effect(
    surface: &mut UiSurface,
    effect: &UiDispatchEffect,
) -> Result<Option<UiNodeId>, String> {
    let UiDispatchEffect::DragDrop {
        target,
        kind,
        pointer_id,
        session_id,
        point,
        payload,
    } = effect
    else {
        return Err("expected drag/drop effect".to_string());
    };

    require_node(surface, *target)?;
    match kind {
        UiDragDropEffectKind::Begin => {
            require_valid_input_owner(surface, *target)?;
            surface.input.begin_drag_drop(
                *target,
                *target,
                *pointer_id,
                *session_id,
                *point,
                payload.clone(),
            )?;
            surface.focus.captured = Some(*target);
            surface.input.captured_pointer_id = Some(*pointer_id);
            set_dragging_component_state(surface, *target, true)?;
            set_drop_target_component_state(surface, *target, true)?;
            Ok(Some(*target))
        }
        UiDragDropEffectKind::Update => {
            require_valid_input_owner(surface, *target)?;
            let previous_target = active_drag_drop_target(surface);
            surface.input.update_drag_drop(
                *target,
                *pointer_id,
                *session_id,
                *point,
                payload.clone(),
            )?;
            sync_drag_drop_target_component_state(surface, previous_target, *target)?;
            Ok(Some(*target))
        }
        UiDragDropEffectKind::Accept => {
            require_valid_input_owner(surface, *target)?;
            let previous_target = active_drag_drop_target(surface);
            surface
                .input
                .accept_drag_drop(*target, *pointer_id, *session_id)?;
            sync_drag_drop_target_component_state(surface, previous_target, *target)?;
            Ok(Some(*target))
        }
        UiDragDropEffectKind::Reject => {
            require_valid_input_owner(surface, *target)?;
            let previous_target = active_drag_drop_target(surface);
            surface
                .input
                .reject_drag_drop(*target, *pointer_id, *session_id)?;
            if let Some(previous_target) = previous_target {
                set_drop_target_component_state(surface, previous_target, false)?;
            }
            set_drop_target_component_state(surface, *target, false)?;
            Ok(Some(*target))
        }
        UiDragDropEffectKind::Complete | UiDragDropEffectKind::Cancel => {
            let previous_target = active_drag_drop_target(surface);
            if let Some(source) = surface.input.end_drag_drop(*pointer_id, *session_id)? {
                if surface.focus.captured == Some(source) {
                    surface.focus.captured = None;
                }
                surface.input.clear_pointer_capture_for(source);
                set_dragging_component_state(surface, source, false)?;
            }
            if let Some(previous_target) = previous_target {
                set_drop_target_component_state(surface, previous_target, false)?;
            }
            Ok(Some(*target))
        }
    }
}

fn active_drag_drop_target(surface: &UiSurface) -> Option<UiNodeId> {
    surface.input.drag_drop.as_ref().map(|drag| drag.target)
}

fn set_dragging_component_state(
    surface: &mut UiSurface,
    node_id: UiNodeId,
    dragging: bool,
) -> Result<(), String> {
    if surface.component_states.set_dragging(node_id, dragging) {
        mark_component_state_render_dirty(surface, node_id)?;
    }
    Ok(())
}

fn set_drop_target_component_state(
    surface: &mut UiSurface,
    node_id: UiNodeId,
    active: bool,
) -> Result<(), String> {
    let drop_hover_changed = surface.component_states.set_drop_hovered(node_id, active);
    let active_target_changed = surface
        .component_states
        .set_active_drag_target(node_id, active);
    if drop_hover_changed || active_target_changed {
        mark_component_state_render_dirty(surface, node_id)?;
    }
    Ok(())
}

fn sync_drag_drop_target_component_state(
    surface: &mut UiSurface,
    previous_target: Option<UiNodeId>,
    current_target: UiNodeId,
) -> Result<(), String> {
    if previous_target != Some(current_target) {
        if let Some(previous_target) = previous_target {
            set_drop_target_component_state(surface, previous_target, false)?;
        }
    }
    set_drop_target_component_state(surface, current_target, true)
}

fn mark_component_state_render_dirty(
    surface: &mut UiSurface,
    node_id: UiNodeId,
) -> Result<(), String> {
    surface
        .mark_component_state_render_dirty(node_id)
        .map_err(|error| format!("component state dirty rejected: {error}"))
}
