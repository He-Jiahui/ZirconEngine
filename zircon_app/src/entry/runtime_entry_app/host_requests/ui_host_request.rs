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
mod tests {
    use super::should_report_count;

    #[test]
    fn unhandled_ui_host_request_diagnostics_are_logarithmically_bounded() {
        assert!(should_report_count(1));
        assert!(should_report_count(2));
        assert!(!should_report_count(3));
        assert!(should_report_count(4));
    }

    #[test]
    fn diagnostic_source_never_formats_dynamic_host_request_content() {
        let source = include_str!("ui_host_request.rs");

        assert!(!source.contains(concat!("request.kind.", "href")));
        assert!(!source.contains(concat!("request.kind.", "popup_id")));
        assert!(!source.contains(concat!("request.kind.", "tooltip_id")));
        assert!(!source.contains(concat!("request.", "tree_id")));
    }
}
