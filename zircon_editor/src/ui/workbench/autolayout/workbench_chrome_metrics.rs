use zircon_runtime_interface::ui::design_tokens::EditorChromeTokens;

/// Design-token chrome extents in logical layout units.
///
/// The shell solves with these values in logical units. Render assembly applies
/// DPI conversion once, so callers must not pre-scale individual fields.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorkbenchChromeMetrics {
    pub top_bar_height: f32,
    pub host_bar_height: f32,
    pub status_bar_height: f32,
    pub panel_header_height: f32,
    pub document_header_height: f32,
    pub viewport_toolbar_height: f32,
    pub rail_width: f32,
    pub separator_thickness: f32,
    pub splitter_hit_size: f32,
}

impl Default for WorkbenchChromeMetrics {
    fn default() -> Self {
        Self::from(EditorChromeTokens::workbench_dense())
    }
}

impl From<EditorChromeTokens> for WorkbenchChromeMetrics {
    fn from(tokens: EditorChromeTokens) -> Self {
        Self {
            top_bar_height: tokens.top_bar_height,
            host_bar_height: tokens.host_bar_height,
            status_bar_height: tokens.status_bar_height,
            panel_header_height: tokens.panel_header_height,
            document_header_height: tokens.document_header_height,
            viewport_toolbar_height: tokens.viewport_toolbar_height,
            rail_width: tokens.activity_rail_width,
            separator_thickness: tokens.separator_thickness,
            splitter_hit_size: tokens.splitter_hit_size,
        }
    }
}

#[cfg(test)]
mod tests {
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
}
