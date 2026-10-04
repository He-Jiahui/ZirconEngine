use zircon_runtime_interface::ui::design_tokens::EditorChromeTokens;

/// Design-token chrome extents in logical layout units.
///
/// The shell solves with these values in logical units. Render assembly applies
/// DPI conversion once, so callers must not pre-scale individual fields.
#[derive(Clone, Copy, Debug, PartialEq)]
/// 壳chrome的logical度量快照；默认沿共享editor令牌，所有解算阶段共用后统一物理缩放。
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
#[path = "tests/workbench_chrome_metrics.rs"]
mod tests;
