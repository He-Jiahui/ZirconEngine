use crate::core::editor_event::{EditorEventEffect, EditorViewportEvent};
use crate::scene::viewport::ViewportFeedback;
use crate::ui::binding::ViewportCommand;
use crate::ui::host::EditorHostEventController;
use crate::ui::workbench::shell_state::WorkbenchShellStateData;

use super::execution_outcome::ExecutionOutcome;

pub(super) fn execute_viewport_event(
    _controller: &EditorHostEventController,
    shell: &mut WorkbenchShellStateData,
    event: &EditorViewportEvent,
) -> Result<ExecutionOutcome, String> {
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
    };
    let feedback = shell.state.apply_viewport_command(&command)?;
    let structural_viewport_change = structural_viewport_event(event);
    let changed = structural_viewport_change
        || feedback.camera_updated
        || feedback.transformed_node.is_some()
        || feedback.hovered_axis.is_some();
    Ok(ExecutionOutcome {
        changed,
        effects: viewport_effects(event, &feedback, structural_viewport_change),
    })
}

fn structural_viewport_event(event: &EditorViewportEvent) -> bool {
    matches!(
        event,
        EditorViewportEvent::LeftReleased
            | EditorViewportEvent::Resized { .. }
            | EditorViewportEvent::ActivateSceneMode { .. }
            | EditorViewportEvent::SetTransformSpace { .. }
            | EditorViewportEvent::SetProjectionMode { .. }
            | EditorViewportEvent::AlignView { .. }
            | EditorViewportEvent::SetDisplayMode { .. }
            | EditorViewportEvent::SetGridMode { .. }
            | EditorViewportEvent::SetTranslateSnap { .. }
            | EditorViewportEvent::SetRotateSnapDegrees { .. }
            | EditorViewportEvent::SetScaleSnap { .. }
            | EditorViewportEvent::SetPreviewLighting { .. }
            | EditorViewportEvent::SetPreviewSkybox { .. }
            | EditorViewportEvent::SetGizmosEnabled { .. }
            | EditorViewportEvent::ToggleOverlayProvider { .. }
            | EditorViewportEvent::FrameSelection
    )
}

fn viewport_effects(
    _event: &EditorViewportEvent,
    feedback: &ViewportFeedback,
    structural_viewport_change: bool,
) -> Vec<EditorEventEffect> {
    let mut effects = Vec::new();

    if structural_viewport_change
        || feedback.camera_updated
        || feedback.transformed_node.is_some()
        || feedback.hovered_axis.is_some()
    {
        effects.push(EditorEventEffect::RenderChanged);
    }

    if structural_viewport_change
        || feedback.transformed_node.is_some()
        || feedback.hovered_axis.is_some()
    {
        effects.push(EditorEventEffect::PresentationChanged);
    }

    if structural_viewport_change || feedback.transformed_node.is_some() {
        effects.push(EditorEventEffect::ReflectionChanged);
    }

    effects
}
