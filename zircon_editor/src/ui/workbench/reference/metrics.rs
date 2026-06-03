use zircon_runtime_interface::ui::layout::UiSize;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EditorWorkbenchReferenceMetrics {
    pub target_width: f32,
    pub target_height: f32,
    pub top_bar_height: f32,
    pub upper_region_height: f32,
    pub status_bar_height: f32,
    pub activity_rail_width: f32,
    pub hierarchy_panel_width: f32,
    pub inspector_panel_width: f32,
    pub panel_header_height: f32,
    pub toolbar_height: f32,
    pub control_height: f32,
    pub compact_row_height: f32,
    pub section_gap: f32,
}

impl EditorWorkbenchReferenceMetrics {
    pub fn target_size(self) -> UiSize {
        UiSize::new(self.target_width, self.target_height)
    }

    pub fn component_gallery_height(self) -> f32 {
        (self.target_height
            - self.top_bar_height
            - self.upper_region_height
            - self.status_bar_height)
            .max(0.0)
    }

    pub fn viewport_width(self) -> f32 {
        (self.target_width
            - self.activity_rail_width
            - self.hierarchy_panel_width
            - self.inspector_panel_width)
            .max(0.0)
    }
}

impl Default for EditorWorkbenchReferenceMetrics {
    fn default() -> Self {
        Self {
            target_width: 1672.0,
            target_height: 941.0,
            top_bar_height: 60.0,
            upper_region_height: 428.0,
            status_bar_height: 46.0,
            activity_rail_width: 72.0,
            hierarchy_panel_width: 332.0,
            inspector_panel_width: 404.0,
            panel_header_height: 50.0,
            toolbar_height: 44.0,
            control_height: 30.0,
            compact_row_height: 28.0,
            section_gap: 12.0,
        }
    }
}
