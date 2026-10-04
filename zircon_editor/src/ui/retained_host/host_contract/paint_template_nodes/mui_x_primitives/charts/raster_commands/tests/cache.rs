use super::{ChartRasterCache, ChartRasterCacheKey, MAX_CHART_RASTER_CACHE_ENTRIES};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_template_nodes::mui_x_primitives::charts::ChartKind;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

fn key(index: u32) -> ChartRasterCacheKey {
    ChartRasterCacheKey::new(
        &TemplatePaneNodeData::default(),
        index,
        1,
        ChartKind::Line,
        PALETTE,
    )
}

#[test]
fn cache_evicts_the_least_recently_used_chart_raster() {
    let mut cache = ChartRasterCache::default();
    for index in 0..MAX_CHART_RASTER_CACHE_ENTRIES {
        cache.insert(
            key(index as u32),
            format!("chart-{index:03}"),
            vec![index as u8].into(),
        );
    }
    assert!(cache.get(&key(0)).is_some());

    cache.insert(key(u32::MAX), "chart-new".to_string(), vec![0].into());

    assert_eq!(cache.entries.len(), MAX_CHART_RASTER_CACHE_ENTRIES);
    assert!(cache.get(&key(0)).is_some());
    assert!(cache.get(&key(1)).is_none());
    assert!(cache.get(&key(u32::MAX)).is_some());
}
