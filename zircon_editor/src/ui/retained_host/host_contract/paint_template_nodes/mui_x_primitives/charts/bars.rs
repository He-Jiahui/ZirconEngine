use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::super::super::render_commands::HostPaintCommand;

type ChartBarColors = [[u8; 4]; 3];

/// 图表表面通过矢量命令绘制条形图和聚合图；柱形位置按传入的 plot 布局，强调、成功与警告色取自宿主调色板。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_bar_chart(
    commands: &mut Vec<HostPaintCommand>,
    plot: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let [primary_bar, success_bar, warning_bar] =
        chart_bar_colors_from_host(current_host_palette());
    push_chart_bar(
        commands,
        plot,
        clip,
        order + 2,
        0.18,
        0.72,
        primary_bar,
        opacity,
    );
    push_chart_bar(
        commands,
        plot,
        clip,
        order + 3,
        0.42,
        0.48,
        success_bar,
        opacity,
    );
    push_chart_bar(
        commands,
        plot,
        clip,
        order + 4,
        0.66,
        0.62,
        warning_bar,
        opacity,
    );
}

fn chart_bar_colors_from_host(palette: HostMaterialPalette) -> ChartBarColors {
    [palette.accent, palette.success, palette.warning]
}

fn push_chart_bar(
    commands: &mut Vec<HostPaintCommand>,
    plot: &FrameRect,
    clip: &FrameRect,
    order: i32,
    x_factor: f32,
    height_factor: f32,
    color: [u8; 4],
    opacity: f32,
) {
    let width = (plot.width * 0.13).max(1.0);
    let height = (plot.height * height_factor).max(1.0);
    super::super::push_quad(
        commands,
        FrameRect {
            x: plot.x + plot.width * x_factor,
            y: plot.y + plot.height - height,
            width,
            height,
        },
        clip,
        order,
        color,
        0.0,
        2.0,
        opacity,
    );
}

#[cfg(test)]
#[path = "tests/bars.rs"]
mod tests;
