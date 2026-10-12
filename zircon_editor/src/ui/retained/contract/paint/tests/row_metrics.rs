use super::super::super::paint_theme::{METRICS, PALETTE};
use super::*;

#[test]
fn row_metrics_project_from_host_control_metrics() {
    let mut host = METRICS;
    host.row_height = 30.0;
    host.font_body = 11.0;
    host.font_large = 15.0;
    host.line_height_ratio = 1.4;
    host.radius_control = 5.0;
    host.border_width = 1.5;
    host.gap_s = 5.0;
    host.gap_m = 9.0;
    host.gap_l = 13.0;
    host.button_pad_x = 14.0;
    host.button_chevron_reserve = 21.0;
    host.selection_indicator_width = 3.0;
    host.input_pad = [9.0, 9.0, 4.0, 5.0];

    let metrics = row_metrics_from_host(host);

    assert_eq!(metrics.row_height, 30.0);
    assert_eq!(metrics.text_font_size, 11.0);
    assert!((metrics.text_line_height - 15.4).abs() < f32::EPSILON);
    assert_eq!(metrics.surface_radius, 5.0);
    assert_eq!(metrics.border_width, 1.5);
    assert_eq!(metrics.text_inset_x, 9.0);
    assert_eq!(metrics.text_inset_y, 5.0);
    assert_eq!(metrics.right_reserve, 30.0);
    assert_eq!(metrics.list_adornment_size, 17.0);
    assert_eq!(metrics.list_adornment_right_inset, 13.0);
    assert_eq!(metrics.selection_indicator_width, 3.0);
    assert_eq!(metrics.tree_base_inset_x, 14.0);
    assert_eq!(metrics.tree_disclosure_size, 13.0);
    assert_eq!(metrics.tree_icon_size, 17.0);
    assert_eq!(metrics.tree_text_gap, 8.0);
    assert_eq!(metrics.tree_action_size, 18.0);
    assert_eq!(metrics.tree_action_button_size, 23.0);
    assert_eq!(metrics.tree_guide_width, 1.5);
    assert_eq!(metrics.tree_guide_vertical_extension, 1.5);
    assert_eq!(metrics.tree_guide_opacity, 0.78);
    assert_eq!(metrics.tree_guide_step, 21.0);
    assert_eq!(metrics.tree_guide_offset_x, 6.5);
    assert_eq!(metrics.property_label_width, 105.0);
    assert_eq!(metrics.component_property_label_width, 115.0);
    assert_eq!(metrics.property_label_min_width, 60.0);
    assert_eq!(metrics.property_text_inset_x, 6.5);
    assert_eq!(metrics.property_axis_width, 13.0);
    assert_eq!(metrics.property_field_inset_y, 4.0);
    assert_eq!(metrics.property_field_radius, 5.0);
    assert_eq!(metrics.property_field_border_width, 1.5);
}

#[test]
fn row_palette_projects_from_host_material_palette() {
    let mut host = PALETTE;
    host.accent = [1, 2, 3, 4];
    host.track = [5, 6, 7, 8];
    host.surface_hover = [9, 10, 11, 12];
    host.border = [13, 14, 15, 16];
    host.text_disabled = [17, 18, 19, 20];
    host.focus_ring = [21, 22, 23, 24];
    host.text_muted = [25, 26, 27, 28];
    host.text = [29, 30, 31, 32];
    host.surface_inset = [33, 34, 35, 36];

    let palette = row_palette_from_host(host);

    assert_eq!(palette.selection_indicator, [1, 2, 3, 4]);
    assert_eq!(palette.tree_guide, [5, 6, 7, 8]);
    assert_eq!(palette.tree_action_slot_surface, [9, 10, 11, 12]);
    assert_eq!(palette.tree_action_slot_border, [13, 14, 15, 16]);
    assert_eq!(palette.disabled_adornment_tint, [17, 18, 19, 20]);
    assert_eq!(palette.property_field_surface, [33, 34, 35, 36]);
    assert_eq!(palette.property_field_border, [13, 14, 15, 16]);
    assert_eq!(palette.property_field_focus_border, [21, 22, 23, 24]);
    assert_eq!(palette.property_axis_label_text, [25, 26, 27, 28]);
    assert_eq!(palette.property_value_text, [29, 30, 31, 32]);
}
