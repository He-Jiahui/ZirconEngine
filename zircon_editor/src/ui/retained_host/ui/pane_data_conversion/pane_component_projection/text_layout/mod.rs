mod attributes;
mod label;
mod model;
mod offsets;
mod selected_segment;
mod text;
mod typography;

use std::collections::BTreeMap;

use self::label::projected_label_fields;
pub(super) use self::model::ProjectedTextLayout;
use self::offsets::projected_layout_offsets;
use self::selected_segment::projected_selected_segment;
use self::text::projected_text;
use self::typography::projected_typography;
use super::drag_overlay::ProjectedDragOverlayData;

pub(super) fn projected_text_layout(
    control_id: &str,
    component_role: &str,
    attributes: &BTreeMap<String, toml::Value>,
    has_bindings: bool,
    drag_overlay: &ProjectedDragOverlayData,
) -> ProjectedTextLayout {
    let label = projected_label_fields(attributes);
    let offsets = projected_layout_offsets(attributes);
    let selected_segment = projected_selected_segment(attributes);
    let typography = projected_typography(attributes, component_role);

    ProjectedTextLayout {
        text: projected_text(
            control_id,
            component_role,
            attributes,
            has_bindings,
            drag_overlay,
        ),
        label_text: label.label_text,
        label_color: label.label_color,
        label_brightness: label.label_brightness,
        layout_offset_x: offsets.layout_offset_x,
        layout_offset_y: offsets.layout_offset_y,
        layout_icon_size: offsets.layout_icon_size,
        layout_content_offset_x: offsets.layout_content_offset_x,
        layout_content_offset_y: offsets.layout_content_offset_y,
        layout_first_cell_offset_x: offsets.layout_first_cell_offset_x,
        layout_second_cell_offset_x: offsets.layout_second_cell_offset_x,
        layout_third_cell_offset_x: offsets.layout_third_cell_offset_x,
        layout_fourth_cell_offset_x: offsets.layout_fourth_cell_offset_x,
        selected_segment_border_width: selected_segment.border_width,
        selected_segment_underline_height: selected_segment.underline_height,
        selected_segment_underline_color: selected_segment.underline_color,
        font_size: typography.font_size,
        font_weight: typography.font_weight,
        text_align: typography.text_align,
        overflow: typography.overflow,
    }
}
