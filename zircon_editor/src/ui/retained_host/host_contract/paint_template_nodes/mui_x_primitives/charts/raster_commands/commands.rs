use std::sync::Arc;

use super::super::super::super::render_commands::HostPaintCommand;
use super::super::raster::ChartRaster;
use super::super::ChartKind;
use super::cache::{
    cached_chart_raster, store_chart_raster, CachedChartRaster, ChartRasterCacheKey,
};
use super::dimensions::chart_raster_dimensions;
use super::gauge::{chart_value, draw_gauge_raster};
use super::line::draw_line_chart_raster;
use super::pie::draw_pie_chart_raster;
use super::sparkline::draw_sparkline_raster;
use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::host_contract::paint_theme::current_host_palette;

/// 折线、饼图、火花线与仪表盘共用这条位图路径；条形和聚合图已由上层矢量分支接管。
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
    if matches!(kind, ChartKind::Aggregate | ChartKind::Bar) {
        return;
    }
    let cache_key = ChartRasterCacheKey::new(node, width, height, kind, current_host_palette());
    let CachedChartRaster { resource_key, rgba } =
        cached_chart_raster(&cache_key).unwrap_or_else(|| {
            let mut raster = ChartRaster::transparent(width, height);
            match kind {
                ChartKind::Line => draw_line_chart_raster(&mut raster),
                ChartKind::Pie => draw_pie_chart_raster(&mut raster, node),
                ChartKind::Sparkline => draw_sparkline_raster(&mut raster),
                ChartKind::Gauge => draw_gauge_raster(&mut raster, chart_value(node)),
                ChartKind::Aggregate | ChartKind::Bar => unreachable!("non-raster chart kind"),
            }
            let resource_key = cache_key.resource_key();
            let rgba = Arc::<[u8]>::from(raster.rgba);
            store_chart_raster(cache_key, resource_key.clone(), Arc::clone(&rgba));
            CachedChartRaster { resource_key, rgba }
        });
    commands.push(HostPaintCommand::image_pixels(
        plot.clone(),
        Some(clip.clone()),
        order,
        resource_key,
        width,
        height,
        rgba,
        None,
        opacity,
    ));
}

#[cfg(test)]
#[path = "tests/commands.rs"]
mod tests;
