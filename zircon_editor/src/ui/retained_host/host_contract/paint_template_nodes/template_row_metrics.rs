use super::super::paint_theme::{
    current_host_metrics, current_host_palette, HostControlMetrics, HostMaterialPalette,
};

const PROPERTY_LABEL_ROW_MULTIPLIER: f32 = 3.5;
const COMPONENT_PROPERTY_LABEL_ROW_MULTIPLIER: f32 = 4.0;
const PROPERTY_LABEL_MAX_WIDTH_RATIO: f32 = 0.45;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchRowMetrics {
    pub row_height: f32,
    pub text_font_size: f32,
    pub text_line_height: f32,
    pub surface_radius: f32,
    pub text_inset_x: f32,
    pub text_inset_y: f32,
    pub right_reserve: f32,
    pub selection_indicator_width: f32,
    pub tree_base_inset_x: f32,
    pub tree_disclosure_size: f32,
    pub tree_icon_size: f32,
    pub tree_text_gap: f32,
    pub tree_right_inset: f32,
    pub tree_action_size: f32,
    pub tree_action_button_size: f32,
    pub tree_action_gap: f32,
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchRowPalette {
    pub selection_indicator: [u8; 4],
    pub tree_guide: [u8; 4],
    pub disabled_adornment_tint: [u8; 4],
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_row_metrics(
) -> WorkbenchRowMetrics {
    row_metrics_from_host(current_host_metrics())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_row_palette(
) -> WorkbenchRowPalette {
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
        text_inset_x: metrics.gap_m,
        text_inset_y: metrics.gap_s,
        right_reserve: metrics.button_chevron_reserve + metrics.gap_m,
        selection_indicator_width: metrics.selection_indicator_width,
        tree_base_inset_x: metrics.button_pad_x,
        tree_disclosure_size: metrics.gap_l,
        tree_icon_size: metrics.font_large,
        tree_text_gap: metrics.gap_s + metrics.border_width * 2.0,
        tree_right_inset: metrics.button_pad_x,
        tree_action_size,
        tree_action_button_size: tree_action_size + metrics.gap_s,
        tree_action_gap: metrics.gap_l + metrics.gap_s,
        tree_guide_step: metrics.button_chevron_reserve,
        tree_guide_offset_x: metrics.gap_s + metrics.border_width,
        property_label_width: metrics.row_height * PROPERTY_LABEL_ROW_MULTIPLIER,
        component_property_label_width: metrics.row_height * COMPONENT_PROPERTY_LABEL_ROW_MULTIPLIER
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
    }
}

fn row_palette_from_host(palette: HostMaterialPalette) -> WorkbenchRowPalette {
    WorkbenchRowPalette {
        selection_indicator: palette.accent,
        tree_guide: palette.track,
        disabled_adornment_tint: palette.text_disabled,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn row_text_line_height(
) -> f32 {
    workbench_row_metrics().text_line_height
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
        assert_eq!(metrics.text_line_height, 15.4);
        assert_eq!(metrics.surface_radius, 5.0);
        assert_eq!(metrics.text_inset_x, 9.0);
        assert_eq!(metrics.text_inset_y, 5.0);
        assert_eq!(metrics.right_reserve, 30.0);
        assert_eq!(metrics.selection_indicator_width, 3.0);
        assert_eq!(metrics.tree_base_inset_x, 14.0);
        assert_eq!(metrics.tree_disclosure_size, 13.0);
        assert_eq!(metrics.tree_icon_size, 15.0);
        assert_eq!(metrics.tree_text_gap, 8.0);
        assert_eq!(metrics.tree_action_size, 18.0);
        assert_eq!(metrics.tree_action_button_size, 23.0);
        assert_eq!(metrics.tree_guide_step, 21.0);
        assert_eq!(metrics.tree_guide_offset_x, 6.5);
        assert_eq!(metrics.property_label_width, 105.0);
        assert_eq!(metrics.component_property_label_width, 115.0);
        assert_eq!(metrics.property_label_min_width, 60.0);
        assert_eq!(metrics.property_text_inset_x, 6.5);
        assert_eq!(metrics.property_axis_width, 13.0);
        assert_eq!(metrics.property_field_inset_y, 4.0);
        assert_eq!(metrics.property_field_radius, 5.0);
    }

    #[test]
    fn row_palette_projects_from_host_material_palette() {
        let mut host = PALETTE;
        host.accent = [1, 2, 3, 4];
        host.track = [5, 6, 7, 8];
        host.text_disabled = [9, 10, 11, 12];

        let palette = row_palette_from_host(host);

        assert_eq!(palette.selection_indicator, [1, 2, 3, 4]);
        assert_eq!(palette.tree_guide, [5, 6, 7, 8]);
        assert_eq!(palette.disabled_adornment_tint, [9, 10, 11, 12]);
    }
}
