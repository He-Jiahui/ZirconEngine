use std::collections::HashMap;
use std::path::Path;

use glyphon::{FontSystem, SwashCache, TextArea, TextAtlas, TextRenderer, Viewport};

use crate::asset::{FontAsset, ProjectAssetManager};
use crate::core::math::UVec2;

use super::atlas::{GlyphAtlasBitmapRetryFrameState, GlyphAtlasSet};
use super::font::{
    mutate_shared_font_database, shared_font_database_generation, shared_font_database_snapshot,
    FontDatabase, MissingGlyphDiagnosticsReport,
};
use super::native_bitmap_atlas::{
    native_bitmap_atlas_frame, native_bitmap_atlas_idle_prepare_report, NativeBitmapAtlasFrame,
    NativeBitmapAtlasPrepareReport, NativeBitmapAtlasSourceCache, NativeBitmapAtlasTextArea,
};
use super::parallel::raster_pool::{TextRasterWorkerPool, TextRasterWorkerPoolOptions};
use super::sdf::{
    SdfAtlasBake, SdfAtlasGlyphKey, SdfAtlasSlot, SdfFontBakeCache, SdfGlyphGenerationError,
    SdfRunCpuPreparation, SdfTextRun,
};
use super::system_text_locale;

pub(crate) struct TextRenderState {
    font_system: FontSystem,
    font_database: FontDatabase,
    font_generation: u64,
    swash_cache: SwashCache,
    bitmap_source_cache: NativeBitmapAtlasSourceCache,
    bitmap_retry_state: GlyphAtlasBitmapRetryFrameState,
    bitmap_atlas: GlyphAtlasSet,
    bitmap_raster_worker_pool: Option<TextRasterWorkerPool>,
    bitmap_atlas_frame_index: u64,
    sdf_font_bake: SdfFontBakeCache,
}

impl TextRenderState {
    pub(crate) fn new(raster_worker_count: usize) -> Self {
        let (font_generation, font_database) = shared_font_database_snapshot();
        let font_system = FontSystem::new_with_locale_and_db(
            system_text_locale(),
            font_database.backend_database_snapshot(),
        );
        Self {
            font_system,
            font_database,
            font_generation,
            swash_cache: SwashCache::new(),
            bitmap_source_cache: NativeBitmapAtlasSourceCache::default(),
            bitmap_retry_state: GlyphAtlasBitmapRetryFrameState::new(),
            bitmap_atlas: GlyphAtlasSet::default(),
            bitmap_raster_worker_pool: TextRasterWorkerPool::new(TextRasterWorkerPoolOptions::new(
                raster_worker_count,
            ))
            .ok(),
            bitmap_atlas_frame_index: 0,
            sdf_font_bake: SdfFontBakeCache::new(),
        }
    }

    pub(crate) fn replace_font_source(
        &mut self,
        owner: &str,
        source_path: &Path,
        asset: Option<&FontAsset>,
        family: Option<&str>,
        face_index: u32,
    ) -> Option<super::font::FontAssetUpdateReport> {
        let (generation, database, result) = mutate_shared_font_database(|database| match asset {
            Some(asset) => database.replace_font_asset(owner, asset, source_path),
            None => database.replace_font_source(owner, source_path, family, face_index),
        });
        self.adopt_font_database(generation, database);
        let report = result.ok()?;
        if report.faces.is_empty() {
            return None;
        }
        Some(report)
    }

    pub(crate) fn remove_font_asset(&mut self, owner: &str) -> super::font::FontAssetUpdateReport {
        let (generation, database, report) =
            mutate_shared_font_database(|database| database.remove_font_asset(owner));
        self.adopt_font_database(generation, database);
        report
    }

    #[cfg(test)]
    pub(crate) fn face_count(&self) -> usize {
        self.font_database.face_count()
    }

    pub(crate) fn font_face_id(&self, backend: glyphon::fontdb::ID) -> Option<super::FontFaceId> {
        self.font_database.font_face_id(backend)
    }

    pub(crate) fn set_project_composite_font(
        &mut self,
        descriptor: Option<super::CompositeFontDescriptor>,
    ) -> bool {
        let (generation, database, changed) =
            mutate_shared_font_database(|database| database.set_project_composite_font(descriptor));
        self.adopt_font_database(generation, database);
        changed
    }

    pub(crate) fn set_default_ui_family(&mut self, family: &str) -> bool {
        let (generation, database, changed) =
            mutate_shared_font_database(|database| database.set_default_ui_family(family));
        self.adopt_font_database(generation, database);
        changed
    }

    pub(crate) fn set_default_ui_family_asset(&mut self, family: Option<&str>) -> bool {
        if let Some(family) = family {
            return self.set_default_ui_family(family);
        }
        let (generation, database, changed) =
            mutate_shared_font_database(|database| database.clear_default_ui_family());
        self.adopt_font_database(generation, database);
        changed
    }

    pub(crate) fn take_missing_glyph_diagnostics(&self) -> MissingGlyphDiagnosticsReport {
        self.font_database.take_missing_glyph_diagnostics()
    }

    /// Refresh a long-lived renderer only when another text owner advanced the
    /// authoritative font lineage. The atomic generation probe keeps the
    /// ordinary frame path free of a shared lock and FontDatabase clone.
    pub(crate) fn refresh_shared_font_database(&mut self) -> bool {
        if shared_font_database_generation() == self.font_generation {
            return false;
        }
        let (generation, database) = shared_font_database_snapshot();
        if generation == self.font_generation {
            return false;
        }
        self.adopt_font_database(generation, database);
        true
    }

    #[cfg(test)]
    pub(crate) fn font_database(&self) -> &FontDatabase {
        &self.font_database
    }

    fn adopt_font_database(&mut self, generation: u64, database: FontDatabase) {
        let render_inputs_changed = generation != self.font_generation;
        self.font_generation = generation;
        self.font_database = database;
        if render_inputs_changed {
            self.font_database.sync_font_system(&mut self.font_system);
        }
    }

    pub(crate) fn invalidate_font_faces(&mut self) {
        self.bitmap_source_cache.discard_all_for_face_invalidation();
        self.bitmap_retry_state.discard_all_for_face_invalidation();
        self.bitmap_atlas = GlyphAtlasSet::default();
        self.sdf_font_bake.invalidate_faces();
    }

    pub(crate) fn prepare_idle_bitmap_atlas(&mut self) -> NativeBitmapAtlasPrepareReport {
        self.bitmap_atlas = GlyphAtlasSet::default();
        self.bitmap_retry_state
            .replace_blocked_glyphs(std::iter::empty());
        native_bitmap_atlas_idle_prepare_report(
            &mut self.bitmap_source_cache,
            &mut self.bitmap_retry_state,
        )
    }

    fn advance_bitmap_atlas_frame_index(&mut self) {
        self.bitmap_atlas_frame_index = self.bitmap_atlas_frame_index.saturating_add(1).max(1);
    }

    pub(crate) fn prepare_bitmap_atlas(
        &mut self,
        viewport_size: UVec2,
        text_areas: &[NativeBitmapAtlasTextArea<'_, '_>],
    ) -> NativeBitmapAtlasFrame {
        self.advance_bitmap_atlas_frame_index();
        let frame = native_bitmap_atlas_frame(
            &mut self.font_system,
            &mut self.font_database,
            self.bitmap_raster_worker_pool.as_ref(),
            &mut self.bitmap_source_cache,
            &mut self.bitmap_retry_state,
            std::mem::take(&mut self.bitmap_atlas),
            viewport_size,
            self.bitmap_atlas_frame_index,
            text_areas,
        );
        self.bitmap_atlas = frame.submission.run.atlas.clone();
        frame
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare_glyphon_fallback(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        renderer: &mut TextRenderer,
        atlas: &mut TextAtlas,
        viewport: &Viewport,
        text_areas: Vec<TextArea<'_>>,
    ) {
        let _ = renderer.prepare(
            device,
            queue,
            &mut self.font_system,
            atlas,
            viewport,
            text_areas,
            &mut self.swash_cache,
        );
    }

    pub(super) fn with_native_text_backend<R>(
        &mut self,
        operation: impl FnOnce(&mut FontSystem, &FontDatabase) -> R,
    ) -> R {
        operation(&mut self.font_system, &self.font_database)
    }

    pub(crate) fn sdf_generation_failures(
        &mut self,
        slots: &[SdfAtlasSlot],
        asset_manager: &ProjectAssetManager,
    ) -> HashMap<SdfAtlasGlyphKey, SdfGlyphGenerationError> {
        self.sdf_font_bake.generation_failures_for_slots(
            slots,
            &mut self.font_database,
            asset_manager,
        )
    }

    pub(crate) fn build_sdf_atlas(
        &mut self,
        atlas_size: UVec2,
        slots: &[SdfAtlasSlot],
        asset_manager: &ProjectAssetManager,
    ) -> SdfAtlasBake {
        self.sdf_font_bake.build_atlas_from_slots(
            atlas_size,
            slots,
            &mut self.font_database,
            asset_manager,
        )
    }

    pub(crate) fn prepare_sdf_runs_cpu<T: SdfTextRun>(
        &mut self,
        texts: &[T],
        asset_manager: &ProjectAssetManager,
    ) -> Vec<SdfRunCpuPreparation> {
        texts
            .iter()
            .map(|text| {
                self.sdf_font_bake
                    .prepare_run_cpu(text, &mut self.font_database, asset_manager)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::TextRenderState;

    #[test]
    fn bitmap_atlas_frame_index_advances_monotonically_and_saturates() {
        let mut state = TextRenderState::new(0);

        state.advance_bitmap_atlas_frame_index();
        assert_eq!(state.bitmap_atlas_frame_index, 1);
        state.advance_bitmap_atlas_frame_index();
        assert_eq!(state.bitmap_atlas_frame_index, 2);

        state.bitmap_atlas_frame_index = u64::MAX;
        state.advance_bitmap_atlas_frame_index();
        assert_eq!(state.bitmap_atlas_frame_index, u64::MAX);
    }

    #[test]
    fn shared_font_database_refresh_adopts_another_renderer_mutation() {
        let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
        let mut reader = TextRenderState::new(0);
        let mut writer = TextRenderState::new(0);
        let previous_family = reader
            .font_database()
            .default_ui_family_for_test()
            .map(str::to_owned);

        assert!(writer.set_default_ui_family("Text Render State Refresh Family"));
        assert!(reader.refresh_shared_font_database());
        assert_eq!(
            reader.font_database().default_ui_family_for_test(),
            Some("Text Render State Refresh Family")
        );
        assert!(!reader.refresh_shared_font_database());

        let _ = writer.set_default_ui_family_asset(previous_family.as_deref());
        assert!(reader.refresh_shared_font_database());
    }
}
