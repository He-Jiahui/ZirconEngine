use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};

use super::super::ChartKind;
use super::gauge::chart_value;
use super::identity::chart_kind_name;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::HostMaterialPalette;

const MAX_CHART_RASTER_CACHE_ENTRIES: usize = 128;
const MAX_CHART_RASTER_CACHE_BYTES: usize = 32 * 1024 * 1024;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct ChartRasterCacheKey {
    kind_name: &'static str,
    width: u32,
    height: u32,
    selected: bool,
    checked: bool,
    value_bits: u32,
    colors: [[u8; 4]; 3],
}

impl ChartRasterCacheKey {
    pub(super) fn new(
        node: &TemplatePaneNodeData,
        width: u32,
        height: u32,
        kind: ChartKind,
        palette: HostMaterialPalette,
    ) -> Self {
        let kind_name = chart_kind_name(kind);
        let (selected, checked, value_bits, colors) = match kind {
            ChartKind::Line => (false, false, 0, [palette.accent, palette.success, [0; 4]]),
            ChartKind::Pie => (
                node.selected,
                node.checked,
                0,
                [palette.accent, palette.success, palette.warning],
            ),
            ChartKind::Sparkline => (false, false, 0, [palette.accent, [0; 4], [0; 4]]),
            ChartKind::Gauge => (
                false,
                false,
                chart_value(node).to_bits(),
                [palette.surface_hover, palette.accent, [0; 4]],
            ),
            ChartKind::Aggregate | ChartKind::Bar => unreachable!("non-raster chart kind"),
        };
        Self {
            kind_name,
            width,
            height,
            selected,
            checked,
            value_bits,
            colors,
        }
    }

    pub(super) fn resource_key(&self) -> String {
        let [primary, secondary, tertiary] = self.colors;
        format!(
            "mui-x-chart:{}:{}x{}:selected:{}:checked:{}:value:{:08x}:colors:{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            self.kind_name,
            self.width,
            self.height,
            u8::from(self.selected),
            u8::from(self.checked),
            self.value_bits,
            primary[0],
            primary[1],
            primary[2],
            primary[3],
            secondary[0],
            secondary[1],
            secondary[2],
            secondary[3],
            tertiary[0],
            tertiary[1],
            tertiary[2],
            tertiary[3],
        )
    }
}

struct ChartRasterCacheEntry {
    resource_key: String,
    rgba: Arc<[u8]>,
    last_used: u64,
}

#[derive(Default)]
struct ChartRasterCache {
    entries: BTreeMap<ChartRasterCacheKey, ChartRasterCacheEntry>,
    resident_bytes: usize,
    access_clock: u64,
}

pub(super) struct CachedChartRaster {
    pub(super) resource_key: String,
    pub(super) rgba: Arc<[u8]>,
}

pub(super) fn cached_chart_raster(key: &ChartRasterCacheKey) -> Option<CachedChartRaster> {
    chart_raster_cache()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .get(key)
}

pub(super) fn store_chart_raster(key: ChartRasterCacheKey, resource_key: String, rgba: Arc<[u8]>) {
    chart_raster_cache()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .insert(key, resource_key, rgba);
}

impl ChartRasterCache {
    fn get(&mut self, key: &ChartRasterCacheKey) -> Option<CachedChartRaster> {
        let last_used = self.next_access();
        self.entries.get_mut(key).map(|entry| {
            entry.last_used = last_used;
            CachedChartRaster {
                resource_key: entry.resource_key.clone(),
                rgba: entry.rgba.clone(),
            }
        })
    }

    fn insert(&mut self, key: ChartRasterCacheKey, resource_key: String, rgba: Arc<[u8]>) {
        self.remove(&key);
        let byte_size = rgba.len();
        if byte_size > MAX_CHART_RASTER_CACHE_BYTES {
            return;
        }
        while !self.entries.is_empty()
            && (self.entries.len() >= MAX_CHART_RASTER_CACHE_ENTRIES
                || self.resident_bytes.saturating_add(byte_size) > MAX_CHART_RASTER_CACHE_BYTES)
        {
            let Some(evicted_key) = self
                .entries
                .iter()
                .min_by_key(|(key, entry)| (entry.last_used, key.clone()))
                .map(|(key, _)| key.clone())
            else {
                break;
            };
            self.remove(&evicted_key);
        }
        self.resident_bytes = self.resident_bytes.saturating_add(byte_size);
        let last_used = self.next_access();
        self.entries.insert(
            key,
            ChartRasterCacheEntry {
                resource_key,
                rgba,
                last_used,
            },
        );
    }

    fn remove(&mut self, key: &ChartRasterCacheKey) {
        if let Some(entry) = self.entries.remove(key) {
            self.resident_bytes = self.resident_bytes.saturating_sub(entry.rgba.len());
        }
    }

    fn next_access(&mut self) -> u64 {
        self.access_clock = self.access_clock.saturating_add(1);
        self.access_clock
    }
}

fn chart_raster_cache() -> &'static Mutex<ChartRasterCache> {
    CHART_RASTER_CACHE.get_or_init(|| Mutex::new(ChartRasterCache::default()))
}

static CHART_RASTER_CACHE: OnceLock<Mutex<ChartRasterCache>> = OnceLock::new();

#[cfg(test)]
mod arc_pixels_tests;

#[cfg(test)]
mod tests {
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
}
