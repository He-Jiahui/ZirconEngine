use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::OnceLock;
use std::sync::{Arc, Mutex};

use swash::scale::image::Content;
use swash::scale::{Render, ScaleContext, Source, StrikeWith};
use swash::zeno::{Format, Vector};
use swash::FontRef;

use super::super::paint_theme::{current_host_text_preferences, HostTextSmoothing};
use super::font::{font_bytes_for_face, font_cache_key_for_face, font_for_face, HostTextFontFace};
use super::sync::lock_recovering_poison;

const MIN_NATIVE_SCALE_SWASH_MAX_COVERAGE: u8 = 128;
const MAX_FALLBACK_SAMPLE_OFFSET_X: f32 = 1.999;

#[derive(Clone)]
pub(in crate::ui::retained_host::host_contract) struct CachedGlyphRaster {
    pub(in crate::ui::retained_host::host_contract) metrics: CachedGlyphMetrics,
    pub(in crate::ui::retained_host::host_contract) bitmap: Arc<[u8]>,
    pub(in crate::ui::retained_host::host_contract) source: CachedGlyphRasterSource,
    pub(in crate::ui::retained_host::host_contract) format: CachedGlyphRasterFormat,
    pub(in crate::ui::retained_host::host_contract) raster_scale: f32,
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
}

#[derive(Clone, Copy, Debug, Eq)]
struct GlyphRasterKey {
    font_face: HostTextFontFace,
    font_cache_key: u64,
    glyph_index: u16,
    logical_px_bits: u32,
    subpixel_offset_bits: u32,
    fallback_raster_scale_bits: u32,
    text_smoothing: HostTextSmoothing,
}

impl PartialEq for GlyphRasterKey {
    fn eq(&self, other: &Self) -> bool {
        self.font_face == other.font_face
            && self.font_cache_key == other.font_cache_key
            && self.glyph_index == other.glyph_index
            && self.logical_px_bits == other.logical_px_bits
            && self.subpixel_offset_bits == other.subpixel_offset_bits
            && self.fallback_raster_scale_bits == other.fallback_raster_scale_bits
            && self.text_smoothing == other.text_smoothing
    }
}

impl Hash for GlyphRasterKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.font_face.hash(state);
        self.font_cache_key.hash(state);
        self.glyph_index.hash(state);
        self.logical_px_bits.hash(state);
        self.subpixel_offset_bits.hash(state);
        self.fallback_raster_scale_bits.hash(state);
        self.text_smoothing.hash(state);
    }
}

pub(in crate::ui::retained_host::host_contract) fn rasterize_cached_glyph(
    font_face: HostTextFontFace,
    glyph_index: u16,
    logical_px: f32,
    raster_scale: f32,
    subpixel_offset: f32,
) -> CachedGlyphRaster {
    static CACHE: OnceLock<Mutex<HashMap<GlyphRasterKey, CachedGlyphRaster>>> = OnceLock::new();

    let logical_px = logical_font_size(logical_px);
    let fallback_raster_scale = fallback_raster_scale(raster_scale);
    let subpixel_offset = normalized_subpixel_offset(subpixel_offset);
    let text_smoothing = current_host_text_preferences().smoothing;
    let key = GlyphRasterKey {
        font_face,
        font_cache_key: font_cache_key_for_face(font_face),
        glyph_index,
        logical_px_bits: logical_px.to_bits(),
        subpixel_offset_bits: subpixel_offset.to_bits(),
        fallback_raster_scale_bits: fallback_raster_scale.to_bits(),
        text_smoothing,
    };
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(raster) = lock_recovering_poison(cache).get(&key).cloned() {
        return raster;
    }

    let raster = rasterize_swash_glyph(
        font_face,
        glyph_index,
        logical_px,
        subpixel_offset,
        text_smoothing,
    )
    .unwrap_or_else(|| {
        rasterize_fontdue_glyph(
            font_face,
            glyph_index,
            logical_px,
            fallback_raster_scale,
            subpixel_offset,
        )
    });
    lock_recovering_poison(cache).insert(key, raster.clone());
    raster
}

fn logical_font_size(logical_px: f32) -> f32 {
    logical_px.max(1.0)
}

fn fallback_raster_scale(raster_scale: f32) -> f32 {
    if raster_scale.is_finite() && raster_scale > 1.0 {
        raster_scale
    } else {
        1.0
    }
}

fn fallback_raster_font_size(logical_px: f32, raster_scale: f32) -> f32 {
    logical_font_size(logical_px) * fallback_raster_scale(raster_scale)
}

fn normalized_subpixel_offset(offset: f32) -> f32 {
    if offset.is_finite() {
        offset.clamp(0.0, 0.999)
    } else {
        0.0
    }
}

fn normalized_fallback_sample_offset_x(offset: f32) -> f32 {
    if offset.is_finite() {
        offset.clamp(0.0, MAX_FALLBACK_SAMPLE_OFFSET_X)
    } else {
        0.0
    }
}

fn fontdue_fallback_sample_offset_x(
    origin_subpixel_offset: f32,
    raster_left_px: f32,
    x_offset: i32,
) -> f32 {
    normalized_fallback_sample_offset_x(
        normalized_subpixel_offset(origin_subpixel_offset) + raster_left_px - x_offset as f32,
    )
}

fn rasterize_swash_glyph(
    font_face: HostTextFontFace,
    glyph_index: u16,
    logical_px: f32,
    subpixel_offset: f32,
    text_smoothing: HostTextSmoothing,
) -> Option<CachedGlyphRaster> {
    static SWASH_CONTEXT: OnceLock<Mutex<ScaleContext>> = OnceLock::new();

    let font = FontRef::from_index(font_bytes_for_face(font_face), 0)?;
    let context = SWASH_CONTEXT.get_or_init(|| Mutex::new(ScaleContext::new()));
    let mut context = lock_recovering_poison(context);
    let mut scaler = context.builder(font).size(logical_px).hint(true).build();
    let mut render = Render::new(&[
        Source::ColorOutline(0),
        Source::ColorBitmap(StrikeWith::BestFit),
        Source::Outline,
    ]);
    render
        .format(swash_format_for_smoothing(text_smoothing))
        .offset(Vector::new(subpixel_offset, 0.0));
    let image = render.render(&mut scaler, glyph_index)?;
    let width = image.placement.width as usize;
    let height = image.placement.height as usize;
    let left = image.placement.left;
    let top = image.placement.top;
    let (format, bitmap) = swash_bitmap(image.content, image.data, width, height, text_smoothing)?;
    if bitmap_max_coverage(&bitmap) < MIN_NATIVE_SCALE_SWASH_MAX_COVERAGE {
        return None;
    }
    let metrics = swash_metrics(
        font_face,
        glyph_index,
        logical_px,
        1.0,
        width,
        height,
        left,
        top,
    );

    Some(CachedGlyphRaster {
        metrics,
        bitmap: Arc::from(bitmap),
        source: CachedGlyphRasterSource::Swash,
        format,
        raster_scale: 1.0,
        sample_offset_x: 0.0,
    })
}

fn swash_format_for_smoothing(smoothing: HostTextSmoothing) -> Format {
    match smoothing {
        HostTextSmoothing::Grayscale => Format::Alpha,
        HostTextSmoothing::Subpixel => Format::Subpixel,
    }
}

fn swash_metrics(
    font_face: HostTextFontFace,
    glyph_index: u16,
    logical_px: f32,
    raster_scale: f32,
    width: usize,
    height: usize,
    left: i32,
    top: i32,
) -> CachedGlyphMetrics {
    let scale = raster_scale.max(1.0);
    let fontdue_y = font_for_face(font_face)
        .map(|font| {
            let metrics = font.metrics_indexed(glyph_index, logical_px);
            (-metrics.bounds.height - metrics.bounds.ymin).floor()
        })
        .unwrap_or(0.0);
    let swash_x = left as f32 / scale;
    let swash_y = -(top as f32) / scale;

    CachedGlyphMetrics {
        width,
        height,
        x_offset: swash_x.round() as i32,
        y_offset: (swash_y - fontdue_y).round() as i32,
    }
}

fn swash_bitmap(
    content: Content,
    data: Vec<u8>,
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
                Some((
                    CachedGlyphRasterFormat::AlphaMask,
                    data.into_iter().take(pixel_count).collect(),
                ))
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
                    HostTextSmoothing::Subpixel => Some((
                        CachedGlyphRasterFormat::SubpixelMask,
                        data.into_iter().take(byte_count).collect(),
                    )),
                }
            }
        }
        Content::Color => rgba_to_alpha(data, pixel_count, false)
            .map(|alpha| (CachedGlyphRasterFormat::AlphaMask, alpha)),
    }
}

fn bitmap_max_coverage(bitmap: &[u8]) -> u8 {
    bitmap.iter().copied().max().unwrap_or(0)
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

fn rasterize_fontdue_glyph(
    font_face: HostTextFontFace,
    glyph_index: u16,
    logical_px: f32,
    raster_scale: f32,
    subpixel_offset: f32,
) -> CachedGlyphRaster {
    let Some(font) = font_for_face(font_face) else {
        return empty_fontdue_raster(raster_scale, subpixel_offset);
    };
    let raster_px = fallback_raster_font_size(logical_px, raster_scale);
    let (metrics, bitmap) = font.rasterize_indexed(glyph_index, raster_px);
    let raster_left_px = metrics.xmin as f32 / raster_scale;
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
        raster_scale,
        sample_offset_x: fontdue_fallback_sample_offset_x(
            subpixel_offset,
            raster_left_px,
            x_offset,
        ),
    }
}

fn empty_fontdue_raster(raster_scale: f32, subpixel_offset: f32) -> CachedGlyphRaster {
    CachedGlyphRaster {
        metrics: CachedGlyphMetrics::default(),
        bitmap: Arc::<[u8]>::from(Vec::<u8>::new()),
        source: CachedGlyphRasterSource::FontdueFallback,
        format: CachedGlyphRasterFormat::AlphaMask,
        raster_scale,
        sample_offset_x: normalized_subpixel_offset(subpixel_offset),
    }
}

#[cfg(test)]
mod tests;
