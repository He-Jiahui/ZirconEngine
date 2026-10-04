use super::*;
use crate::ui::retained_host::host_contract::paint_theme::METRICS;

#[test]
fn fallback_text_metrics_project_from_host_control_metrics() {
    assert_eq!(
        HostPaintCommand::fallback_text_metrics_from_host(METRICS),
        (METRICS.font_body, METRICS.line_height(METRICS.font_body))
    );
}
