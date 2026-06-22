use crate::ui::retained_host::primitives::Color;

pub(in super::super) struct ProjectedTextLayout {
    pub(in super::super) text: String,
    pub(in super::super) label_text: String,
    pub(in super::super) label_color: Color,
    pub(in super::super) label_brightness: f32,
    pub(in super::super) layout_offset_x: f32,
    pub(in super::super) layout_offset_y: f32,
    pub(in super::super) layout_icon_size: f32,
    pub(in super::super) layout_content_offset_x: f32,
    pub(in super::super) layout_content_offset_y: f32,
    pub(in super::super) layout_first_cell_offset_x: f32,
    pub(in super::super) layout_second_cell_offset_x: f32,
    pub(in super::super) layout_third_cell_offset_x: f32,
    pub(in super::super) layout_fourth_cell_offset_x: f32,
    pub(in super::super) selected_segment_border_width: Option<f64>,
    pub(in super::super) selected_segment_underline_height: f32,
    pub(in super::super) selected_segment_underline_color: Color,
    pub(in super::super) font_size: f32,
    pub(in super::super) font_weight: i32,
    pub(in super::super) text_align: String,
    pub(in super::super) overflow: String,
}
