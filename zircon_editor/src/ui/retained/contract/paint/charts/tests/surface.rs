use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

#[test]
fn mui_x_chart_surface_colors_project_from_host_palette() {
    let mut palette = PALETTE;
    palette.surface = [10, 11, 12, 255];
    palette.surface_inset = [20, 21, 22, 255];
    palette.warning_container = [30, 31, 32, 255];

    let normal_node = TemplatePaneNodeData::default();
    let mut loading_node = TemplatePaneNodeData::default();
    loading_node.component_variant = "mui-chart-loading".into();

    assert_eq!(chart_plot_color_from_host(palette), [10, 11, 12, 255]);
    assert_eq!(
        chart_surface_color_from_host(&normal_node, palette),
        [20, 21, 22, 255]
    );
    assert_eq!(
        chart_surface_color_from_host(&loading_node, palette),
        [30, 31, 32, 255]
    );
}
