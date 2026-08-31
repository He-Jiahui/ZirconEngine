use zircon_runtime_interface::ui::dispatch::{
    UiDispatchReply, UiInputDiagnosticsMode, UiInputDispatchResult, UiInputEvent,
    UiInputRoutePolicy, UiMouseMotionInputEvent,
};

use super::super::surface::UiSurface;

pub(super) fn dispatch_mouse_motion_input(
    _surface: &UiSurface,
    motion: UiMouseMotionInputEvent,
    diagnostics_mode: UiInputDiagnosticsMode,
) -> UiInputDispatchResult {
    let mut result = UiInputDispatchResult::new(
        UiInputEvent::MouseMotion(motion),
        UiDispatchReply::unhandled(),
    );
    result.diagnostics.route_policy = UiInputRoutePolicy::Unrouted;
    if diagnostics_mode.captures_full_trace() {
        result
            .diagnostics
            .notes
            .push("raw_mouse_motion".to_string());
    }
    result
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::ui::{
        dispatch::{UiInputDiagnosticsMode, UiInputEventMetadata, UiMouseMotionInputEvent},
        event_ui::UiTreeId,
    };

    use crate::ui::surface::UiSurface;

    use super::dispatch_mouse_motion_input;

    #[test]
    fn summary_mouse_motion_preserves_reply_without_materializing_note() {
        let surface = UiSurface::new(UiTreeId::new("runtime.raw-mouse-motion"));
        let motion = UiMouseMotionInputEvent {
            metadata: UiInputEventMetadata::default(),
            delta_x: 2.0,
            delta_y: -1.0,
        };

        let summary =
            dispatch_mouse_motion_input(&surface, motion.clone(), UiInputDiagnosticsMode::Summary);
        let full = dispatch_mouse_motion_input(&surface, motion, UiInputDiagnosticsMode::Full);

        assert_eq!(summary.reply, full.reply);
        assert!(summary.diagnostics.notes.is_empty());
        assert_eq!(
            full.diagnostics
                .notes
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            ["raw_mouse_motion"]
        );
    }
}
