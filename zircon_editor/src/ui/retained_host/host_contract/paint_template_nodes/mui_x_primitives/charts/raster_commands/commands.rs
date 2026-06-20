use super::super::super::super::render_commands::HostPaintCommand;
use super::super::raster::ChartRaster;
use super::super::ChartKind;
use super::dimensions::chart_raster_dimensions;
use super::gauge::{chart_value, draw_gauge_raster};
use super::identity::chart_kind_name;
use super::line::draw_line_chart_raster;
use super::pie::draw_pie_chart_raster;
use super::sparkline::draw_sparkline_raster;
use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_chart_raster(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    plot: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    kind: ChartKind,
) {
    let Some((width, height)) = chart_raster_dimensions(plot) else {
        return;
    };
    let mut raster = ChartRaster::transparent(width, height);
    match kind {
        ChartKind::Line => draw_line_chart_raster(&mut raster),
        ChartKind::Pie => draw_pie_chart_raster(&mut raster, node),
        ChartKind::Sparkline => draw_sparkline_raster(&mut raster),
        ChartKind::Gauge => draw_gauge_raster(&mut raster, chart_value(node)),
        ChartKind::Aggregate | ChartKind::Bar => return,
    }
    commands.push(HostPaintCommand::image_pixels(
        plot.clone(),
        Some(clip.clone()),
        order,
        format!("mui-x-chart:{}:{}x{}", chart_kind_name(kind), width, height),
        width,
        height,
        raster.rgba,
        None,
        opacity,
    ));
}
