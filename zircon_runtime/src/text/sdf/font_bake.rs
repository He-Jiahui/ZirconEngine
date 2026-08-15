use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use crate::asset::ProjectAssetManager;
use crate::text::font::{
    load_text_font_source, resolve_font_handle_batch, shared_font_database_generation,
    FontDatabase, LoadedTextFontSource, TextDecorationMetrics, TextDecorationMetricsCache,
};
use crate::text::sdf::{SdfBakeParams, SdfGenerationScheduler, SdfGlyphGenerationError};
use crate::text::{
    FontFaceId, FontFamilyName, FontQuery, FontStretch, FontStyle, FontWeight, InstancedFaceId,
};

mod async_batch;
mod atlas_build;
mod atlas_pages;
mod distance_field;
mod dynamic_batch;
mod font_asset_cache;
mod glyph_cache;
mod glyph_metrics;
mod model;
mod offline_source;
mod prepared_atlas;
mod source_context;

use atlas_pages::SdfPersistentAtlasCache;
use dynamic_batch::SdfDynamicGenerationTotals;
use font_asset_cache::SdfFontAssetFaceCache;
use glyph_metrics::{fallback_metrics, glyph_metrics};
use offline_source::{SdfOfflineSourceCache, SdfOfflineSourceCacheReport};
use prepared_atlas::SdfPreparedAtlasCache;
use source_context::{SdfGenerationSourceCache, SdfGenerationSourceCacheReport};

pub(crate) use glyph_metrics::scale_sdf_metrics_for_display;
pub(crate) use model::{
    SdfAtlasBake, SdfAtlasBakeDirtyPage, SdfAtlasBakePage, SdfAtlasBakeReport,
    SdfAtlasGlyphGenerationFailure, SdfAtlasGlyphKey, SdfAtlasRect, SdfAtlasSlot, SdfBakedGlyph,
    SdfGlyphMetrics, SdfRunCpuPreparation, SdfShapedGlyphIdentity, SdfTextRun,
};

const DEFAULT_FONT_ASSET: &str = "res://fonts/default.font.toml";
const FALLBACK_ADVANCE_RATIO: f32 = 0.6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SdfShapedFontResolution {
    face: FontFaceId,
    instance: Option<InstancedFaceId>,
}

pub(crate) struct SdfFontBakeCache {
    observed_font_generation: u64,
    fonts: HashMap<FontFaceId, fontsdf::Font>,
    glyphs: HashMap<SdfAtlasGlyphKey, RawBakedGlyph>,
    // One LRU key domain bounds bitmap glyphs and all CPU-side glyph sidecars together.
    baked_glyph_recency: HashMap<SdfAtlasGlyphKey, u64>,
    baked_glyph_recency_order: BTreeSet<(u64, SdfAtlasGlyphKey)>,
    baked_glyph_access_epoch: u64,
    resident_baked_glyph_byte_count: usize,
    baked_glyph_eviction_count: usize,
    reported_baked_glyph_eviction_count: usize,
    measured_glyphs: HashMap<SdfAtlasGlyphKey, SdfGlyphMetrics>,
    face_resolutions: HashMap<SdfAtlasGlyphKey, Vec<FontFaceId>>,
    shaped_face_resolutions: HashMap<SdfAtlasGlyphKey, Option<SdfShapedFontResolution>>,
    font_asset_faces: SdfFontAssetFaceCache,
    decoration_metrics: TextDecorationMetricsCache,
    offline_source: SdfOfflineSourceCache,
    source_contexts: SdfGenerationSourceCache,
    dynamic_generation_totals: SdfDynamicGenerationTotals,
    reported_source_contexts: SdfGenerationSourceCacheReport,
    reported_dynamic_generation: SdfDynamicGenerationTotals,
    reported_offline_source: SdfOfflineSourceCacheReport,
    atlas_pages: SdfPersistentAtlasCache,
    prepared_atlas: SdfPreparedAtlasCache,
    async_generation: SdfAsyncGenerationState,
}

impl SdfFontBakeCache {
    pub(crate) fn new() -> Self {
        let observed_font_generation = shared_font_database_generation();
        Self {
            observed_font_generation,
            fonts: HashMap::new(),
            glyphs: HashMap::new(),
            baked_glyph_recency: HashMap::new(),
            baked_glyph_recency_order: BTreeSet::new(),
            baked_glyph_access_epoch: 0,
            resident_baked_glyph_byte_count: 0,
            baked_glyph_eviction_count: 0,
            reported_baked_glyph_eviction_count: 0,
            measured_glyphs: HashMap::new(),
            face_resolutions: HashMap::new(),
            shaped_face_resolutions: HashMap::new(),
            font_asset_faces: SdfFontAssetFaceCache::default(),
            decoration_metrics: TextDecorationMetricsCache::default(),
            offline_source: SdfOfflineSourceCache::default(),
            source_contexts: SdfGenerationSourceCache::new(observed_font_generation),
            dynamic_generation_totals: SdfDynamicGenerationTotals::default(),
            reported_source_contexts: SdfGenerationSourceCacheReport::default(),
            reported_dynamic_generation: SdfDynamicGenerationTotals::default(),
            reported_offline_source: SdfOfflineSourceCacheReport::default(),
            atlas_pages: SdfPersistentAtlasCache::default(),
            prepared_atlas: SdfPreparedAtlasCache::default(),
            async_generation: SdfAsyncGenerationState::default(),
        }
    }

    pub(crate) fn invalidate_faces(&mut self) {
        let generation = shared_font_database_generation();
        self.clear_face_derived_caches(generation);
        self.observed_font_generation = generation;
    }

    pub(crate) fn cancel_scheduled_generation(&mut self, scheduler: &SdfGenerationScheduler) {
        self.cancel_async_generation(scheduler);
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
        let font = text.font().map(Arc::<str>::from);
        let font_family = text.font_family().map(Arc::<str>::from);
        let language = text.language().map(Arc::<str>::from);
        let keys = text
            .render_scalars()
            .into_iter()
            .enumerate()
            .map(|(glyph_index, glyph)| {
                let shaped = text.shaped_glyph(glyph_index);
                SdfAtlasGlyphKey {
                    glyph,
                    glyph_id: shaped.map(|glyph| glyph.glyph_id),
                    font_id: shaped.and_then(|glyph| glyph.font_id),
                    font_instance_id: shaped.and_then(|glyph| glyph.font_instance_id),
                    font: font.clone(),
                    font_family: font_family.clone(),
                    language: language.clone(),
                    font_weight: text.font_weight(),
                    bake_params: SdfBakeParams::default(),
                }
            })
            .collect::<Vec<_>>();
        self.prime_shaped_face_resolutions(&keys, font_database);
        let mut resolved_faces = Vec::new();
        for key in &keys {
            if let Some(face) = self
                .resolve_faces_for_key_cached(key, font_database, asset_manager)
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
        let font = text.font().map(Arc::<str>::from);
        let font_family = text.font_family().map(Arc::<str>::from);
        let language = text.language().map(Arc::<str>::from);
        let keys = text
            .render_scalars()
            .into_iter()
            .enumerate()
            .map(|(glyph_index, glyph)| {
                let shaped = text.shaped_glyph(glyph_index);
                SdfAtlasGlyphKey {
                    glyph,
                    glyph_id: shaped.map(|glyph| glyph.glyph_id),
                    font_id: shaped.and_then(|glyph| glyph.font_id),
                    font_instance_id: shaped.and_then(|glyph| glyph.font_instance_id),
                    font: font.clone(),
                    font_family: font_family.clone(),
                    language: language.clone(),
                    font_weight: text.font_weight(),
                    bake_params: SdfBakeParams::default(),
                }
            })
            .collect::<Vec<_>>();
        self.prime_shaped_face_resolutions(&keys, font_database);
        let glyph_metrics = keys
            .iter()
            .map(|key| {
                if sdf_scalar_is_invisible_format(key.glyph) {
                    SdfGlyphMetrics::default()
                } else {
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
            font: font.map(Arc::<str>::from),
            font_family: font_family.map(Arc::<str>::from),
            language: language
                .map(str::trim)
                .filter(|language| !language.is_empty())
                .map(Arc::<str>::from),
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
        if let Some(glyph) = self.cached_baked_glyph(key) {
            return glyph.metrics;
        }
        if let Some(metrics) = self.measured_glyphs.get(key) {
            let metrics = *metrics;
            self.touch_cached_glyph_key(key.clone());
            return metrics;
        }
        let metrics = self.measure_key(key, font_database, asset_manager);
        self.measured_glyphs.insert(key.clone(), metrics);
        self.touch_cached_glyph_key(key.clone());
        self.enforce_baked_glyph_budget(&[]);
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
            let Some(font) = self.fonts.get(&face) else {
                continue;
            };
            let resolved_shaped_face = self
                .shaped_face_resolutions
                .get(key)
                .copied()
                .flatten()
                .map(|resolution| resolution.face);
            let index = glyph_index(font, key, face, resolved_shaped_face);
            let metrics = font.metrics_indexed_sdf(index, px);
            return glyph_metrics(font, px, metrics);
        }
        fallback_metrics(px)
    }

    fn bake_glyph_cached(&mut self, key: &SdfAtlasGlyphKey) -> RawBakedGlyph {
        if let Some(glyph) = self.cached_baked_glyph(key) {
            return glyph;
        }
        if let Some(error) = self.async_generation.transient_failure(key) {
            return RawBakedGlyph::failed(
                fallback_metrics(key.bake_params.bake_em_px_f32()),
                error,
            );
        }
        RawBakedGlyph::failed(
            fallback_metrics(key.bake_params.bake_em_px_f32()),
            SdfGlyphGenerationError::MissingGlyphOutline(
                key.glyph_id
                    .and_then(|glyph_id| u16::try_from(glyph_id).ok())
                    .unwrap_or(0),
            ),
        )
    }

    fn resolve_faces_for_key_cached(
        &mut self,
        key: &SdfAtlasGlyphKey,
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) -> Vec<FontFaceId> {
        if let Some(faces) = self.face_resolutions.get(key).cloned() {
            self.touch_cached_glyph_key(key.clone());
            return faces;
        }
        let faces = self.resolve_faces_for_key(key, font_database, asset_manager);
        self.face_resolutions.insert(key.clone(), faces.clone());
        self.touch_cached_glyph_key(key.clone());
        self.enforce_baked_glyph_budget(&[]);
        faces
    }

    fn resolve_font_asset_face_cached(
        &mut self,
        font_asset: Option<&str>,
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) -> Option<FontFaceId> {
        self.font_asset_faces
            .resolve(font_asset, font_database, asset_manager)
    }

    fn resolve_faces_for_key(
        &mut self,
        key: &SdfAtlasGlyphKey,
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) -> Vec<FontFaceId> {
        self.prime_shaped_face_resolutions(std::slice::from_ref(key), font_database);
        if let Some(face) = self
            .shaped_face_resolutions
            .get(key)
            .copied()
            .flatten()
            .map(|resolution| resolution.face)
        {
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

    fn prime_shaped_face_resolutions(
        &mut self,
        keys: &[SdfAtlasGlyphKey],
        font_database: &FontDatabase,
    ) {
        let unshaped = keys
            .iter()
            .filter(|key| {
                key.font_id.is_none()
                    && key.font_instance_id.is_none()
                    && !self.shaped_face_resolutions.contains_key(*key)
            })
            .cloned()
            .collect::<Vec<_>>();
        for key in unshaped {
            self.shaped_face_resolutions.insert(key.clone(), None);
            self.touch_cached_glyph_key(key);
        }
        let pending = keys
            .iter()
            .filter(|key| {
                (key.font_id.is_some() || key.font_instance_id.is_some())
                    && !self.shaped_face_resolutions.contains_key(*key)
            })
            .collect::<Vec<_>>();
        if !pending.is_empty() {
            let pairs = pending
                .iter()
                .map(|key| (key.font_id, key.font_instance_id))
                .collect::<Vec<_>>();
            for (key, (face, instance_id)) in
                pending.into_iter().zip(resolve_font_handle_batch(&pairs))
            {
                let instance_face = instance_id
                    .and_then(|instance| font_database.font_instance(instance))
                    .map(|instance| instance.face);
                let resolved = match (face, instance_id, instance_face) {
                    (Some(face), None, None) => Some(SdfShapedFontResolution {
                        face,
                        instance: None,
                    }),
                    (Some(face), Some(instance), Some(instance_face)) if face == instance_face => {
                        Some(SdfShapedFontResolution {
                            face,
                            instance: Some(instance),
                        })
                    }
                    (None, Some(instance), Some(instance_face)) => Some(SdfShapedFontResolution {
                        face: instance_face,
                        instance: Some(instance),
                    }),
                    _ => None,
                };
                self.shaped_face_resolutions.insert(key.clone(), resolved);
                self.touch_cached_glyph_key(key.clone());
            }
        }
        self.enforce_baked_glyph_budget(&[]);
    }

    fn ensure_current_font_generation(&mut self) {
        self.sync_font_generation(shared_font_database_generation());
    }

    fn ensure_current_font_generation_scheduled(&mut self, scheduler: &SdfGenerationScheduler) {
        let generation = shared_font_database_generation();
        if self.observed_font_generation != generation {
            self.cancel_async_generation(scheduler);
        }
        self.sync_font_generation(generation);
    }

    fn sync_font_generation(&mut self, generation: u64) {
        if self.observed_font_generation == generation {
            return;
        }
        self.clear_face_derived_caches(generation);
        self.observed_font_generation = generation;
    }

    fn clear_face_derived_caches(&mut self, generation: u64) {
        self.fonts.clear();
        self.clear_cached_glyph_entries();
        self.font_asset_faces.clear();
        self.decoration_metrics = TextDecorationMetricsCache::default();
        self.offline_source = SdfOfflineSourceCache::default();
        self.source_contexts = SdfGenerationSourceCache::new(generation);
        self.dynamic_generation_totals = SdfDynamicGenerationTotals::default();
        self.reported_source_contexts = SdfGenerationSourceCacheReport::default();
        self.reported_dynamic_generation = SdfDynamicGenerationTotals::default();
        self.reported_offline_source = SdfOfflineSourceCacheReport::default();
        self.atlas_pages = SdfPersistentAtlasCache::default();
        self.prepared_atlas = SdfPreparedAtlasCache::default();
        self.async_generation = SdfAsyncGenerationState::default();
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
    bitmap: Arc<[u8]>,
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
            bitmap: Arc::from([]),
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
    if let Some(face) = font_database.font_asset_primary_face(asset) {
        return Some(face);
    }
    let Some(manifest) = load_text_font_source(asset, Some(asset_manager)) else {
        font_database.remove_font_asset(asset);
        return None;
    };
    register_loaded_font_manifest(font_database, asset, &manifest)
}

fn register_loaded_font_manifest(
    font_database: &mut FontDatabase,
    asset_ref: &str,
    manifest: &LoadedTextFontSource,
) -> Option<FontFaceId> {
    if let Some(asset) = &manifest.asset {
        return font_database
            .replace_font_asset(asset_ref, asset, &manifest.source_path)
            .ok()
            .and_then(|report| report.faces.first().copied());
    }

    font_database
        .replace_font_source(
            asset_ref,
            &manifest.source_path,
            manifest.family.as_deref(),
            manifest.face_index,
        )
        .ok()
        .and_then(|report| report.faces.first().copied())
}

fn shaped_glyph_id_for_face(
    key: &SdfAtlasGlyphKey,
    face: FontFaceId,
    resolved_shaped_face: Option<FontFaceId>,
) -> Option<u16> {
    (resolved_shaped_face == Some(face))
        .then_some(())
        .and_then(|()| key.glyph_id)
        .and_then(|glyph_id| u16::try_from(glyph_id).ok())
}

fn glyph_index(
    font: &fontsdf::Font,
    key: &SdfAtlasGlyphKey,
    face: FontFaceId,
    resolved_shaped_face: Option<FontFaceId>,
) -> u16 {
    if let Some(glyph_id) = shaped_glyph_id_for_face(key, face, resolved_shaped_face) {
        return glyph_id;
    }
    if font.chars().contains_key(&key.glyph) {
        font.lookup_glyph_index(key.glyph)
    } else {
        0
    }
}

#[cfg(test)]
mod tests;
use async_batch::SdfAsyncGenerationState;
