//! 尚无产品适配器时保留 UI Action 的类型化可观察性。
//! 日志只采样计数和稳定身份字段，不展开动作内容。

use zircon_runtime::diagnostic_log::write_warn;
use zircon_runtime_interface::ZrRuntimeUiActionHostRequestV1;

use super::super::RuntimeEntryApp;

pub(super) fn report_unhandled_runtime_ui_action(
    app: &mut RuntimeEntryApp,
    request: ZrRuntimeUiActionHostRequestV1,
) {
    app.unhandled_ui_action_count = app.unhandled_ui_action_count.saturating_add(1);
    if !should_report_count(app.unhandled_ui_action_count) {
        return;
    }
    write_warn(
        "runtime_ui_action",
        format!(
            "runtime_ui_action_unhandled count={} viewport={:?} surface={} tree={} node={} sequence={} action_index={} kind={} target={}",
            app.unhandled_ui_action_count,
            request.target_viewport,
            request.target_surface,
            request.tree_id.0.as_str(),
            request.target.0,
            request.input_sequence,
            request.action_index,
            if request.invocation.is_action() {
                "action"
            } else {
                "route"
            },
            request.invocation.target_id(),
        ),
    );
}

fn should_report_count(count: u64) -> bool {
    count.is_power_of_two()
}

#[cfg(test)]
#[path = "tests/ui_action.rs"]
mod tests;
