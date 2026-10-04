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
