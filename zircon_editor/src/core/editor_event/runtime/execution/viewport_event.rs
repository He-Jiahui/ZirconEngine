use crate::core::editor_event::{EditorEventEffect, EditorViewportEvent};
use crate::EditorIntent;

use super::super::execution_outcome::ExecutionOutcome;
use super::super::runtime_inner::EditorEventRuntimeInner;

pub(super) fn execute_viewport_event(
    inner: &mut EditorEventRuntimeInner,
    event: &EditorViewportEvent,
) -> Result<ExecutionOutcome, String> {
    let feedback = match event {
        EditorViewportEvent::PointerMoved { x, y } => {
            let feedback = inner
                .state
                .apply_viewport_command(&crate::ui::ViewportCommand::PointerMoved { x: *x, y: *y });
            if inner.dragging_gizmo && feedback.transformed_node.is_some() {
                inner.state.apply_intent(EditorIntent::DragGizmo)?;
            }
            feedback
        }
        EditorViewportEvent::LeftPressed { x, y } => {
            let feedback = inner
                .state
                .apply_viewport_command(&crate::ui::ViewportCommand::LeftPressed { x: *x, y: *y });
            inner.dragging_gizmo = feedback.hovered_axis.is_some();
            if inner.dragging_gizmo {
                inner.state.apply_intent(EditorIntent::BeginGizmoDrag)?;
            }
            feedback
        }
        EditorViewportEvent::LeftReleased => {
            if inner.dragging_gizmo {
                inner.state.apply_intent(EditorIntent::EndGizmoDrag)?;
            }
            inner.dragging_gizmo = false;
            inner
                .state
                .apply_viewport_command(&crate::ui::ViewportCommand::LeftReleased)
        }
        EditorViewportEvent::RightPressed { x, y } => inner
            .state
            .apply_viewport_command(&crate::ui::ViewportCommand::RightPressed { x: *x, y: *y }),
        EditorViewportEvent::RightReleased => inner
            .state
            .apply_viewport_command(&crate::ui::ViewportCommand::RightReleased),
        EditorViewportEvent::MiddlePressed { x, y } => inner
            .state
            .apply_viewport_command(&crate::ui::ViewportCommand::MiddlePressed { x: *x, y: *y }),
        EditorViewportEvent::MiddleReleased => inner
            .state
            .apply_viewport_command(&crate::ui::ViewportCommand::MiddleReleased),
        EditorViewportEvent::Scrolled { delta } => inner
            .state
            .apply_viewport_command(&crate::ui::ViewportCommand::Scrolled { delta: *delta }),
        EditorViewportEvent::Resized { width, height } => {
            inner
                .state
                .apply_viewport_command(&crate::ui::ViewportCommand::Resized {
                    width: *width,
                    height: *height,
                })
        }
        EditorViewportEvent::SetTool { tool } => inner
            .state
            .apply_viewport_command(&crate::ui::ViewportCommand::SetTool(*tool)),
        EditorViewportEvent::SetTransformSpace { space } => inner
            .state
            .apply_viewport_command(&crate::ui::ViewportCommand::SetTransformSpace(*space)),
        EditorViewportEvent::SetProjectionMode { mode } => inner
            .state
            .apply_viewport_command(&crate::ui::ViewportCommand::SetProjectionMode(*mode)),
        EditorViewportEvent::AlignView { orientation } => inner
            .state
            .apply_viewport_command(&crate::ui::ViewportCommand::AlignView(*orientation)),
        EditorViewportEvent::SetDisplayMode { mode } => inner
            .state
            .apply_viewport_command(&crate::ui::ViewportCommand::SetDisplayMode(*mode)),
        EditorViewportEvent::SetGridMode { mode } => inner
            .state
            .apply_viewport_command(&crate::ui::ViewportCommand::SetGridMode(*mode)),
        EditorViewportEvent::SetTranslateSnap { step } => inner
            .state
            .apply_viewport_command(&crate::ui::ViewportCommand::SetTranslateSnap(*step)),
        EditorViewportEvent::SetRotateSnapDegrees { step } => inner
            .state
            .apply_viewport_command(&crate::ui::ViewportCommand::SetRotateSnapDegrees(*step)),
        EditorViewportEvent::SetScaleSnap { step } => inner
            .state
            .apply_viewport_command(&crate::ui::ViewportCommand::SetScaleSnap(*step)),
        EditorViewportEvent::SetPreviewLighting { enabled } => inner
            .state
            .apply_viewport_command(&crate::ui::ViewportCommand::SetPreviewLighting(*enabled)),
        EditorViewportEvent::SetPreviewSkybox { enabled } => inner
            .state
            .apply_viewport_command(&crate::ui::ViewportCommand::SetPreviewSkybox(*enabled)),
        EditorViewportEvent::SetGizmosEnabled { enabled } => inner
            .state
            .apply_viewport_command(&crate::ui::ViewportCommand::SetGizmosEnabled(*enabled)),
        EditorViewportEvent::FrameSelection => inner
            .state
            .apply_viewport_command(&crate::ui::ViewportCommand::FrameSelection),
    };
    Ok(ExecutionOutcome {
        changed: matches!(
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
        ) || feedback.camera_updated
            || feedback.transformed_node.is_some()
            || feedback.hovered_axis.is_some(),
        effects: vec![
            EditorEventEffect::RenderChanged,
            EditorEventEffect::PresentationChanged,
            EditorEventEffect::ReflectionChanged,
        ],
    })
}
