use zircon_editor_ui::ViewportCommand;

use crate::editor_event::{host_adapter, EditorEventRuntime, EditorViewportEvent};
use crate::host::slint_host::event_bridge::SlintDispatchEffects;

use super::super::common::dispatch_envelope;

pub(crate) fn dispatch_viewport_event(
    runtime: &EditorEventRuntime,
    event: EditorViewportEvent,
) -> Result<SlintDispatchEffects, String> {
    dispatch_envelope(runtime, host_adapter::slint_viewport(event))
}

#[cfg(test)]
#[cfg(test)]
pub(crate) fn dispatch_viewport_command(
    runtime: &EditorEventRuntime,
    command: ViewportCommand,
) -> Result<SlintDispatchEffects, String> {
    dispatch_viewport_event(runtime, viewport_event_from_command(command))
}

pub(crate) fn viewport_event_from_command(command: ViewportCommand) -> EditorViewportEvent {
    match command {
        ViewportCommand::PointerMoved { x, y } => EditorViewportEvent::PointerMoved { x, y },
        ViewportCommand::LeftPressed { x, y } => EditorViewportEvent::LeftPressed { x, y },
        ViewportCommand::LeftReleased => EditorViewportEvent::LeftReleased,
        ViewportCommand::RightPressed { x, y } => EditorViewportEvent::RightPressed { x, y },
        ViewportCommand::RightReleased => EditorViewportEvent::RightReleased,
        ViewportCommand::MiddlePressed { x, y } => EditorViewportEvent::MiddlePressed { x, y },
        ViewportCommand::MiddleReleased => EditorViewportEvent::MiddleReleased,
        ViewportCommand::Scrolled { delta } => EditorViewportEvent::Scrolled { delta },
        ViewportCommand::Resized { width, height } => {
            EditorViewportEvent::Resized { width, height }
        }
        ViewportCommand::SetTool(tool) => EditorViewportEvent::SetTool { tool },
        ViewportCommand::SetTransformSpace(space) => {
            EditorViewportEvent::SetTransformSpace { space }
        }
        ViewportCommand::SetProjectionMode(mode) => EditorViewportEvent::SetProjectionMode { mode },
        ViewportCommand::AlignView(orientation) => EditorViewportEvent::AlignView { orientation },
        ViewportCommand::SetDisplayMode(mode) => EditorViewportEvent::SetDisplayMode { mode },
        ViewportCommand::SetGridMode(mode) => EditorViewportEvent::SetGridMode { mode },
        ViewportCommand::SetTranslateSnap(step) => EditorViewportEvent::SetTranslateSnap { step },
        ViewportCommand::SetRotateSnapDegrees(step) => {
            EditorViewportEvent::SetRotateSnapDegrees { step }
        }
        ViewportCommand::SetScaleSnap(step) => EditorViewportEvent::SetScaleSnap { step },
        ViewportCommand::SetPreviewLighting(enabled) => {
            EditorViewportEvent::SetPreviewLighting { enabled }
        }
        ViewportCommand::SetPreviewSkybox(enabled) => {
            EditorViewportEvent::SetPreviewSkybox { enabled }
        }
        ViewportCommand::SetGizmosEnabled(enabled) => {
            EditorViewportEvent::SetGizmosEnabled { enabled }
        }
        ViewportCommand::FrameSelection => EditorViewportEvent::FrameSelection,
    }
}
