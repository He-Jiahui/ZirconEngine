mod bars;
mod raster;
mod raster_commands;

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_geometry::inset;
use super::super::super::paint_theme::PALETTE;
use super::super::render_commands::HostPaintCommand;

const MUI_X_CHART_INSET: f32 = 8.0;

#[derive(Clone, Copy)]
pub(super) enum ChartKind {
    Aggregate,
    Line,
    Bar,
    Pie,
    Sparkline,
    Gauge,
}

pub(super) fn chart_kind(component_role: &str, role: &str) -> Option<ChartKind> {
    if super::matches_any_role(component_role, role, &["mui-x-line-chart", "LineChart"]) {
        Some(ChartKind::Line)
    } else if super::matches_any_role(component_role, role, &["mui-x-bar-chart", "BarChart"]) {
        Some(ChartKind::Bar)
    } else if super::matches_any_role(component_role, role, &["mui-x-pie-chart", "PieChart"]) {
        Some(ChartKind::Pie)
    } else if super::matches_any_role(component_role, role, &["mui-x-sparkline", "SparkLineChart"])
    {
        Some(ChartKind::Sparkline)
    } else if super::matches_any_role(component_role, role, &["mui-x-gauge", "Gauge"]) {
        Some(ChartKind::Gauge)
    } else if super::matches_any_role(component_role, role, &["mui-x-charts", "Charts"]) {
        Some(ChartKind::Aggregate)
    } else {
        None
    }
}

pub(super) fn push_chart(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    kind: ChartKind,
) {
    let radius = super::node_radius(node).max(4.0);
    super::push_quad(
        commands,
        rect.clone(),
        clip,
        order,
        super::node_background(node).unwrap_or_else(|| chart_surface_color(node)),
        0.0,
        radius,
        opacity,
    );

    let plot = inset(rect, MUI_X_CHART_INSET);
    super::push_quad(
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
            bars::push_bar_chart(commands, &plot, clip, order, opacity)
        }
        ChartKind::Line | ChartKind::Pie | ChartKind::Sparkline | ChartKind::Gauge => {
            raster_commands::push_chart_raster(
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
