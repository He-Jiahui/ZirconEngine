use crate::core::editor_event::{EditorEventEffect, EditorViewportEvent};
use crate::scene::viewport::ViewportFeedback;
use crate::ui::binding::ViewportCommand;
use crate::ui::host::EditorHostEventController;
use crate::ui::workbench::shell_state::WorkbenchShellStateData;
use crate::EditorIntent;

use super::execution_outcome::ExecutionOutcome;
pub(super) fn execute_viewport_event(
    controller: &EditorHostEventController,
    shell: &mut WorkbenchShellStateData,
    event: &EditorViewportEvent,
) -> Result<ExecutionOutcome, String> {
    let feedback = match event {
        EditorViewportEvent::PointerMoved { x, y } => {
            let feedback = shell
                .state
                .apply_viewport_command(&ViewportCommand::PointerMoved { x: *x, y: *y });
            if controller.gizmo_drag().is_active() && feedback.transformed_node.is_some() {
                shell.state.apply_intent(EditorIntent::DragGizmo)?;
            }
            feedback
        }
        EditorViewportEvent::LeftPressed { x, y } => {
            let feedback = shell
                .state
                .apply_viewport_command(&ViewportCommand::LeftPressed { x: *x, y: *y });
            controller
                .gizmo_drag()
                .set_active(feedback.hovered_axis.is_some());
            if controller.gizmo_drag().is_active() {
                shell.state.apply_intent(EditorIntent::BeginGizmoDrag)?;
            }
            feedback
        }
        EditorViewportEvent::LeftReleased => {
            if controller.gizmo_drag().is_active() {
                shell.state.apply_intent(EditorIntent::EndGizmoDrag)?;
            }
            controller.gizmo_drag().clear();
            shell
                .state
                .apply_viewport_command(&ViewportCommand::LeftReleased)
        }
        EditorViewportEvent::RightPressed { x, y } => shell
            .state
            .apply_viewport_command(&ViewportCommand::RightPressed { x: *x, y: *y }),
        EditorViewportEvent::RightReleased => shell
            .state
            .apply_viewport_command(&ViewportCommand::RightReleased),
        EditorViewportEvent::MiddlePressed { x, y } => shell
            .state
            .apply_viewport_command(&ViewportCommand::MiddlePressed { x: *x, y: *y }),
        EditorViewportEvent::MiddleReleased => shell
            .state
            .apply_viewport_command(&ViewportCommand::MiddleReleased),
        EditorViewportEvent::Scrolled { delta } => shell
            .state
            .apply_viewport_command(&ViewportCommand::Scrolled { delta: *delta }),
        EditorViewportEvent::Resized { width, height } => {
            shell
                .state
                .apply_viewport_command(&ViewportCommand::Resized {
                    width: *width,
                    height: *height,
                })
        }
        EditorViewportEvent::SetTool { tool } => shell
            .state
            .apply_viewport_command(&ViewportCommand::SetTool(*tool)),
        EditorViewportEvent::SetTransformSpace { space } => shell
            .state
            .apply_viewport_command(&ViewportCommand::SetTransformSpace(*space)),
        EditorViewportEvent::SetProjectionMode { mode } => shell
            .state
            .apply_viewport_command(&ViewportCommand::SetProjectionMode(*mode)),
        EditorViewportEvent::AlignView { orientation } => shell
            .state
            .apply_viewport_command(&ViewportCommand::AlignView(*orientation)),
        EditorViewportEvent::SetDisplayMode { mode } => shell
            .state
            .apply_viewport_command(&ViewportCommand::SetDisplayMode(*mode)),
        EditorViewportEvent::SetGridMode { mode } => shell
            .state
            .apply_viewport_command(&ViewportCommand::SetGridMode(*mode)),
        EditorViewportEvent::SetTranslateSnap { step } => shell
            .state
            .apply_viewport_command(&ViewportCommand::SetTranslateSnap(*step)),
        EditorViewportEvent::SetRotateSnapDegrees { step } => shell
            .state
            .apply_viewport_command(&ViewportCommand::SetRotateSnapDegrees(*step)),
        EditorViewportEvent::SetScaleSnap { step } => shell
            .state
            .apply_viewport_command(&ViewportCommand::SetScaleSnap(*step)),
        EditorViewportEvent::SetPreviewLighting { enabled } => shell
            .state
            .apply_viewport_command(&ViewportCommand::SetPreviewLighting(*enabled)),
        EditorViewportEvent::SetPreviewSkybox { enabled } => shell
            .state
            .apply_viewport_command(&ViewportCommand::SetPreviewSkybox(*enabled)),
        EditorViewportEvent::SetGizmosEnabled { enabled } => shell
            .state
            .apply_viewport_command(&ViewportCommand::SetGizmosEnabled(*enabled)),
        EditorViewportEvent::FrameSelection => shell
            .state
            .apply_viewport_command(&ViewportCommand::FrameSelection),
    };
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
            | EditorViewportEvent::SetTool { .. }
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
