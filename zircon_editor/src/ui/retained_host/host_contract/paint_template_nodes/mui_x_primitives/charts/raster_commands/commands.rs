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
            store_chart_raster(cache_key, resource_key.clone(), raster.rgba.clone());
            CachedChartRaster {
                resource_key,
                rgba: raster.rgba,
            }
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
mod tests {
    use super::ChartRasterCacheKey;
    use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
    use crate::ui::retained_host::host_contract::paint_template_nodes::mui_x_primitives::charts::ChartKind;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

    #[test]
    fn chart_resource_key_separates_dynamic_chart_content() {
        let base = TemplatePaneNodeData::default();
        let mut changed_value = base.clone();
        changed_value.value_percent = 0.8;
        let mut selected = base.clone();
        selected.selected = true;

        assert_ne!(
            ChartRasterCacheKey::new(&base, 64, 32, ChartKind::Gauge, PALETTE),
            ChartRasterCacheKey::new(&changed_value, 64, 32, ChartKind::Gauge, PALETTE),
        );
        assert_ne!(
            ChartRasterCacheKey::new(&base, 64, 32, ChartKind::Pie, PALETTE),
            ChartRasterCacheKey::new(&selected, 64, 32, ChartKind::Pie, PALETTE),
        );
    }
}
