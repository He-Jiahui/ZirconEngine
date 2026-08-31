use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::OnceLock;
use std::sync::{Arc, Mutex};

use swash::scale::image::Content;
use swash::scale::{Render, ScaleContext, Source, StrikeWith};
use swash::zeno::{Format, Vector};
use swash::FontRef;

use super::super::paint_theme::{current_host_text_preferences, HostTextSmoothing};
use super::font::{host_font_snapshot_for_face, HostTextFontFace, HostTextFontSnapshot};
use super::sync::lock_recovering_poison;

mod metrics;

use self::metrics::{
    fontdue_fallback_sample_offset_x, missing_fontdue_y_offset, normalized_subpixel_offset,
    physical_raster_px_size, swash_hinting_for_physical_size, NATIVE_RASTER_SAMPLE_SCALE,
    NATIVE_SWASH_SAMPLE_OFFSET_X, NATIVE_SWASH_SAMPLE_OFFSET_Y,
};

#[derive(Clone)]
pub(in crate::ui::retained_host::host_contract) struct CachedGlyphRaster {
    pub(in crate::ui::retained_host::host_contract) metrics: CachedGlyphMetrics,
    pub(in crate::ui::retained_host::host_contract) bitmap: Arc<[u8]>,
    pub(in crate::ui::retained_host::host_contract) source: CachedGlyphRasterSource,
    pub(in crate::ui::retained_host::host_contract) format: CachedGlyphRasterFormat,
    pub(in crate::ui::retained_host::host_contract) raster_px_size: u32,
    pub(in crate::ui::retained_host::host_contract) sample_scale: f32,
    pub(in crate::ui::retained_host::host_contract) sample_offset_x: f32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::ui::retained_host::host_contract) struct CachedGlyphMetrics {
    pub(in crate::ui::retained_host::host_contract) width: usize,
    pub(in crate::ui::retained_host::host_contract) height: usize,
    pub(in crate::ui::retained_host::host_contract) x_offset: i32,
    pub(in crate::ui::retained_host::host_contract) y_offset: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::ui::retained_host::host_contract) enum CachedGlyphRasterSource {
    Swash,
    FontdueFallback,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::ui::retained_host::host_contract) enum CachedGlyphRasterFormat {
    AlphaMask,
    SubpixelMask,
    ColorRgba,
}

#[derive(Clone, Copy, Debug, Eq)]
struct GlyphRasterKey {
    font_source: GlyphRasterFontSource,
    font_cache_key: u64,
    glyph_index: u16,
    raster_px_size: u32,
    subpixel_offset_bits: u32,
    text_smoothing: HostTextSmoothing,
}

impl PartialEq for GlyphRasterKey {
    fn eq(&self, other: &Self) -> bool {
        self.font_source == other.font_source
            && self.font_cache_key == other.font_cache_key
            && self.glyph_index == other.glyph_index
            && self.raster_px_size == other.raster_px_size
            && self.subpixel_offset_bits == other.subpixel_offset_bits
            && self.text_smoothing == other.text_smoothing
    }
}

impl Hash for GlyphRasterKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.font_source.hash(state);
        self.font_cache_key.hash(state);
        self.glyph_index.hash(state);
        self.raster_px_size.hash(state);
        self.subpixel_offset_bits.hash(state);
        self.text_smoothing.hash(state);
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum GlyphRasterFontSource {
    Host(HostTextFontFace),
    RuntimeArtifact,
}

#[derive(Default)]
struct GlyphRasterCache {
    entries: HashMap<GlyphRasterKey, CachedGlyphRaster>,
    #[cfg(any(test, feature = "profiling"))]
    resident_bitmap_bytes: usize,
    #[cfg(any(test, feature = "profiling"))]
    peak_entry_count: usize,
    #[cfg(any(test, feature = "profiling"))]
    peak_resident_bitmap_bytes: usize,
}

#[cfg(any(test, feature = "profiling"))]
#[derive(Clone, Copy)]
struct GlyphRasterCachePublication {
    duplicate: bool,
    entry_count: usize,
    resident_bitmap_bytes: usize,
    peak_entry_count: usize,
    peak_resident_bitmap_bytes: usize,
}

impl GlyphRasterCache {
    #[cfg(any(test, feature = "profiling"))]
    fn publish_profiled(
        &mut self,
        key: GlyphRasterKey,
        raster: CachedGlyphRaster,
    ) -> GlyphRasterCachePublication {
        let bitmap_bytes = raster.bitmap.len();
        let previous = self.entries.insert(key, raster);
        let duplicate = previous.is_some();
        let replaced_bitmap_bytes = previous.map_or(0, |previous| previous.bitmap.len());
        self.resident_bitmap_bytes = self
            .resident_bitmap_bytes
            .saturating_sub(replaced_bitmap_bytes)
            .saturating_add(bitmap_bytes);
        self.peak_entry_count = self.peak_entry_count.max(self.entries.len());
        self.peak_resident_bitmap_bytes = self
            .peak_resident_bitmap_bytes
            .max(self.resident_bitmap_bytes);

        GlyphRasterCachePublication {
            duplicate,
            entry_count: self.entries.len(),
            resident_bitmap_bytes: self.resident_bitmap_bytes,
            peak_entry_count: self.peak_entry_count,
            peak_resident_bitmap_bytes: self.peak_resident_bitmap_bytes,
        }
    }
}

pub(in crate::ui::retained_host::host_contract) fn rasterize_cached_glyph(
    font_face: HostTextFontFace,
    glyph_index: u16,
    logical_px: f32,
    surface_scale_factor: f32,
    subpixel_offset: f32,
) -> CachedGlyphRaster {
    let font = host_font_snapshot_for_face(font_face);
    let text_smoothing = current_host_text_preferences().smoothing;
    rasterize_cached_font_glyph(
        GlyphRasterFontSource::Host(font_face),
        &font,
        glyph_index,
        physical_raster_px_size(logical_px, surface_scale_factor),
        subpixel_offset,
        text_smoothing,
    )
}

pub(super) fn rasterize_cached_host_glyph(
    font_face: HostTextFontFace,
    font: &HostTextFontSnapshot,
    glyph_index: u16,
    physical_px: f32,
    subpixel_offset: f32,
    text_smoothing: HostTextSmoothing,
) -> CachedGlyphRaster {
    rasterize_cached_font_glyph(
        GlyphRasterFontSource::Host(font_face),
        font,
        glyph_index,
        physical_raster_px_size(physical_px, 1.0),
        subpixel_offset,
        text_smoothing,
    )
}

/// Rasterizes a glyph from an exact runtime-shaped face using the retained-host cache.
pub(in crate::ui::retained_host::host_contract) fn rasterize_cached_runtime_artifact_glyph(
    font: &HostTextFontSnapshot,
    glyph_index: u16,
    physical_px: f32,
    subpixel_offset: f32,
    text_smoothing: HostTextSmoothing,
) -> CachedGlyphRaster {
    rasterize_cached_font_glyph(
        GlyphRasterFontSource::RuntimeArtifact,
        font,
        glyph_index,
        physical_raster_px_size(physical_px, 1.0),
        subpixel_offset,
        text_smoothing,
    )
}

fn rasterize_cached_font_glyph(
    font_source: GlyphRasterFontSource,
    font: &HostTextFontSnapshot,
    glyph_index: u16,
    raster_px_size: u32,
    subpixel_offset: f32,
    text_smoothing: HostTextSmoothing,
) -> CachedGlyphRaster {
    static CACHE: OnceLock<Mutex<GlyphRasterCache>> = OnceLock::new();
    let subpixel_offset = normalized_subpixel_offset(subpixel_offset);
    let key = GlyphRasterKey {
        font_source,
        font_cache_key: font.cache_key(),
        glyph_index,
        raster_px_size,
        subpixel_offset_bits: subpixel_offset.to_bits(),
        text_smoothing,
    };
    let cache = CACHE.get_or_init(|| Mutex::new(GlyphRasterCache::default()));
    if let Some(raster) = lock_recovering_poison(cache).entries.get(&key).cloned() {
        zircon_runtime::profile_counter!("editor", "retained_text_raster_cache_hit_count", 1);
        zircon_runtime::profile_counter!("editor", "retained_text_raster_cache_miss_count", 0);
        return raster;
    }

    zircon_runtime::profile_counter!("editor", "retained_text_raster_cache_hit_count", 0);
    zircon_runtime::profile_counter!("editor", "retained_text_raster_cache_miss_count", 1);
    zircon_runtime::profile_scope!("editor", "host_painter", "text_raster_cache_miss");
    let raster = rasterize_swash_glyph(
        font,
        glyph_index,
        raster_px_size,
        subpixel_offset,
        text_smoothing,
    )
    .unwrap_or_else(|| rasterize_fontdue_glyph(font, glyph_index, raster_px_size, subpixel_offset));
    zircon_runtime::profile_counter!(
        "editor",
        "retained_text_raster_cache_miss_bitmap_bytes",
        raster.bitmap.len()
    );
    zircon_runtime::profile_counter!(
        "editor",
        "retained_text_raster_cache_miss_swash_count",
        usize::from(raster.source == CachedGlyphRasterSource::Swash)
    );
    zircon_runtime::profile_counter!(
        "editor",
        "retained_text_raster_cache_miss_fontdue_fallback_count",
        usize::from(raster.source == CachedGlyphRasterSource::FontdueFallback)
    );
    #[cfg(any(test, feature = "profiling"))]
    {
        let publication = lock_recovering_poison(cache).publish_profiled(key, raster.clone());
        zircon_runtime::profile_counter!(
            "editor",
            "retained_text_raster_cache_duplicate_publish_count",
            usize::from(publication.duplicate)
        );
        zircon_runtime::profile_counter!(
            "editor",
            "retained_text_raster_cache_resident_entry_count",
            publication.entry_count
        );
        zircon_runtime::profile_counter!(
            "editor",
            "retained_text_raster_cache_resident_bitmap_bytes",
            publication.resident_bitmap_bytes
        );
        zircon_runtime::profile_counter!(
            "editor",
            "retained_text_raster_cache_peak_entry_count",
            publication.peak_entry_count
        );
        zircon_runtime::profile_counter!(
            "editor",
            "retained_text_raster_cache_peak_bitmap_bytes",
            publication.peak_resident_bitmap_bytes
        );
    }
    #[cfg(not(any(test, feature = "profiling")))]
    lock_recovering_poison(cache)
        .entries
        .insert(key, raster.clone());
    raster
}

fn rasterize_swash_glyph(
    font: &HostTextFontSnapshot,
    glyph_index: u16,
    raster_px_size: u32,
    subpixel_offset: f32,
    text_smoothing: HostTextSmoothing,
) -> Option<CachedGlyphRaster> {
    static SWASH_CONTEXT: OnceLock<Mutex<ScaleContext>> = OnceLock::new();

    let swash_font = FontRef::from_index(font.bytes(), font.collection_index() as usize)?;
    let context = SWASH_CONTEXT.get_or_init(|| Mutex::new(ScaleContext::new()));
    let mut context = lock_recovering_poison(context);
    let physical_px = raster_px_size as f32;
    let mut scaler = context
        .builder(swash_font)
        .size(physical_px)
        .hint(swash_hinting_for_physical_size(physical_px))
        .build();
    let mut render = Render::new(&[
        Source::ColorOutline(0),
        Source::ColorBitmap(StrikeWith::BestFit),
        Source::Outline,
    ]);
    render
        .format(swash_format_for_smoothing(text_smoothing))
        .offset(Vector::new(subpixel_offset, NATIVE_SWASH_SAMPLE_OFFSET_Y));
    let image = render.render(&mut scaler, glyph_index)?;
    let width = image.placement.width as usize;
    let height = image.placement.height as usize;
    let left = image.placement.left;
    let top = image.placement.top;
    let (format, bitmap) = swash_bitmap(
        image.content,
        image.source,
        image.data,
        width,
        height,
        text_smoothing,
    )?;
    if !bitmap_has_visible_ink(format, &bitmap) {
        return None;
    }
    let metrics = swash_metrics(font, glyph_index, physical_px, width, height, left, top);

    Some(CachedGlyphRaster {
        metrics,
        bitmap: Arc::from(bitmap),
        source: CachedGlyphRasterSource::Swash,
        format,
        raster_px_size,
        sample_scale: NATIVE_RASTER_SAMPLE_SCALE,
        sample_offset_x: NATIVE_SWASH_SAMPLE_OFFSET_X,
    })
}

fn swash_format_for_smoothing(smoothing: HostTextSmoothing) -> Format {
    match smoothing {
        HostTextSmoothing::Grayscale => Format::Alpha,
        HostTextSmoothing::Subpixel => Format::Subpixel,
    }
}

fn swash_metrics(
    font: &HostTextFontSnapshot,
    glyph_index: u16,
    physical_px: f32,
    width: usize,
    height: usize,
    left: i32,
    top: i32,
) -> CachedGlyphMetrics {
    let fontdue_y = font
        .font()
        .map(|font| {
            let metrics = font.metrics_indexed(glyph_index, physical_px);
            (-metrics.bounds.height - metrics.bounds.ymin).floor()
        })
        .unwrap_or_else(missing_fontdue_y_offset);
    let swash_x = left as f32;
    let swash_y = -(top as f32);

    CachedGlyphMetrics {
        width,
        height,
        x_offset: swash_x.round() as i32,
        y_offset: (swash_y - fontdue_y).round() as i32,
    }
}

fn swash_bitmap(
    content: Content,
    source: Source,
    mut data: Vec<u8>,
    width: usize,
    height: usize,
    text_smoothing: HostTextSmoothing,
) -> Option<(CachedGlyphRasterFormat, Vec<u8>)> {
    let pixel_count = width.checked_mul(height)?;
    match content {
        Content::Mask => {
            if data.len() < pixel_count {
                None
            } else {
                data.truncate(pixel_count);
                Some((CachedGlyphRasterFormat::AlphaMask, data))
            }
        }
        Content::SubpixelMask => {
            let byte_count = pixel_count.checked_mul(4)?;
            if data.len() < byte_count {
                None
            } else {
                match text_smoothing {
                    HostTextSmoothing::Grayscale => rgba_to_alpha(data, pixel_count, true)
                        .map(|alpha| (CachedGlyphRasterFormat::AlphaMask, alpha)),
                    HostTextSmoothing::Subpixel => {
                        data.truncate(byte_count);
                        Some((CachedGlyphRasterFormat::SubpixelMask, data))
                    }
                }
            }
        }
        Content::Color => {
            let byte_count = pixel_count.checked_mul(4)?;
            (data.len() >= byte_count).then(|| {
                data.truncate(byte_count);
                if matches!(source, Source::ColorOutline(_)) {
                    unpremultiply_rgba8_in_place(&mut data);
                }
                (CachedGlyphRasterFormat::ColorRgba, data)
            })
        }
    }
}

fn bitmap_has_visible_ink(format: CachedGlyphRasterFormat, bitmap: &[u8]) -> bool {
    match format {
        CachedGlyphRasterFormat::AlphaMask | CachedGlyphRasterFormat::SubpixelMask => {
            bitmap.iter().any(|coverage| *coverage > 0)
        }
        CachedGlyphRasterFormat::ColorRgba => bitmap.chunks_exact(4).any(|pixel| pixel[3] > 0),
    }
}

fn rgba_to_alpha(data: Vec<u8>, pixel_count: usize, subpixel_mask: bool) -> Option<Vec<u8>> {
    if data.len() < pixel_count.checked_mul(4)? {
        return None;
    }
    let mut alpha = Vec::with_capacity(pixel_count);
    for pixel in data.chunks_exact(4).take(pixel_count) {
        let coverage = if subpixel_mask {
            pixel[0].max(pixel[1]).max(pixel[2])
        } else {
            pixel[3]
        };
        alpha.push(coverage);
    }
    Some(alpha)
}

fn unpremultiply_rgba8_in_place(data: &mut [u8]) {
    for pixel in data.chunks_exact_mut(4) {
        let alpha = u32::from(pixel[3]);
        if alpha == 0 {
            pixel[..3].fill(0);
            continue;
        }
        if alpha == u32::from(u8::MAX) {
            continue;
        }

        for channel in &mut pixel[..3] {
            let straight = (u32::from(*channel) * u32::from(u8::MAX) + alpha / 2) / alpha;
            *channel = straight.min(u32::from(u8::MAX)) as u8;
        }
    }
}

fn rasterize_fontdue_glyph(
    font: &HostTextFontSnapshot,
    glyph_index: u16,
    raster_px_size: u32,
    subpixel_offset: f32,
) -> CachedGlyphRaster {
    let Some(font) = font.font() else {
        return empty_fontdue_raster(raster_px_size, subpixel_offset);
    };
    let raster_px = raster_px_size as f32;
    let (metrics, bitmap) = font.rasterize_indexed(glyph_index, raster_px);
    let raster_left_px = metrics.xmin as f32;
    let x_offset = raster_left_px.floor() as i32;
    CachedGlyphRaster {
        metrics: CachedGlyphMetrics {
            width: metrics.width,
            height: metrics.height,
            x_offset,
            y_offset: 0,
        },
        bitmap: Arc::from(bitmap),
        source: CachedGlyphRasterSource::FontdueFallback,
        format: CachedGlyphRasterFormat::AlphaMask,
        raster_px_size,
        sample_scale: NATIVE_RASTER_SAMPLE_SCALE,
        sample_offset_x: fontdue_fallback_sample_offset_x(
            subpixel_offset,
            raster_left_px,
            x_offset,
        ),
    }
}

fn empty_fontdue_raster(raster_px_size: u32, subpixel_offset: f32) -> CachedGlyphRaster {
    CachedGlyphRaster {
        metrics: CachedGlyphMetrics::default(),
        bitmap: Arc::<[u8]>::from(Vec::<u8>::new()),
        source: CachedGlyphRasterSource::FontdueFallback,
        format: CachedGlyphRasterFormat::AlphaMask,
        raster_px_size,
        sample_scale: NATIVE_RASTER_SAMPLE_SCALE,
        sample_offset_x: normalized_subpixel_offset(subpixel_offset),
    }
}

#[cfg(test)]
mod tests;
