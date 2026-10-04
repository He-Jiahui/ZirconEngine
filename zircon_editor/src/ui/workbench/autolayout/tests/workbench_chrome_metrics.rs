use super::WorkbenchChromeMetrics;
use zircon_runtime_interface::ui::design_tokens::EditorChromeTokens;

#[test]
fn default_metrics_follow_shared_workbench_chrome_tokens() {
    let metrics = WorkbenchChromeMetrics::default();
    let tokens = EditorChromeTokens::workbench_dense();

    assert_eq!(metrics.top_bar_height, tokens.top_bar_height);
    assert_eq!(metrics.host_bar_height, tokens.host_bar_height);
    assert_eq!(metrics.status_bar_height, tokens.status_bar_height);
    assert_eq!(metrics.panel_header_height, tokens.panel_header_height);
    assert_eq!(
        metrics.document_header_height,
        tokens.document_header_height
    );
    assert_eq!(
        metrics.viewport_toolbar_height,
        tokens.viewport_toolbar_height
    );
    assert_eq!(metrics.rail_width, tokens.activity_rail_width);
    assert_eq!(metrics.separator_thickness, tokens.separator_thickness);
    assert_eq!(metrics.splitter_hit_size, tokens.splitter_hit_size);
}
