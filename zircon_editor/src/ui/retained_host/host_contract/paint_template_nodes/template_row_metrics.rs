use super::super::paint_theme::{
    HostControlMetrics, HostMaterialPalette, current_host_metrics, current_host_palette,
};

const PROPERTY_LABEL_ROW_MULTIPLIER: f32 = 3.5;
const COMPONENT_PROPERTY_LABEL_ROW_MULTIPLIER: f32 = 4.0;
const PROPERTY_LABEL_MAX_WIDTH_RATIO: f32 = 0.45;
const TREE_GUIDE_OPACITY: f32 = 0.78;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchRowMetrics {
    pub row_height: f32,
    pub text_font_size: f32,
    pub text_line_height: f32,
    pub surface_radius: f32,
    pub border_width: f32,
    pub text_inset_x: f32,
    pub text_inset_y: f32,
    pub right_reserve: f32,
    pub list_adornment_size: f32,
    pub list_adornment_right_inset: f32,
    pub selection_indicator_width: f32,
    pub tree_base_inset_x: f32,
    pub tree_disclosure_size: f32,
    pub tree_icon_size: f32,
    pub tree_text_gap: f32,
    pub tree_right_inset: f32,
    pub tree_action_size: f32,
    pub tree_action_button_size: f32,
    pub tree_action_gap: f32,
    pub tree_guide_width: f32,
    pub tree_guide_vertical_extension: f32,
    pub tree_guide_opacity: f32,
    pub tree_guide_step: f32,
    pub tree_guide_offset_x: f32,
    pub property_label_width: f32,
    pub component_property_label_width: f32,
    pub property_label_min_width: f32,
    pub property_label_max_width_ratio: f32,
    pub property_text_inset_x: f32,
    pub property_text_inset_y: f32,
    pub property_axis_width: f32,
    pub property_axis_gap: f32,
    pub property_group_gap: f32,
    pub property_field_inset_y: f32,
    pub property_field_radius: f32,
    pub property_field_border_width: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchRowPalette {
    pub selection_indicator: [u8; 4],
    pub tree_guide: [u8; 4],
    pub tree_action_slot_surface: [u8; 4],
    pub tree_action_slot_border: [u8; 4],
    pub disabled_adornment_tint: [u8; 4],
    pub property_field_surface: [u8; 4],
    pub property_field_border: [u8; 4],
    pub property_field_focus_border: [u8; 4],
    pub property_axis_label_text: [u8; 4],
    pub property_value_text: [u8; 4],
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_row_metrics()
-> WorkbenchRowMetrics {
    row_metrics_from_host(current_host_metrics())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_row_palette()
-> WorkbenchRowPalette {
    row_palette_from_host(current_host_palette())
}

fn row_metrics_from_host(metrics: HostControlMetrics) -> WorkbenchRowMetrics {
    let tree_action_size = metrics.gap_m * 2.0;
    let property_field_radius = metrics.radius_control;
    WorkbenchRowMetrics {
        row_height: metrics.row_height,
        text_font_size: metrics.font_body,
        text_line_height: metrics.line_height(metrics.font_body),
        surface_radius: metrics.radius_control,
        border_width: metrics.border_width,
        text_inset_x: metrics.gap_m,
        text_inset_y: metrics.gap_s,
        right_reserve: metrics.button_chevron_reserve + metrics.gap_m,
        list_adornment_size: (metrics.row_height - metrics.gap_l).max(1.0),
        list_adornment_right_inset: metrics.gap_l,
        selection_indicator_width: metrics.selection_indicator_width,
        tree_base_inset_x: metrics.button_pad_x,
        tree_disclosure_size: metrics.gap_l,
        tree_icon_size: (metrics.row_height - metrics.gap_l).max(1.0),
        tree_text_gap: metrics.gap_s + metrics.border_width * 2.0,
        tree_right_inset: metrics.button_pad_x,
        tree_action_size,
        tree_action_button_size: tree_action_size + metrics.gap_s,
        tree_action_gap: metrics.gap_l + metrics.gap_s,
        tree_guide_width: metrics.border_width,
        tree_guide_vertical_extension: metrics.border_width,
        tree_guide_opacity: TREE_GUIDE_OPACITY,
        tree_guide_step: metrics.button_chevron_reserve,
        tree_guide_offset_x: metrics.gap_s + metrics.border_width,
        property_label_width: metrics.row_height * PROPERTY_LABEL_ROW_MULTIPLIER,
        component_property_label_width: metrics.row_height
            * COMPONENT_PROPERTY_LABEL_ROW_MULTIPLIER
            - metrics.gap_s,
        property_label_min_width: metrics.font_large * COMPONENT_PROPERTY_LABEL_ROW_MULTIPLIER,
        property_label_max_width_ratio: PROPERTY_LABEL_MAX_WIDTH_RATIO,
        property_text_inset_x: metrics.gap_s + metrics.border_width,
        property_text_inset_y: metrics.gap_s,
        property_axis_width: metrics.gap_l,
        property_axis_gap: metrics.gap_s,
        property_group_gap: metrics.gap_s + metrics.border_width * 2.0,
        property_field_inset_y: metrics.input_pad[2],
        property_field_radius,
        property_field_border_width: metrics.border_width,
    }
}

fn row_palette_from_host(palette: HostMaterialPalette) -> WorkbenchRowPalette {
    WorkbenchRowPalette {
        selection_indicator: palette.accent,
        tree_guide: palette.track,
        tree_action_slot_surface: palette.surface_hover,
        tree_action_slot_border: palette.border,
        disabled_adornment_tint: palette.text_disabled,
        property_field_surface: palette.surface_inset,
        property_field_border: palette.border,
        property_field_focus_border: palette.focus_ring,
        property_axis_label_text: palette.text_muted,
        property_value_text: palette.text,
    }
}

#[cfg(test)]
mod tests {
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
}
