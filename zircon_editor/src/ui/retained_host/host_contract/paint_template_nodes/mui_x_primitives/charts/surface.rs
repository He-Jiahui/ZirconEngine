use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_geometry::inset;
use super::super::super::super::paint_theme::PALETTE;
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
    let radius = super::super::node_radius(node).max(4.0);
    super::super::push_quad(
        commands,
        rect.clone(),
        clip,
        order,
        super::super::node_background(node).unwrap_or_else(|| chart_surface_color(node)),
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
        PALETTE.surface,
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

fn chart_surface_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.component_variant.as_str().contains("loading") {
        PALETTE.warning_container
    } else {
        PALETTE.surface_inset
    }
}
