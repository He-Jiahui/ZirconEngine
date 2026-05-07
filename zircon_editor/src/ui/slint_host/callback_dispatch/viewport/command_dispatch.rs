use crate::ui::binding::ViewportCommand;

use crate::core::editor_event::{
    EditorEvent, EditorEventEnvelope, EditorEventRuntime, EditorEventSource, EditorViewportEvent,
};
use crate::ui::slint_host::event_bridge::UiHostEventEffects;

use super::super::common::dispatch_envelope;

pub(crate) fn dispatch_viewport_event(
    runtime: &EditorEventRuntime,
    event: EditorViewportEvent,
) -> Result<UiHostEventEffects, String> {
    dispatch_envelope(
        runtime,
        EditorEventEnvelope::new(EditorEventSource::Slint, EditorEvent::Viewport(event)),
    )
}

#[cfg(test)]
#[cfg(test)]
pub(crate) fn dispatch_viewport_command(
    runtime: &EditorEventRuntime,
    command: ViewportCommand,
) -> Result<UiHostEventEffects, String> {
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
