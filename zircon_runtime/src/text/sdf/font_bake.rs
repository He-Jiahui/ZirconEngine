use std::collections::{BTreeSet, HashMap};

use crate::asset::ProjectAssetManager;
use crate::core::framework::text::TextFontFaceHandle;
use crate::core::math::UVec2;
use crate::text::atlas::GlyphAtlasPageKey;
use crate::text::font::{
    load_text_font_source, shared_font_database_generation, FontDatabase, LoadedTextFontSource,
    TextDecorationMetrics, TextDecorationMetricsCache,
};
use crate::text::sdf::{SdfBakeParams, SdfGlyphGenerationError};
use crate::text::{FontFaceId, FontFamilyName, FontQuery, FontStretch, FontStyle, FontWeight};

mod distance_field;
mod offline_source;

use distance_field::bake_distance_field_glyph;
use offline_source::SdfOfflineSourceCache;

const DEFAULT_FONT_ASSET: &str = "res://fonts/default.font.toml";
const FALLBACK_ADVANCE_RATIO: f32 = 0.6;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SdfAtlasRect {
    pub(crate) x: u32,
    pub(crate) y: u32,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct SdfAtlasGlyphKey {
    pub(crate) glyph: char,
    pub(crate) glyph_id: Option<u32>,
    pub(crate) font_id: Option<TextFontFaceHandle>,
    pub(crate) font_instance_id: Option<TextFontFaceHandle>,
    pub(crate) font: Option<String>,
    pub(crate) font_family: Option<String>,
    pub(crate) language: Option<String>,
    pub(crate) font_weight: u16,
    pub(crate) bake_params: SdfBakeParams,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SdfAtlasSlot {
    pub(crate) key: SdfAtlasGlyphKey,
    pub(crate) page_key: GlyphAtlasPageKey,
    pub(crate) rect: SdfAtlasRect,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SdfShapedGlyphIdentity {
    pub(crate) glyph_id: u32,
    pub(crate) font_id: Option<TextFontFaceHandle>,
    pub(crate) font_instance_id: Option<TextFontFaceHandle>,
}

pub(crate) trait SdfTextRun {
    fn font(&self) -> Option<&str>;
    fn font_family(&self) -> Option<&str>;
    fn language(&self) -> Option<&str>;
    fn font_weight(&self) -> u16;
    fn font_size(&self) -> f32;
    fn render_scalars(&self) -> Vec<char>;
    fn resolved_glyph_advances(&self) -> Option<Vec<f32>>;
    fn shaped_glyph(&self, glyph_index: usize) -> Option<SdfShapedGlyphIdentity>;
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct SdfRunCpuPreparation {
    pub(crate) glyph_metrics: Vec<SdfGlyphMetrics>,
    pub(crate) glyph_advances: Vec<f32>,
    pub(crate) decoration_metrics: TextDecorationMetrics,
}

pub(crate) struct SdfFontBakeCache {
    observed_font_generation: u64,
    fonts: HashMap<FontFaceId, fontsdf::Font>,
    glyphs: HashMap<SdfAtlasGlyphKey, RawBakedGlyph>,
    measured_glyphs: HashMap<SdfAtlasGlyphKey, SdfGlyphMetrics>,
    face_resolutions: HashMap<SdfAtlasGlyphKey, Vec<FontFaceId>>,
    font_asset_faces: HashMap<(u64, String), Option<FontFaceId>>,
    decoration_metrics: TextDecorationMetricsCache,
    offline_source: SdfOfflineSourceCache,
}

#[derive(Clone, Debug)]
pub(crate) struct SdfAtlasBake {
    pub(crate) pixels: Vec<u8>,
    pub(crate) pages: Vec<SdfAtlasBakePage>,
    pub(crate) glyphs: Vec<SdfBakedGlyph>,
    pub(crate) generation_failures: Vec<SdfAtlasGlyphGenerationFailure>,
    pub(crate) report: SdfAtlasBakeReport,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SdfAtlasBakePage {
    pub(crate) page_key: crate::text::atlas::GlyphAtlasPageKey,
    pub(crate) source_offset: usize,
    pub(crate) byte_len: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SdfAtlasGlyphGenerationFailure {
    pub(crate) slot_index: usize,
    pub(crate) key: SdfAtlasGlyphKey,
    pub(crate) error: SdfGlyphGenerationError,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SdfAtlasBakeReport {
    pub(crate) slot_count: usize,
    pub(crate) visible_glyph_count: usize,
    pub(crate) empty_glyph_count: usize,
    pub(crate) atlas_byte_len: usize,
    pub(crate) nonzero_pixel_count: usize,
    /// Materialized faces retained by the cache after this atlas build.
    pub(crate) resident_font_count: usize,
    /// Faces materialized by this atlas build rather than reused from the cache.
    pub(crate) loaded_font_count: usize,
    pub(crate) generation_failure_count: usize,
    pub(crate) r8_byte_len: usize,
    pub(crate) rgba_byte_len: usize,
    pub(crate) offline_glyph_count: usize,
    pub(crate) dynamic_glyph_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct SdfBakedGlyph {
    pub(crate) metrics: SdfGlyphMetrics,
    pub(crate) visible: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct SdfGlyphMetrics {
    pub(crate) bitmap_width: u32,
    pub(crate) bitmap_height: u32,
    pub(crate) bitmap_left: f32,
    pub(crate) bitmap_bottom: f32,
    pub(crate) advance: f32,
    pub(crate) ascent: f32,
}

impl SdfFontBakeCache {
    pub(crate) fn new() -> Self {
        Self {
            observed_font_generation: shared_font_database_generation(),
            fonts: HashMap::new(),
            glyphs: HashMap::new(),
            measured_glyphs: HashMap::new(),
            face_resolutions: HashMap::new(),
            font_asset_faces: HashMap::new(),
            decoration_metrics: TextDecorationMetricsCache::default(),
            offline_source: SdfOfflineSourceCache::default(),
        }
    }

    pub(crate) fn invalidate_faces(&mut self) {
        self.clear_face_derived_caches();
        self.observed_font_generation = shared_font_database_generation();
    }

    pub(crate) fn text_decoration_metrics<T: SdfTextRun + ?Sized>(
        &mut self,
        text: &T,
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) -> TextDecorationMetrics {
        self.ensure_current_font_generation();
        let requested_face =
            self.resolve_font_asset_face_cached(text.font(), font_database, asset_manager);
        let default_face = self.resolve_font_asset_face_cached(
            Some(DEFAULT_FONT_ASSET),
            font_database,
            asset_manager,
        );
        let primary_face = requested_face.or(default_face);
        let primary_metrics = primary_face
            .map(|face| {
                self.decoration_metrics
                    .resolve(font_database, face, text.font_size())
            })
            .unwrap_or_else(|| TextDecorationMetrics::fallback(text.font_size()));
        let mut resolved_faces = Vec::new();
        for (glyph_index, glyph) in text.render_scalars().into_iter().enumerate() {
            let shaped = text.shaped_glyph(glyph_index);
            let key = SdfAtlasGlyphKey {
                glyph,
                glyph_id: shaped.map(|glyph| glyph.glyph_id),
                font_id: shaped.and_then(|glyph| glyph.font_id),
                font_instance_id: shaped.and_then(|glyph| glyph.font_instance_id),
                font: text.font().map(str::to_owned),
                font_family: text.font_family().map(str::to_owned),
                language: text.language().map(str::to_owned),
                font_weight: text.font_weight(),
                bake_params: SdfBakeParams::default(),
            };
            if let Some(face) = self
                .resolve_faces_for_key_cached(&key, font_database, asset_manager)
                .into_iter()
                .next()
                .filter(|face| !resolved_faces.contains(face))
            {
                resolved_faces.push(face);
            }
        }
        primary_metrics.aggregate_fallback_thicknesses(resolved_faces.into_iter().map(|face| {
            self.decoration_metrics
                .resolve(font_database, face, text.font_size())
        }))
    }

    pub(crate) fn prepare_run_cpu<T: SdfTextRun + ?Sized>(
        &mut self,
        text: &T,
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) -> SdfRunCpuPreparation {
        self.ensure_current_font_generation();
        let decoration_metrics = self.text_decoration_metrics(text, font_database, asset_manager);
        let glyph_metrics = text
            .render_scalars()
            .into_iter()
            .enumerate()
            .map(|(glyph_index, glyph)| {
                if sdf_scalar_is_invisible_format(glyph) {
                    SdfGlyphMetrics::default()
                } else {
                    let shaped = text.shaped_glyph(glyph_index);
                    let key = SdfAtlasGlyphKey {
                        glyph,
                        glyph_id: shaped.map(|glyph| glyph.glyph_id),
                        font_id: shaped.and_then(|glyph| glyph.font_id),
                        font_instance_id: shaped.and_then(|glyph| glyph.font_instance_id),
                        font: text.font().map(str::to_owned),
                        font_family: text.font_family().map(str::to_owned),
                        language: text.language().map(str::to_owned),
                        font_weight: text.font_weight(),
                        bake_params: SdfBakeParams::default(),
                    };
                    let metrics = self.measure_key_cached(&key, font_database, asset_manager);
                    scale_sdf_metrics_for_display(metrics, text.font_size(), key.bake_params)
                }
            })
            .collect::<Vec<_>>();
        let glyph_advances = text.resolved_glyph_advances().unwrap_or_else(|| {
            glyph_metrics
                .iter()
                .map(|metrics| metrics.advance)
                .collect()
        });
        SdfRunCpuPreparation {
            glyph_metrics,
            glyph_advances,
            decoration_metrics,
        }
    }

    pub(crate) fn build_atlas_from_slots(
        &mut self,
        atlas_size: UVec2,
        slots: &[SdfAtlasSlot],
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) -> SdfAtlasBake {
        self.ensure_current_font_generation();
        let resident_font_count_before = self.fonts.len();
        let width = atlas_size.x.max(1);
        let height = atlas_size.y.max(1);
        let page_keys = slots
            .iter()
            .map(|slot| slot.page_key)
            .collect::<BTreeSet<_>>();
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
        let mut glyphs = Vec::with_capacity(slots.len());
        let mut generation_failures = Vec::new();
        let mut offline_glyph_count = 0_usize;
        let mut dynamic_glyph_count = 0_usize;

        for (slot_index, slot) in slots.iter().enumerate() {
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

    pub(crate) fn generation_failures_for_slots(
        &mut self,
        slots: &[SdfAtlasSlot],
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) -> HashMap<SdfAtlasGlyphKey, SdfGlyphGenerationError> {
        self.ensure_current_font_generation();
        slots
            .iter()
            .filter_map(|slot| {
                self.bake_glyph_cached(&slot.key, font_database, asset_manager)
                    .generation_error
                    .map(|error| (slot.key.clone(), error))
            })
            .collect()
    }

    pub(crate) fn measure_glyph(
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
        self.ensure_current_font_generation();
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
            font_weight: FontWeight::clamped(font_weight).0,
            bake_params: SdfBakeParams::default(),
        };
        let metrics = self.measure_key_cached(&key, font_database, asset_manager);
        scale_sdf_metrics_for_display(metrics, font_size, key.bake_params)
    }

    fn measure_key_cached(
        &mut self,
        key: &SdfAtlasGlyphKey,
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) -> SdfGlyphMetrics {
        if let Some(glyph) = self.glyphs.get(key) {
            return glyph.metrics;
        }
        if let Some(metrics) = self.measured_glyphs.get(key) {
            return *metrics;
        }
        let metrics = self.measure_key(key, font_database, asset_manager);
        self.measured_glyphs.insert(key.clone(), metrics);
        metrics
    }

    fn measure_key(
        &mut self,
        key: &SdfAtlasGlyphKey,
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) -> SdfGlyphMetrics {
        let px = key.bake_params.bake_em_px_f32();
        for face in self.resolve_faces_for_key_cached(key, font_database, asset_manager) {
            if !self.ensure_sdf_font(face, font_database) {
                continue;
            }
            let font = self.fonts.get(&face).expect("ensured SDF font");
            let index = glyph_index(font, key, face, font_database);
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
        self.measured_glyphs.insert(key.clone(), glyph.metrics);
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
        for face in self.resolve_faces_for_key_cached(key, font_database, asset_manager) {
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

    fn resolve_faces_for_key_cached(
        &mut self,
        key: &SdfAtlasGlyphKey,
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) -> Vec<FontFaceId> {
        if let Some(faces) = self.face_resolutions.get(key) {
            return faces.clone();
        }
        let faces = self.resolve_faces_for_key(key, font_database, asset_manager);
        self.face_resolutions.insert(key.clone(), faces.clone());
        faces
    }

    fn resolve_font_asset_face_cached(
        &mut self,
        font_asset: Option<&str>,
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) -> Option<FontFaceId> {
        let asset = font_asset
            .filter(|asset| !asset.trim().is_empty())
            .unwrap_or(DEFAULT_FONT_ASSET);
        let key = (shared_font_database_generation(), asset.to_owned());
        if let Some(face) = self.font_asset_faces.get(&key) {
            return *face;
        }
        let face = resolve_font_face(Some(asset), font_database, asset_manager);
        self.font_asset_faces.insert(key, face);
        face
    }

    fn resolve_faces_for_key(
        &mut self,
        key: &SdfAtlasGlyphKey,
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) -> Vec<FontFaceId> {
        if let Some(face) = shaped_face_for_key(key, font_database) {
            return font_database
                .standalone_face_bytes(face)
                .is_ok()
                .then_some(face)
                .into_iter()
                .collect();
        }
        let requested_face =
            self.resolve_font_asset_face_cached(key.font.as_deref(), font_database, asset_manager);
        let default_face = self.resolve_font_asset_face_cached(
            Some(DEFAULT_FONT_ASSET),
            font_database,
            asset_manager,
        );
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

    fn ensure_current_font_generation(&mut self) {
        self.sync_font_generation(shared_font_database_generation());
    }

    fn sync_font_generation(&mut self, generation: u64) {
        if self.observed_font_generation == generation {
            return;
        }
        self.clear_face_derived_caches();
        self.observed_font_generation = generation;
    }

    fn clear_face_derived_caches(&mut self) {
        self.fonts.clear();
        self.glyphs.clear();
        self.measured_glyphs.clear();
        self.face_resolutions.clear();
        self.font_asset_faces.clear();
        self.decoration_metrics = TextDecorationMetricsCache::default();
        self.offline_source = SdfOfflineSourceCache::default();
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

pub(crate) fn sdf_scalar_requires_atlas_slot(scalar: char) -> bool {
    !scalar.is_whitespace() && !sdf_scalar_is_invisible_format(scalar)
}

pub(crate) fn sdf_scalar_is_invisible_format(scalar: char) -> bool {
    matches!(
        scalar,
        '\u{061C}'
            | '\u{180E}'
            | '\u{200B}'..='\u{200F}'
            | '\u{202A}'..='\u{202E}'
            | '\u{2060}'..='\u{206F}'
            | '\u{FE00}'..='\u{FE0F}'
            | '\u{FEFF}'
            | '\u{E0100}'..='\u{E01EF}'
    )
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
    let manifest = load_text_font_source(asset, Some(asset_manager))?;
    register_loaded_font_manifest(font_database, &manifest)
}

fn register_loaded_font_manifest(
    font_database: &mut FontDatabase,
    manifest: &LoadedTextFontSource,
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

fn shaped_face_for_key(key: &SdfAtlasGlyphKey, font_database: &FontDatabase) -> Option<FontFaceId> {
    let face = key
        .font_id
        .and_then(crate::text::font::resolve_font_face_handle);
    let instance_face = key
        .font_instance_id
        .and_then(crate::text::font::resolve_font_instance_handle)
        .and_then(|instance| font_database.font_instance(instance))
        .map(|instance| instance.face);
    match (face, instance_face) {
        (Some(face), Some(instance_face)) if face != instance_face => None,
        (Some(face), _) => Some(face),
        (None, Some(instance_face)) => Some(instance_face),
        (None, None) => None,
    }
}

fn shaped_glyph_id_for_face(
    key: &SdfAtlasGlyphKey,
    face: FontFaceId,
    font_database: &FontDatabase,
) -> Option<u16> {
    (shaped_face_for_key(key, font_database) == Some(face))
        .then_some(())
        .and_then(|()| key.glyph_id)
        .and_then(|glyph_id| u16::try_from(glyph_id).ok())
}

fn glyph_index(
    font: &fontsdf::Font,
    key: &SdfAtlasGlyphKey,
    face: FontFaceId,
    font_database: &FontDatabase,
) -> u16 {
    if let Some(glyph_id) = shaped_glyph_id_for_face(key, face, font_database) {
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

pub(crate) fn scale_sdf_metrics_for_display(
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
