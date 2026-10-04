use crate::core::editor_event::{EditorEventEffect, EditorViewportEvent};
use crate::scene::viewport::ViewportFeedback;
use crate::ui::binding::ViewportCommand;
use crate::ui::host::EditorHostEventController;
use crate::ui::workbench::shell_state::WorkbenchShellStateData;
use crate::ui::workbench::state::EditorViewportStateError;

use super::execution_outcome::ExecutionOutcome;

pub(super) fn execute_viewport_event(
    _controller: &EditorHostEventController,
    shell: &mut WorkbenchShellStateData,
    event: &EditorViewportEvent,
) -> Result<ExecutionOutcome, EditorViewportStateError> {
    let (target_view, event) = match event {
        EditorViewportEvent::ForView { view_id, event } => {
            // The event wire carries the core nominal ID while the Workbench manager retains its
            // own UI nominal ID. Keep the manager lookup on the UI side and the viewport registry
            // lookup on the core side; matching raw strings at this boundary would hide a type
            // ownership error.
            let manager_view_id =
                crate::ui::workbench::view::ViewInstanceId::new(view_id.0.clone());
            if !shell
                .manager
                .view_instance_ids_for_descriptor_key("editor.scene")
                .iter()
                .any(|candidate| candidate == &manager_view_id)
                || shell
                    .state
                    .viewport_controller
                    .session_if_live(view_id)
                    .is_none()
            {
                return Err(EditorViewportStateError::StaleViewportView {
                    view_id: view_id.clone(),
                });
            }
            (Some(view_id), event.as_ref())
        }
        _ => (None, event),
    };
    let command = match event {
        EditorViewportEvent::PointerMoved { x, y } => {
            ViewportCommand::PointerMoved { x: *x, y: *y }
        }
        EditorViewportEvent::LeftPressed {
            x,
            y,
            selection_mutation,
        } => ViewportCommand::LeftPressed {
            x: *x,
            y: *y,
            selection_mutation: *selection_mutation,
        },
        EditorViewportEvent::LeftReleased => ViewportCommand::LeftReleased,
        EditorViewportEvent::CancelInteraction => ViewportCommand::CancelInteraction,
        EditorViewportEvent::RightPressed { x, y } => {
            ViewportCommand::RightPressed { x: *x, y: *y }
        }
        EditorViewportEvent::RightReleased => ViewportCommand::RightReleased,
        EditorViewportEvent::MiddlePressed { x, y } => {
            ViewportCommand::MiddlePressed { x: *x, y: *y }
        }
        EditorViewportEvent::MiddleReleased => ViewportCommand::MiddleReleased,
        EditorViewportEvent::Scrolled { delta } => ViewportCommand::Scrolled { delta: *delta },
        EditorViewportEvent::Resized { width, height } => ViewportCommand::Resized {
            width: *width,
            height: *height,
        },
        EditorViewportEvent::ActivateSceneMode { mode } => {
            ViewportCommand::ActivateSceneMode(mode.clone())
        }
        EditorViewportEvent::SetTransformSpace { space } => {
            ViewportCommand::SetTransformSpace(*space)
        }
        EditorViewportEvent::SetPivotMode { mode } => ViewportCommand::SetPivotMode(*mode),
        EditorViewportEvent::SetProjectionMode { mode } => {
            ViewportCommand::SetProjectionMode(*mode)
        }
        EditorViewportEvent::AlignView { orientation } => ViewportCommand::AlignView(*orientation),
        EditorViewportEvent::SetDisplayMode { mode } => ViewportCommand::SetDisplayMode(*mode),
        EditorViewportEvent::SetGridMode { mode } => ViewportCommand::SetGridMode(*mode),
        EditorViewportEvent::SetTranslateSnap { step } => ViewportCommand::SetTranslateSnap(*step),
        EditorViewportEvent::SetRotateSnapDegrees { step } => {
            ViewportCommand::SetRotateSnapDegrees(*step)
        }
        EditorViewportEvent::SetScaleSnap { step } => ViewportCommand::SetScaleSnap(*step),
        EditorViewportEvent::SetPreviewLighting { enabled } => {
            ViewportCommand::SetPreviewLighting(*enabled)
        }
        EditorViewportEvent::SetPreviewSkybox { enabled } => {
            ViewportCommand::SetPreviewSkybox(*enabled)
        }
        EditorViewportEvent::SetGizmosEnabled { enabled } => {
            ViewportCommand::SetGizmosEnabled(*enabled)
        }
        EditorViewportEvent::ToggleOverlayProvider { provider_id } => {
            ViewportCommand::ToggleOverlayProvider {
                provider_id: provider_id.clone(),
            }
        }
        EditorViewportEvent::FrameSelection => ViewportCommand::FrameSelection,
        EditorViewportEvent::ForView { view_id, .. } => {
            return Err(EditorViewportStateError::StaleViewportView {
                view_id: view_id.clone(),
            });
        }
    };
    let feedback = match target_view {
        Some(view_id) => shell
            .state
            .apply_viewport_command_for_view(view_id, &command)?,
        None => shell.state.apply_viewport_command(&command)?,
    };
    let structural_viewport_change = structural_viewport_event(event, &feedback);
    let chrome_projection_change = event.changes_chrome_projection();
    let changed = structural_viewport_change
        || feedback.camera_updated
        || feedback.transformed_node.is_some()
        || feedback.hovered_axis.is_some()
        || feedback.interaction_extract_stale;
    Ok(ExecutionOutcome {
        changed,
        effects: viewport_effects(
            event,
            &feedback,
            structural_viewport_change,
            chrome_projection_change,
        ),
    })
}

fn structural_viewport_event(event: &EditorViewportEvent, feedback: &ViewportFeedback) -> bool {
    feedback.settings_changed
        || matches!(
            event,
            EditorViewportEvent::LeftReleased
                | EditorViewportEvent::Resized { .. }
                | EditorViewportEvent::ActivateSceneMode { .. }
                | EditorViewportEvent::AlignView { .. }
                | EditorViewportEvent::ToggleOverlayProvider { .. }
                | EditorViewportEvent::FrameSelection
        )
}

fn viewport_effects(
    _event: &EditorViewportEvent,
    feedback: &ViewportFeedback,
    structural_viewport_change: bool,
    chrome_projection_change: bool,
) -> Vec<EditorEventEffect> {
    let render_changed = structural_viewport_change
        || feedback.camera_updated
        || feedback.transformed_node.is_some()
        || feedback.hovered_axis.is_some()
        || feedback.interaction_extract_stale;
    let presentation_changed = (structural_viewport_change && !chrome_projection_change)
        || feedback.transformed_node.is_some()
        || feedback.hovered_axis.is_some();
    let reflection_changed = structural_viewport_change || feedback.transformed_node.is_some();
    let effect_count = usize::from(render_changed)
        .saturating_add(usize::from(presentation_changed))
        .saturating_add(usize::from(reflection_changed));
    let mut effects = Vec::with_capacity(effect_count);

    if render_changed {
        effects.push(EditorEventEffect::RenderChanged);
    }

    if presentation_changed {
        effects.push(EditorEventEffect::PresentationChanged);
    }

    if reflection_changed {
        effects.push(EditorEventEffect::ReflectionChanged);
    }

    effects
}

#[cfg(test)]
#[path = "tests/viewport_event.rs"]
mod tests;
