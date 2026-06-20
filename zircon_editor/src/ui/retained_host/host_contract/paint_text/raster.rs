use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::OnceLock;
use std::sync::{Arc, Mutex};

use fontdue::Metrics;

use super::font::fallback_font;

#[derive(Clone)]
pub(in crate::ui::retained_host::host_contract) struct CachedGlyphRaster {
    pub(in crate::ui::retained_host::host_contract) metrics: Metrics,
    pub(in crate::ui::retained_host::host_contract) bitmap: Arc<[u8]>,
}

#[derive(Clone, Copy, Debug, Eq)]
struct GlyphRasterKey {
    glyph_index: u16,
    px_bits: u32,
}

impl PartialEq for GlyphRasterKey {
    fn eq(&self, other: &Self) -> bool {
        self.glyph_index == other.glyph_index && self.px_bits == other.px_bits
    }
}

impl Hash for GlyphRasterKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.glyph_index.hash(state);
        self.px_bits.hash(state);
    }
}

pub(in crate::ui::retained_host::host_contract) fn rasterize_cached_glyph(
    glyph_index: u16,
    px: f32,
) -> CachedGlyphRaster {
    static CACHE: OnceLock<Mutex<HashMap<GlyphRasterKey, CachedGlyphRaster>>> = OnceLock::new();

    let key = GlyphRasterKey {
        glyph_index,
        px_bits: px.to_bits(),
    };
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(raster) = cache
        .lock()
        .expect("glyph raster cache lock")
        .get(&key)
        .cloned()
    {
        return raster;
    }

    let (metrics, bitmap) = fallback_font().rasterize_indexed(glyph_index, px);
    let raster = CachedGlyphRaster {
        metrics,
        bitmap: Arc::from(bitmap),
    };
    cache
        .lock()
        .expect("glyph raster cache lock")
        .insert(key, raster.clone());
    raster
}
