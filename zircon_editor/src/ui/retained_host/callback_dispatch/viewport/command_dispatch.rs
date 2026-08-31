use crate::ui::binding::ViewportCommand;

use crate::core::editor_event::{
    EditorEvent, EditorEventEnvelope, EditorEventSource, EditorViewportEvent,
};
use crate::ui::host::EditorHostEventController;
use crate::ui::retained_host::event_bridge::UiHostEventEffects;

use super::super::common::dispatch_envelope;

pub(crate) fn dispatch_viewport_event(
    runtime: &EditorHostEventController,
    event: EditorViewportEvent,
) -> Result<UiHostEventEffects, String> {
    dispatch_envelope(
        runtime,
        EditorEventEnvelope::new(
            EditorEventSource::RetainedHost,
            EditorEvent::Viewport(event),
        ),
    )
}

#[cfg(test)]
pub(crate) fn dispatch_viewport_command(
    runtime: &EditorHostEventController,
    command: ViewportCommand,
) -> Result<UiHostEventEffects, String> {
    dispatch_viewport_event(runtime, viewport_event_from_command(command))
}

pub(crate) fn viewport_event_from_command(command: ViewportCommand) -> EditorViewportEvent {
    match command {
        ViewportCommand::PointerMoved { x, y } => EditorViewportEvent::PointerMoved { x, y },
        ViewportCommand::LeftPressed {
            x,
            y,
            selection_mutation,
        } => EditorViewportEvent::LeftPressed {
            x,
            y,
            selection_mutation,
        },
        ViewportCommand::LeftReleased => EditorViewportEvent::LeftReleased,
        ViewportCommand::CancelInteraction => EditorViewportEvent::CancelInteraction,
        ViewportCommand::RightPressed { x, y } => EditorViewportEvent::RightPressed { x, y },
        ViewportCommand::RightReleased => EditorViewportEvent::RightReleased,
        ViewportCommand::MiddlePressed { x, y } => EditorViewportEvent::MiddlePressed { x, y },
        ViewportCommand::MiddleReleased => EditorViewportEvent::MiddleReleased,
        ViewportCommand::Scrolled { delta } => EditorViewportEvent::Scrolled { delta },
        ViewportCommand::Resized { width, height } => {
            EditorViewportEvent::Resized { width, height }
        }
        ViewportCommand::ActivateSceneMode(mode) => EditorViewportEvent::ActivateSceneMode { mode },
        ViewportCommand::SetTransformSpace(space) => {
            EditorViewportEvent::SetTransformSpace { space }
        }
        ViewportCommand::SetPivotMode(mode) => EditorViewportEvent::SetPivotMode { mode },
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
        ViewportCommand::ToggleOverlayProvider { provider_id } => {
            EditorViewportEvent::ToggleOverlayProvider { provider_id }
        }
        ViewportCommand::FrameSelection => EditorViewportEvent::FrameSelection,
    }
}

#[cfg(test)]
mod tests {
    use super::viewport_event_from_command;
    use crate::core::editor_event::EditorViewportEvent;
    use crate::ui::binding::ViewportCommand;

    #[test]
    fn overlay_provider_toggle_preserves_its_registered_id() {
        assert_eq!(
            viewport_event_from_command(ViewportCommand::ToggleOverlayProvider {
                provider_id: "weather.viewport.overlay.provider".to_string(),
            }),
            EditorViewportEvent::ToggleOverlayProvider {
                provider_id: "weather.viewport.overlay.provider".to_string(),
            }
        );
    }
}
