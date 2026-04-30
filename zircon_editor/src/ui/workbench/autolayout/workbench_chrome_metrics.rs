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
        Self {
            top_bar_height: 25.0,
            host_bar_height: 32.0,
            status_bar_height: 24.0,
            panel_header_height: 25.0,
            document_header_height: 31.0,
            viewport_toolbar_height: 28.0,
            rail_width: 34.0,
            separator_thickness: 1.0,
            splitter_hit_size: 8.0,
        }
    }
}
