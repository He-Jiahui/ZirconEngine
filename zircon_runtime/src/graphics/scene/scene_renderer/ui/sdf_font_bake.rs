use std::collections::HashMap;

use crate::asset::ProjectAssetManager;
use crate::core::framework::render::{
    FontFaceId, FontFamilyName, FontQuery, FontStretch, FontStyle, FontWeight,
};
use crate::graphics::text::font::{
    FontDatabase, TextDecorationMetrics, TextDecorationMetricsCache,
};
use crate::graphics::text::sdf::{SdfBakeParams, SdfGlyphGenerationError};
use zircon_runtime_interface::ui::surface::UiResolvedStyle;

use super::font_asset::{load_ui_font_manifest_with_asset_manager, LoadedUiFontManifest};
use super::render::ScreenSpaceUiTextBatch;
use super::sdf_atlas::{
    distance_field_atlas_page_keys, SdfAtlasGlyphKey, SdfAtlasPlan, SdfAtlasRect,
};

mod distance_field;
mod offline_source;

use distance_field::bake_distance_field_glyph;
use offline_source::SdfOfflineSourceCache;

const DEFAULT_FONT_ASSET: &str = "res://fonts/default.font.toml";
const FALLBACK_ADVANCE_RATIO: f32 = 0.6;

pub(super) struct SdfFontBakeCache {
    fonts: HashMap<FontFaceId, fontsdf::Font>,
    glyphs: HashMap<SdfAtlasGlyphKey, RawBakedGlyph>,
    decoration_metrics: TextDecorationMetricsCache,
    offline_source: SdfOfflineSourceCache,
}

#[derive(Clone, Debug)]
pub(super) struct SdfAtlasBake {
    pub(super) pixels: Vec<u8>,
    pub(super) pages: Vec<SdfAtlasBakePage>,
    pub(super) glyphs: Vec<SdfBakedGlyph>,
    pub(super) generation_failures: Vec<SdfAtlasGlyphGenerationFailure>,
    pub(super) report: SdfAtlasBakeReport,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SdfAtlasBakePage {
    pub(super) page_key: crate::graphics::text::atlas::GlyphAtlasPageKey,
    pub(super) source_offset: usize,
    pub(super) byte_len: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SdfAtlasGlyphGenerationFailure {
    pub(super) slot_index: usize,
    pub(super) key: SdfAtlasGlyphKey,
    pub(super) error: SdfGlyphGenerationError,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct SdfAtlasBakeReport {
    pub(super) slot_count: usize,
    pub(super) visible_glyph_count: usize,
    pub(super) empty_glyph_count: usize,
    pub(super) atlas_byte_len: usize,
    pub(super) nonzero_pixel_count: usize,
    /// Materialized faces retained by the cache after this atlas build.
    pub(super) resident_font_count: usize,
    /// Faces materialized by this atlas build rather than reused from the cache.
    pub(super) loaded_font_count: usize,
    pub(super) generation_failure_count: usize,
    pub(super) r8_byte_len: usize,
    pub(super) rgba_byte_len: usize,
    pub(super) offline_glyph_count: usize,
    pub(super) dynamic_glyph_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct SdfBakedGlyph {
    pub(super) metrics: SdfGlyphMetrics,
    pub(super) visible: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct SdfGlyphMetrics {
    pub(super) bitmap_width: u32,
    pub(super) bitmap_height: u32,
    pub(super) bitmap_left: f32,
    pub(super) bitmap_bottom: f32,
    pub(super) advance: f32,
    pub(super) ascent: f32,
}

impl SdfFontBakeCache {
    pub(super) fn new() -> Self {
        Self {
            fonts: HashMap::new(),
            glyphs: HashMap::new(),
            decoration_metrics: TextDecorationMetricsCache::default(),
            offline_source: SdfOfflineSourceCache::default(),
        }
    }

    pub(super) fn text_decoration_metrics(
        &mut self,
        text: &ScreenSpaceUiTextBatch,
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) -> TextDecorationMetrics {
        let requested_face = resolve_font_face(text.font.as_deref(), font_database, asset_manager);
        let default_face =
            resolve_font_face(Some(DEFAULT_FONT_ASSET), font_database, asset_manager);
        let primary_face = requested_face.or(default_face);
        let primary_metrics = primary_face
            .map(|face| {
                self.decoration_metrics
                    .resolve(font_database, face, text.font_size)
            })
            .unwrap_or_else(|| TextDecorationMetrics::fallback(text.font_size));
        let mut resolved_faces = Vec::new();
        for (glyph_index, glyph) in text.text.chars().enumerate() {
            let key = SdfAtlasGlyphKey {
                glyph,
                glyph_id: text
                    .shaped_glyphs
                    .get(glyph_index)
                    .map(|glyph| glyph.glyph_id),
                font_id: text
                    .shaped_glyphs
                    .get(glyph_index)
                    .and_then(|glyph| glyph.font_id)
                    .map(|face| face.0),
                font_instance_id: text
                    .shaped_glyphs
                    .get(glyph_index)
                    .and_then(|glyph| glyph.font_instance_id)
                    .map(|instance| instance.0),
                font: text.font.clone(),
                font_family: text.font_family.clone(),
                language: text.language.clone(),
                font_weight: text.font_weight,
                bake_params: SdfBakeParams::default(),
            };
            if let Some(face) = self
                .resolve_faces_for_key(&key, font_database, asset_manager)
                .into_iter()
                .next()
                .filter(|face| !resolved_faces.contains(face))
            {
                resolved_faces.push(face);
            }
        }
        primary_metrics.aggregate_fallback_thicknesses(resolved_faces.into_iter().map(|face| {
            self.decoration_metrics
                .resolve(font_database, face, text.font_size)
        }))
    }

    pub(super) fn build_atlas(
        &mut self,
        plan: &SdfAtlasPlan,
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) -> SdfAtlasBake {
        let resident_font_count_before = self.fonts.len();
        let width = plan.atlas_size.x.max(1);
        let height = plan.atlas_size.y.max(1);
        let page_keys = distance_field_atlas_page_keys(plan);
        let mut pages = Vec::with_capacity(page_keys.len().max(1));
        let mut pixels = Vec::new();
        for page_key in page_keys {
            let storage_format = page_key.format.storage_format();
            let byte_len =
                width as usize * height as usize * storage_format.bytes_per_pixel() as usize;
            let source_offset = pixels.len();
            pixels.resize(source_offset.saturating_add(byte_len), 0);
            pages.push(SdfAtlasBakePage {
                page_key,
                source_offset,
                byte_len,
            });
        }
        if pages.is_empty() {
            pixels.push(0);
        }
        let mut glyphs = Vec::with_capacity(plan.slots.len());
        let mut generation_failures = Vec::new();
        let mut offline_glyph_count = 0_usize;
        let mut dynamic_glyph_count = 0_usize;

        for (slot_index, slot) in plan.slots.iter().enumerate() {
            let baked = self.bake_glyph_cached(&slot.key, font_database, asset_manager);
            if let Some(error) = baked.generation_error {
                generation_failures.push(SdfAtlasGlyphGenerationFailure {
                    slot_index,
                    key: slot.key.clone(),
                    error,
                });
            }
            match baked.source {
                RawBakedGlyphSource::Offline => {
                    offline_glyph_count = offline_glyph_count.saturating_add(1)
                }
                RawBakedGlyphSource::Dynamic => {
                    dynamic_glyph_count = dynamic_glyph_count.saturating_add(1)
                }
                RawBakedGlyphSource::Failed => {}
            }
            if let Some(page) = pages.iter().find(|page| page.page_key == slot.page_key) {
                let page_end = page.source_offset.saturating_add(page.byte_len);
                let bytes_per_pixel = slot.page_key.format.storage_format().bytes_per_pixel();
                let page_pixels = &mut pixels[page.source_offset..page_end];
                write_glyph_bitmap(
                    page_pixels,
                    width,
                    height,
                    bytes_per_pixel,
                    slot.rect,
                    baked.metrics.bitmap_width,
                    baked.metrics.bitmap_height,
                    &baked.bitmap,
                );
            }
            glyphs.push(SdfBakedGlyph {
                metrics: baked.metrics,
                visible: baked.visible,
            });
        }

        let visible_glyph_count = glyphs.iter().filter(|glyph| glyph.visible).count();
        let resident_font_count = self.fonts.len();
        let report = SdfAtlasBakeReport {
            slot_count: glyphs.len(),
            visible_glyph_count,
            empty_glyph_count: glyphs.len().saturating_sub(visible_glyph_count),
            atlas_byte_len: pixels.len(),
            nonzero_pixel_count: pixels.iter().filter(|pixel| **pixel != 0).count(),
            resident_font_count,
            loaded_font_count: resident_font_count.saturating_sub(resident_font_count_before),
            generation_failure_count: generation_failures.len(),
            r8_byte_len: pages
                .iter()
                .filter(|page| page.page_key.format.storage_format().bytes_per_pixel() == 1)
                .map(|page| page.byte_len)
                .sum(),
            rgba_byte_len: pages
                .iter()
                .filter(|page| page.page_key.format.storage_format().bytes_per_pixel() == 4)
                .map(|page| page.byte_len)
                .sum(),
            offline_glyph_count,
            dynamic_glyph_count,
        };

        SdfAtlasBake {
            pixels,
            pages,
            glyphs,
            generation_failures,
            report,
        }
    }

    pub(super) fn generation_failures_for_plan(
        &mut self,
        plan: &SdfAtlasPlan,
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) -> HashMap<SdfAtlasGlyphKey, SdfGlyphGenerationError> {
        plan.slots
            .iter()
            .filter_map(|slot| {
                self.bake_glyph_cached(&slot.key, font_database, asset_manager)
                    .generation_error
                    .map(|error| (slot.key.clone(), error))
            })
            .collect()
    }

    pub(super) fn measure_glyph(
        &mut self,
        glyph: char,
        font: Option<&str>,
        font_family: Option<&str>,
        language: Option<&str>,
        font_weight: u16,
        font_size: f32,
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) -> SdfGlyphMetrics {
        let key = SdfAtlasGlyphKey {
            glyph,
            glyph_id: None,
            font_id: None,
            font_instance_id: None,
            font: font.map(str::to_string),
            font_family: font_family.map(str::to_string),
            language: language
                .map(str::trim)
                .filter(|language| !language.is_empty())
                .map(str::to_string),
            font_weight: UiResolvedStyle::normalized_font_weight(font_weight),
            bake_params: SdfBakeParams::default(),
        };
        let metrics = self
            .glyphs
            .get(&key)
            .map(|glyph| glyph.metrics)
            .unwrap_or_else(|| self.measure_key(&key, font_database, asset_manager));
        scale_sdf_metrics_for_display(metrics, font_size, key.bake_params)
    }

    fn measure_key(
        &mut self,
        key: &SdfAtlasGlyphKey,
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) -> SdfGlyphMetrics {
        let px = key.bake_params.bake_em_px_f32();
        for face in self.resolve_faces_for_key(key, font_database, asset_manager) {
            if !self.ensure_sdf_font(face, font_database) {
                continue;
            }
            let font = self.fonts.get(&face).expect("ensured SDF font");
            let index = glyph_index(font, key);
            let metrics = font.metrics_indexed_sdf(index, px);
            return glyph_metrics(font, px, metrics);
        }
        fallback_metrics(px)
    }

    fn bake_glyph_cached(
        &mut self,
        key: &SdfAtlasGlyphKey,
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) -> RawBakedGlyph {
        if let Some(glyph) = self.glyphs.get(key) {
            return glyph.clone();
        }
        let glyph = self.bake_glyph(key, font_database, asset_manager);
        self.glyphs.insert(key.clone(), glyph.clone());
        glyph
    }

    fn bake_glyph(
        &mut self,
        key: &SdfAtlasGlyphKey,
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) -> RawBakedGlyph {
        let px = key.bake_params.bake_em_px_f32();
        let mut generation_error = None;
        for face in self.resolve_faces_for_key(key, font_database, asset_manager) {
            let _ = self.ensure_sdf_font(face, font_database);
            if let Some(glyph) =
                self.offline_source
                    .load_glyph(key, face, font_database, asset_manager)
            {
                return glyph;
            }
            match bake_distance_field_glyph(key, face, font_database) {
                Ok(glyph) => return glyph,
                Err(error) => generation_error = Some(error),
            }
        }
        RawBakedGlyph::failed(
            fallback_metrics(px),
            generation_error.unwrap_or(SdfGlyphGenerationError::MissingGlyphOutline(
                key.glyph_id
                    .and_then(|glyph_id| u16::try_from(glyph_id).ok())
                    .unwrap_or(0),
            )),
        )
    }

    fn resolve_faces_for_key(
        &mut self,
        key: &SdfAtlasGlyphKey,
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) -> Vec<FontFaceId> {
        if let Some(face) = key.font_id.map(FontFaceId) {
            return font_database
                .standalone_face_bytes(face)
                .is_ok()
                .then_some(face)
                .into_iter()
                .collect();
        }
        let requested_face = resolve_font_face(key.font.as_deref(), font_database, asset_manager);
        let default_face =
            resolve_font_face(Some(DEFAULT_FONT_ASSET), font_database, asset_manager);
        let resolved_face = requested_face.or(default_face).map(|primary| {
            font_database.resolve_fallback_face_for_codepoint(
                primary,
                key.glyph,
                &font_query_for_key(key),
                None,
                key.language.as_deref(),
            )
        });
        let mut faces = Vec::new();
        for face in [resolved_face, requested_face, default_face]
            .into_iter()
            .flatten()
        {
            if !faces.contains(&face) {
                faces.push(face);
            }
        }
        faces
    }

    fn ensure_sdf_font(&mut self, face: FontFaceId, font_database: &FontDatabase) -> bool {
        if self.fonts.contains_key(&face) {
            return true;
        }
        let Some(font) = font_database
            .standalone_face_bytes(face)
            .ok()
            .and_then(|bytes| fontsdf::Font::from_bytes(bytes.as_ref()).ok())
        else {
            return false;
        };
        self.fonts.insert(face, font);
        true
    }
}

fn font_query_for_key(key: &SdfAtlasGlyphKey) -> FontQuery {
    FontQuery {
        families: vec![FontFamilyName::from(
            key.font_family.as_deref().unwrap_or_default(),
        )],
        weight: FontWeight::clamped(key.font_weight),
        style: FontStyle::Normal,
        stretch: FontStretch::NORMAL,
    }
}

#[derive(Clone, Debug)]
struct RawBakedGlyph {
    metrics: SdfGlyphMetrics,
    bitmap: Vec<u8>,
    visible: bool,
    generation_error: Option<SdfGlyphGenerationError>,
    source: RawBakedGlyphSource,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RawBakedGlyphSource {
    Offline,
    Dynamic,
    Failed,
}

impl RawBakedGlyph {
    fn failed(metrics: SdfGlyphMetrics, error: SdfGlyphGenerationError) -> Self {
        Self {
            metrics,
            bitmap: Vec::new(),
            visible: false,
            generation_error: Some(error),
            source: RawBakedGlyphSource::Failed,
        }
    }
}

fn resolve_font_face(
    font_asset: Option<&str>,
    font_database: &mut FontDatabase,
    asset_manager: &ProjectAssetManager,
) -> Option<FontFaceId> {
    let asset = font_asset
        .filter(|asset| !asset.trim().is_empty())
        .unwrap_or(DEFAULT_FONT_ASSET);
    let manifest = load_ui_font_manifest_with_asset_manager(asset, Some(asset_manager))?;
    register_loaded_font_manifest(font_database, &manifest)
}

fn register_loaded_font_manifest(
    font_database: &mut FontDatabase,
    manifest: &LoadedUiFontManifest,
) -> Option<FontFaceId> {
    if let Some(asset) = &manifest.asset {
        return font_database
            .register_font_asset(asset, &manifest.source_path)
            .ok()
            .and_then(|faces| faces.first().copied());
    }

    font_database
        .register_font_file(
            &manifest.source_path,
            manifest.family.as_deref(),
            manifest.face_index,
        )
        .ok()
}

fn glyph_index(font: &fontsdf::Font, key: &SdfAtlasGlyphKey) -> u16 {
    if let Some(glyph_id) = key
        .glyph_id
        .and_then(|glyph_id| u16::try_from(glyph_id).ok())
    {
        return glyph_id;
    }
    if font.chars().contains_key(&key.glyph) {
        font.lookup_glyph_index(key.glyph)
    } else {
        0
    }
}

fn glyph_metrics(font: &fontsdf::Font, px: f32, metrics: fontsdf::Metrics) -> SdfGlyphMetrics {
    let ascent = font
        .inner()
        .horizontal_line_metrics(px)
        .map(|metrics| metrics.ascent)
        .unwrap_or(px);
    SdfGlyphMetrics {
        bitmap_width: metrics.width as u32,
        bitmap_height: metrics.height as u32,
        bitmap_left: metrics.xmin as f32,
        bitmap_bottom: metrics.ymin as f32,
        advance: metrics.advance_width.max(px * FALLBACK_ADVANCE_RATIO),
        ascent,
    }
}

pub(super) fn scale_sdf_metrics_for_display(
    metrics: SdfGlyphMetrics,
    display_px: f32,
    bake_params: SdfBakeParams,
) -> SdfGlyphMetrics {
    let scale = display_px.max(1.0) / bake_params.bake_em_px_f32();
    SdfGlyphMetrics {
        bitmap_width: scale_bitmap_dimension(metrics.bitmap_width, scale),
        bitmap_height: scale_bitmap_dimension(metrics.bitmap_height, scale),
        bitmap_left: metrics.bitmap_left * scale,
        bitmap_bottom: metrics.bitmap_bottom * scale,
        advance: metrics.advance * scale,
        ascent: metrics.ascent * scale,
    }
}

fn scale_bitmap_dimension(value: u32, scale: f32) -> u32 {
    if value == 0 {
        0
    } else {
        ((value as f32 * scale).round() as u32).max(1)
    }
}

fn fallback_metrics(px: f32) -> SdfGlyphMetrics {
    SdfGlyphMetrics {
        advance: px.max(1.0) * FALLBACK_ADVANCE_RATIO,
        ascent: px.max(1.0),
        ..SdfGlyphMetrics::default()
    }
}

fn write_glyph_bitmap(
    pixels: &mut [u8],
    atlas_width: u32,
    atlas_height: u32,
    bytes_per_pixel: u32,
    rect: SdfAtlasRect,
    glyph_width: u32,
    glyph_height: u32,
    glyph_bitmap: &[u8],
) {
    let copy_width = glyph_width.min(rect.width);
    let copy_height = glyph_height.min(rect.height);
    let right = rect.x.saturating_add(copy_width).min(atlas_width);
    let bottom = rect.y.saturating_add(copy_height).min(atlas_height);

    for y in rect.y..bottom {
        for x in rect.x..right {
            let local_x = x - rect.x;
            let local_y = y - rect.y;
            let src = (local_y as usize * glyph_width as usize + local_x as usize)
                * bytes_per_pixel as usize;
            let dst = (y as usize * atlas_width as usize + x as usize) * bytes_per_pixel as usize;
            let src_end = src.saturating_add(bytes_per_pixel as usize);
            let dst_end = dst.saturating_add(bytes_per_pixel as usize);
            if let (Some(source), Some(destination)) =
                (glyph_bitmap.get(src..src_end), pixels.get_mut(dst..dst_end))
            {
                destination.copy_from_slice(source);
            }
        }
    }
}

#[cfg(test)]
mod tests;
