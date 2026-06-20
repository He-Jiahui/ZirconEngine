#[derive(Clone, Default)]
pub(crate) struct HostWindowSurfaceMetricsData {
    pub outer_margin_px: f32,
    pub rail_width_px: f32,
    pub top_bar_height_px: f32,
    pub host_bar_height_px: f32,
    pub panel_header_height_px: f32,
    pub document_header_height_px: f32,
}

#[derive(Clone, Default)]
pub(crate) struct HostWindowSurfaceOrchestrationData {
    pub left_rail_width_px: f32,
    pub right_rail_width_px: f32,
    pub left_stack_width_px: f32,
    pub right_stack_width_px: f32,
    pub left_panel_width_px: f32,
    pub right_panel_width_px: f32,
    pub bottom_panel_height_px: f32,
    pub main_content_y_px: f32,
    pub document_zone_x_px: f32,
    pub right_stack_x_px: f32,
    pub bottom_panel_y_px: f32,
}
