use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_geometry::inset;
use super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::super::super::render_commands::HostPaintCommand;
use super::identity::ChartKind;

const MUI_X_CHART_INSET: f32 = 8.0;

/// 组件外框与绘图区先由此入口定界；部分图种再以缓存位图叠加，避免通用回退重复绘制。
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
#[path = "tests/surface.rs"]
mod tests;
