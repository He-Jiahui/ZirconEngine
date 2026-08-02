use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host as host_contract;

use super::super::text_layout::ProjectedTextLayout;
use super::super::value_media::ProjectedValueMedia;

pub(super) fn assign_content_fields(
    node: &mut host_contract::TemplatePaneNodeData,
    text_layout: ProjectedTextLayout,
    value_media: ProjectedValueMedia,
) {
    node.text = text_layout.text.into();
    node.label_text = text_layout.label_text.into();
    node.label_color = text_layout.label_color;
    node.label_brightness = text_layout.label_brightness;
    node.layout_offset_x = text_layout.layout_offset_x;
    node.layout_offset_y = text_layout.layout_offset_y;
    node.layout_icon_size = text_layout.layout_icon_size;
    node.layout_content_offset_x = text_layout.layout_content_offset_x;
    node.layout_content_offset_y = text_layout.layout_content_offset_y;
    node.layout_first_cell_offset_x = text_layout.layout_first_cell_offset_x;
    node.layout_second_cell_offset_x = text_layout.layout_second_cell_offset_x;
    node.layout_third_cell_offset_x = text_layout.layout_third_cell_offset_x;
    node.layout_fourth_cell_offset_x = text_layout.layout_fourth_cell_offset_x;
    node.icon_placement = text_layout.icon_placement.into();
    node.has_selected_segment_border_width = text_layout.selected_segment_border_width.is_some();
    node.selected_segment_border_width =
        text_layout.selected_segment_border_width.unwrap_or(0.0) as f32;
    node.selected_segment_underline_height = text_layout.selected_segment_underline_height;
    node.selected_segment_underline_color = text_layout.selected_segment_underline_color;
    node.font_size = text_layout.font_size;
    node.font_weight = text_layout.font_weight;
    node.text_align = text_layout.text_align.into();
    node.overflow = text_layout.overflow.into();

    node.value_text = value_media.value_text.into();
    node.value_number = value_media.value_number as f32;
    node.value_percent = value_media.value_percent;
    node.value_color = value_media.value_color;
    node.media_source = value_media.media_source.into();
    node.icon_name = value_media.icon_name.into();
    node.has_preview_image = value_media.has_preview_image;
    node.preview_image = value_media.preview_image;
    node.vector_components = model_rc(value_media.vector_components);
}
