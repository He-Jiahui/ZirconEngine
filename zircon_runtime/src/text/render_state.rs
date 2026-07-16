use std::collections::HashMap;
use std::path::Path;

use glyphon::{FontSystem, SwashCache, TextArea, TextAtlas, TextRenderer, Viewport};

use crate::asset::{FontAsset, ProjectAssetManager};
use crate::core::math::UVec2;

use super::atlas::{GlyphAtlasBitmapRetryFrameState, GlyphAtlasSet};
use super::font::{
    publish_shared_font_database, shared_font_database_snapshot, FontDatabase,
    MissingGlyphDiagnosticsReport, SystemFontPolicy,
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

pub(crate) struct TextRenderState {
    font_system: FontSystem,
    font_database: FontDatabase,
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
        let mut font_system = FontSystem::new();
        let (_, mut font_database) = shared_font_database_snapshot();
        font_database.apply_system_font_policy(SystemFontPolicy::Discover);
        font_database.sync_font_system(&mut font_system);
        Self {
            font_system,
            font_database,
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

    pub(crate) fn register_font_source(
        &mut self,
        source_path: &Path,
        asset: Option<&FontAsset>,
        family: Option<&str>,
        face_index: u32,
    ) -> bool {
        let face = match asset {
            Some(asset) => self
                .font_database
                .register_font_asset(asset, source_path)
                .ok()
                .and_then(|faces| faces.first().copied()),
            None => self
                .font_database
                .register_font_file(source_path, family, face_index)
                .ok(),
        };
        face.is_some_and(|face| {
            self.font_database
                .load_face_into_font_system(face, &mut self.font_system)
                .is_ok()
        })
    }

    pub(crate) fn face_count(&self) -> usize {
        self.font_database.face_count()
    }

    pub(crate) fn set_project_composite_font(
        &mut self,
        descriptor: Option<super::CompositeFontDescriptor>,
    ) {
        self.font_database.set_project_composite_font(descriptor);
    }

    pub(crate) fn set_default_ui_family(&mut self, family: &str) {
        self.font_database.set_default_ui_family(family);
        self.font_system
            .db_mut()
            .set_sans_serif_family(family.to_string());
        self.font_system
            .db_mut()
            .set_monospace_family(family.to_string());
    }

    pub(crate) fn publish_font_database(&self) -> u64 {
        publish_shared_font_database(&self.font_database)
    }

    pub(crate) fn take_missing_glyph_diagnostics(&self) -> MissingGlyphDiagnosticsReport {
        self.font_database.take_missing_glyph_diagnostics()
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
}
