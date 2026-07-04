use super::super::super::super::paint_theme::{current_host_metrics, HostControlMetrics};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchCommandPaletteMetrics
{
    pub panel_radius: f32,
    pub search_radius: f32,
    pub row_radius: f32,
    pub font_size: f32,
    pub line_height: f32,
    pub panel_padding_x: f32,
    pub search_top: f32,
    pub search_height: f32,
    pub search_icon_size: f32,
    pub search_icon_x: f32,
    pub search_text_x: f32,
    pub search_text_y: f32,
    pub list_top: f32,
    pub row_inset_x: f32,
    pub row_height: f32,
    pub row_text_x: f32,
    pub row_text_y: f32,
    pub match_indicator_left: f32,
    pub match_indicator_width: f32,
    pub match_indicator_height: f32,
    pub empty_text_y: f32,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn command_palette_metrics(
) -> WorkbenchCommandPaletteMetrics {
    command_palette_metrics_from_host(current_host_metrics())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn command_palette_metrics_from_host(
    metrics: HostControlMetrics,
) -> WorkbenchCommandPaletteMetrics {
    let font_size = (metrics.font_body + metrics.font_large) * 0.5;
    let line_height = metrics.line_height(font_size);
    let search_top = metrics.gap_m + metrics.border_width * 2.0;
    let search_height = metrics.row_height + metrics.gap_s + metrics.border_width * 2.0;
    let list_top = search_top + search_height + metrics.gap_m;
    let search_icon_size = (metrics.row_height - metrics.gap_m)
        .max(metrics.font_body)
        .round();
    let search_icon_x = metrics.input_pad[0] + metrics.border_width * 2.0;
    WorkbenchCommandPaletteMetrics {
        panel_radius: metrics.radius_control + metrics.border_width * 2.0,
        search_radius: metrics.radius_control,
        row_radius: (metrics.radius_control - metrics.border_width).max(0.0),
        font_size,
        line_height,
        panel_padding_x: metrics.gap_l,
        search_top,
        search_height,
        search_icon_size,
        search_icon_x,
        search_text_x: search_icon_x + search_icon_size + metrics.gap_s,
        search_text_y: metrics.input_pad[2] + metrics.gap_s,
        list_top,
        row_inset_x: metrics.gap_m,
        row_height: metrics.row_height + metrics.border_width * 2.0,
        row_text_x: metrics.input_pad[0] + metrics.border_width,
        row_text_y: metrics.input_pad[2] + metrics.border_width * 2.0,
        match_indicator_left: metrics.gap_s,
        match_indicator_width: metrics.selection_indicator_width.max(metrics.border_width),
        match_indicator_height: metrics.font_large.max(metrics.gap_l),
        empty_text_y: search_top
            + search_height
            + metrics.gap_l
            + metrics.gap_s
            + metrics.border_width * 2.0,
    }
}
