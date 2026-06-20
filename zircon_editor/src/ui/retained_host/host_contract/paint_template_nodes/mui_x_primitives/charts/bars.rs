use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_theme::PALETTE;
use super::super::super::render_commands::HostPaintCommand;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_bar_chart(
    commands: &mut Vec<HostPaintCommand>,
    plot: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_chart_bar(
        commands,
        plot,
        clip,
        order + 2,
        0.18,
        0.72,
        PALETTE.accent,
        opacity,
    );
    push_chart_bar(
        commands,
        plot,
        clip,
        order + 3,
        0.42,
        0.48,
        PALETTE.success,
        opacity,
    );
    push_chart_bar(
        commands,
        plot,
        clip,
        order + 4,
        0.66,
        0.62,
        PALETTE.warning,
        opacity,
    );
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
