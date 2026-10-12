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
