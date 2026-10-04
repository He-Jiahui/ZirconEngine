use zircon_runtime_interface::ui::dispatch::{
    UiDispatchReply, UiInputDiagnosticsMode, UiInputDispatchResult, UiInputEvent,
    UiInputRoutePolicy, UiMouseMotionInputEvent,
};

use super::super::surface::UiSurface;

/// 保留原始相对鼠标运动回执给宿主消费；此事件本身没有 UI 命中路由。
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
#[path = "tests/mouse_motion.rs"]
mod tests;
