use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_metrics, HostControlMetrics,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchFieldMetrics {
    pub border_width: f32,
    pub radius: f32,
    pub font_size: f32,
    pub line_height: f32,
    pub input_pad_left: f32,
    pub input_pad_right: f32,
    pub search_icon_size: f32,
    pub search_text_left: f32,
    pub search_max_height: f32,
    pub search_fallback_ring_size: f32,
    pub search_fallback_radius: f32,
    pub stepper_width: f32,
    pub stepper_divider_width: f32,
    pub stepper_divider_inset_y: f32,
    pub stepper_glyph_left_inset: f32,
    pub stepper_glyph_width: f32,
    pub stepper_glyph_height: f32,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_field_metrics(
) -> WorkbenchFieldMetrics {
    workbench_field_metrics_from_host(current_host_metrics())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_field_metrics_from_host(
    metrics: HostControlMetrics,
) -> WorkbenchFieldMetrics {
    let border_width = metrics.border_width;
    let search_icon_size = (metrics.row_height - metrics.gap_l)
        .max(metrics.font_body)
        .round();
    let search_text_left = metrics.input_pad[0] + search_icon_size + metrics.gap_s;
    let stepper_glyph_width = (search_icon_size - metrics.gap_m + border_width * 2.0)
        .round()
        .max(border_width);
    WorkbenchFieldMetrics {
        border_width,
        radius: metrics.radius_control,
        font_size: metrics.font_body,
        line_height: metrics.line_height(metrics.font_body),
        input_pad_left: metrics.input_pad[0],
        input_pad_right: metrics.input_pad[1],
        search_icon_size,
        search_text_left,
        search_max_height: metrics.row_height + border_width * 4.0,
        search_fallback_ring_size: metrics.gap_m,
        search_fallback_radius: metrics.gap_m * 0.5,
        stepper_width: metrics.button_chevron_reserve.max(search_icon_size),
        stepper_divider_width: border_width,
        stepper_divider_inset_y: metrics.gap_s,
        stepper_glyph_left_inset: metrics.gap_s,
        stepper_glyph_width,
        stepper_glyph_height: search_icon_size,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::METRICS;

    #[test]
    fn field_metrics_project_from_host_control_metrics() {
        let host_metrics = HostControlMetrics {
            row_height: 30.0,
            gap_s: 5.0,
            gap_m: 9.0,
            border_width: 2.0,
            font_body: 12.0,
            input_pad: [9.0, 9.0, 3.0, 5.0],
            ..METRICS
        };

        let metrics = workbench_field_metrics_from_host(host_metrics);

        assert_eq!(metrics.font_size, 12.0);
        assert!((metrics.line_height - 14.4).abs() < 0.001);
        assert_eq!(metrics.search_icon_size, 18.0);
        assert_eq!(metrics.search_text_left, 32.0);
        assert_eq!(metrics.search_max_height, 38.0);
        assert_eq!(metrics.stepper_width, 18.0);
        assert_eq!(metrics.stepper_divider_width, 2.0);
        assert_eq!(metrics.stepper_divider_inset_y, 5.0);
        assert_eq!(metrics.stepper_glyph_width, 13.0);
    }
}
