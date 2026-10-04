//! 验证先向动态 Runtime 通知 close 请求，成功后再根据宿主关闭策略销毁表面和选择退出。
//! 该守卫约束源级接线，仍需结合被调用实现理解运行时契约。

use super::super::source_assertions::assert_source_order;
use super::sources::{runtime_window_events_source, runtime_window_lifecycle_source};

#[test]
fn runtime_entry_notifies_runtime_before_applying_close_policy() {
    let runtime_window_events_source = runtime_window_events_source();
    let runtime_window_lifecycle_source = runtime_window_lifecycle_source();

    assert_source_order(
        runtime_window_events_source.as_str(),
        &[
            "WindowEvent::CloseRequested",
            "self.handle_window_close_requested(event_loop);",
        ],
        "runtime entry should delegate close-request policy handling to the window lifecycle module",
    );
    assert_source_order(
        runtime_window_lifecycle_source.as_str(),
        &[
            "fn handle_window_close_requested",
            "ZrRuntimeEventV1::window_close_requested",
            "if !self.dispatch_runtime_event(event_loop, event)",
            "return;",
            "self.window_lifecycle_policy.should_close_on_request()",
            "let surface_release = self.application_lifecycle.destroy_surfaces();",
            "let teardown_failed = !self.finish_surface_release(surface_release);",
            "if teardown_failed",
            ".should_exit_after_primary_close()",
            "event_loop.exit();",
        ],
        "runtime entry should notify the runtime about close requests before applying the configurable close policy",
    );
}
