use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_geometry::inset;
use super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::super::super::render_commands::HostPaintCommand;
use super::identity::ChartKind;

const MUI_X_CHART_INSET: f32 = 8.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_chart(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    kind: ChartKind,
) {
    let palette = current_host_palette();
    let radius = super::super::node_radius(node).max(4.0);
    super::super::push_quad(
        commands,
        rect.clone(),
        clip,
        order,
        super::super::node_background(node)
            .unwrap_or_else(|| chart_surface_color_from_host(node, palette)),
        0.0,
        radius,
        opacity,
    );

    let plot = inset(rect, MUI_X_CHART_INSET);
    super::super::push_quad(
        commands,
        plot.clone(),
        clip,
        order + 1,
        chart_plot_color_from_host(palette),
        0.0,
        3.0,
        opacity,
    );

    match kind {
        ChartKind::Aggregate | ChartKind::Bar => {
            super::bars::push_bar_chart(commands, &plot, clip, order, opacity)
        }
        ChartKind::Line | ChartKind::Pie | ChartKind::Sparkline | ChartKind::Gauge => {
            super::raster_commands::push_chart_raster(
                commands,
                node,
                &plot,
                clip,
                order + 2,
                opacity,
                kind,
            )
        }
    }
}

fn chart_plot_color_from_host(palette: HostMaterialPalette) -> [u8; 4] {
    palette.surface
}

fn chart_surface_color_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> [u8; 4] {
    if node.component_variant.as_str().contains("loading") {
        palette.warning_container
    } else {
        palette.surface_inset
    }
}

#[cfg(test)]
mod tests {
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
}
