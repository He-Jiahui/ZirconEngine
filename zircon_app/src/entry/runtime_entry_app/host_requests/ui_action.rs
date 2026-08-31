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
mod tests {
    use super::should_report_count;

    #[test]
    fn unhandled_action_diagnostics_are_logarithmically_bounded() {
        assert!(should_report_count(1));
        assert!(should_report_count(2));
        assert!(!should_report_count(3));
        assert!(should_report_count(4));
        assert!(!should_report_count(usize::MAX as u64));
    }

    #[test]
    fn diagnostic_source_never_formats_action_payload_or_secure_reference() {
        let source = include_str!("ui_action.rs");

        assert!(!source.contains(concat!("request.invocation.", "payload")));
        assert!(!source.contains(concat!("request.", "secure_value")));
    }
}
