//! 尚无产品适配器时记录通用 UI 宿主请求的有限诊断。
//! 只报告类型和静态身份，避免动态内容进入日志。

use zircon_runtime::diagnostic_log::write_warn;
use zircon_runtime_interface::ZrRuntimeUiHostRequestV1;

use super::super::RuntimeEntryApp;

pub(super) fn report_unhandled_runtime_ui_host_request(
    app: &mut RuntimeEntryApp,
    request: ZrRuntimeUiHostRequestV1,
) {
    app.unhandled_ui_host_request_count = app.unhandled_ui_host_request_count.saturating_add(1);
    if !should_report_count(app.unhandled_ui_host_request_count) {
        return;
    }
    write_warn(
        "runtime_ui_host_request",
        format!(
            "runtime_ui_host_request_unhandled count={} viewport={:?} surface={} sequence={} request_index={} effect_index={} kind={}",
            app.unhandled_ui_host_request_count,
            request.target_viewport,
            request.target_surface,
            request.input_sequence,
            request.request_index,
            request.effect_index,
            request.kind.as_str(),
        ),
    );
}

fn should_report_count(count: u64) -> bool {
    count.is_power_of_two()
}

#[cfg(test)]
#[path = "tests/ui_host_request.rs"]
mod tests;

/// Typed adapter shared by production and the bounded platform-call regression.
fn apply_ui_cursor_with(
    kind: &zircon_runtime_interface::ZrRuntimeUiHostRequestKindV1,
    mut set_cursor: impl FnMut(winit::cursor::CursorIcon),
) -> bool {
    use winit::cursor::CursorIcon;
    use zircon_runtime_interface::{ui::tree::UiCursor, ZrRuntimeUiHostRequestKindV1};
    let ZrRuntimeUiHostRequestKindV1::Cursor(cursor) = kind else {
        return false;
    };
    let icon = match cursor {
        UiCursor::Default => CursorIcon::Default,
        UiCursor::Pointer => CursorIcon::Pointer,
        UiCursor::Text => CursorIcon::Text,
        UiCursor::ResizeEw => CursorIcon::EwResize,
        UiCursor::ResizeNs => CursorIcon::NsResize,
        UiCursor::Grab => CursorIcon::Grab,
        UiCursor::Grabbing => CursorIcon::Grabbing,
    };
    set_cursor(icon);
    true
}

pub(super) fn apply_runtime_ui_cursor_host_request(
    window: &dyn winit::window::Window,
    kind: &zircon_runtime_interface::ZrRuntimeUiHostRequestKindV1,
) -> bool {
    apply_ui_cursor_with(kind, |icon| window.set_cursor(icon.into()))
}
