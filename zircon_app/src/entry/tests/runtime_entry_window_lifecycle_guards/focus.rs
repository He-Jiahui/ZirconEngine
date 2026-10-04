//! 验证焦点变化先更新 cadence 再发前后台生命周期事件。
//! 该守卫约束源级接线，仍需结合被调用实现理解运行时契约。

use super::super::source_assertions::assert_source_order;
use super::sources::{runtime_window_events_source, runtime_window_lifecycle_source};

// TODO: [CR-APP-ENTRY-0010] 确认焦点守卫与正在修改的 focus helper 分工；当前外部基线缺少旧内联状态构造，需待外来改动稳定后复核功能和受管测试。
#[test]
fn runtime_entry_translates_focus_changes_to_lifecycle_events() {
    let runtime_window_events_source = runtime_window_events_source();
    let runtime_window_lifecycle_source = runtime_window_lifecycle_source();

    assert_source_order(
        runtime_window_events_source.as_str(),
        &[
            "WindowEvent::Focused(focused)",
            "self.handle_window_focus_changed(event_loop, focused);",
        ],
        "runtime entry should delegate focus lifecycle forwarding to the window lifecycle module",
    );
    assert_source_order(
        runtime_window_lifecycle_source.as_str(),
        &[
            "fn handle_window_focus_changed",
            "if self.frame_cadence.set_window_focused(focused)",
            "self.request_runtime_frame();",
            "let state = if focused",
            "ZR_RUNTIME_LIFECYCLE_STATE_FOREGROUND_V1",
            "ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1",
            "ZrRuntimeEventV1::lifecycle",
            "self.dispatch_runtime_event(event_loop, event);",
        ],
        "runtime entry should translate focus changes into runtime foreground/background lifecycle events",
    );
}
