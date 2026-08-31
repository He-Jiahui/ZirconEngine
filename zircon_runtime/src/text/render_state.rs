use std::sync::Arc;

use crate::asset::ProjectAssetManager;
use crate::core::math::UVec2;
use crate::core::runtime::tasks::TaskPools;

use super::atlas::{
    GlyphAtlasBitmapPageShadowCommit, GlyphAtlasBitmapRetryFrameState, GlyphAtlasSet,
};
use super::font::{
    FontCollectionRevision, FontCollectionService, FontDatabase, MissingGlyphDiagnosticsReport,
    TextDecorationMetrics, shared_font_collection_service,
};
use super::native_bitmap_atlas::{
    NativeBitmapAtlasFrame, NativeBitmapAtlasGlyphRun, NativeBitmapAtlasPrepareReport,
    NativeBitmapAtlasSourceCache, native_bitmap_atlas_frame,
    native_bitmap_atlas_idle_prepare_report,
};
use super::parallel::raster_pool::{
    TextRasterThreadBudgetSource, TextRasterWorkerPool, TextRasterWorkerPoolOptions,
};
use super::sdf::{
    SdfAtlasBake, SdfAtlasGlyphKey, SdfAtlasSlot, SdfFontBakeCache, SdfGenerationScheduler,
    SdfGenerationSchedulerOptions, SdfRunCpuPreparation, SdfTextRun,
};

pub(crate) struct TextRenderState {
    font_collection: Arc<FontCollectionService>,
    font_database: FontDatabase,
    font_generation: u64,
    bitmap_source_cache: NativeBitmapAtlasSourceCache,
    bitmap_retry_state: GlyphAtlasBitmapRetryFrameState,
    bitmap_atlas: GlyphAtlasSet,
    bitmap_raster_worker_pool: Option<TextRasterWorkerPool>,
    bitmap_atlas_frame_index: u64,
    sdf_font_bake: SdfFontBakeCache,
    sdf_generation_scheduler: Option<SdfGenerationScheduler>,
    sdf_generation_frame_index: u64,
}

impl TextRenderState {
    pub(crate) fn new(raster_worker_count: usize) -> Self {
        Self::new_with_font_collection(raster_worker_count, shared_font_collection_service())
    }

    pub(crate) fn new_with_font_collection(
        raster_worker_count: usize,
        font_collection: Arc<FontCollectionService>,
    ) -> Self {
        Self::new_with_raster_worker_options(
            TextRasterWorkerPoolOptions::new(raster_worker_count),
            None,
            font_collection,
        )
    }

    pub(crate) fn new_with_process_raster_worker_budget() -> Self {
        Self::new_with_font_collection_and_process_raster_worker_budget(
            shared_font_collection_service(),
        )
    }

    pub(crate) fn new_with_font_collection_and_process_raster_worker_budget(
        font_collection: Arc<FontCollectionService>,
    ) -> Self {
        let task_pools = TaskPools::process_default();
        let sdf_parallelism = task_pools.compute().parallelism();
        let sdf_generation_scheduler = SdfGenerationScheduler::new(
            task_pools.compute().clone(),
            SdfGenerationSchedulerOptions::new(sdf_parallelism.saturating_mul(2)),
        );
        Self::new_with_raster_worker_options(
            Self::process_raster_worker_options(&task_pools),
            Some(sdf_generation_scheduler),
            font_collection,
        )
    }

    fn process_raster_worker_options(task_pools: &TaskPools) -> TextRasterWorkerPoolOptions {
        let worker_count = task_pools.thread_counts().async_compute_threads;
        TextRasterWorkerPoolOptions::new(worker_count)
            .with_thread_budget_source(TextRasterThreadBudgetSource::TaskPoolAsyncCompute)
    }

    fn new_with_raster_worker_options(
        raster_worker_options: TextRasterWorkerPoolOptions,
        sdf_generation_scheduler: Option<SdfGenerationScheduler>,
        font_collection: Arc<FontCollectionService>,
    ) -> Self {
        let (font_generation, font_database) = font_collection.snapshot();
        let sdf_font_bake =
            SdfFontBakeCache::new_with_font_collection(Arc::clone(&font_collection));
        Self {
            font_collection,
            font_database,
            font_generation,
            bitmap_source_cache: NativeBitmapAtlasSourceCache::default(),
            bitmap_retry_state: GlyphAtlasBitmapRetryFrameState::new(),
            bitmap_atlas: GlyphAtlasSet::default(),
            bitmap_raster_worker_pool: TextRasterWorkerPool::new(raster_worker_options).ok(),
            bitmap_atlas_frame_index: 0,
            sdf_font_bake,
            sdf_generation_scheduler,
            sdf_generation_frame_index: 0,
        }
    }

    #[cfg(test)]
    pub(crate) fn face_count(&self) -> usize {
        self.font_database.face_count()
    }

    pub(crate) fn take_missing_glyph_diagnostics(&self) -> MissingGlyphDiagnosticsReport {
        self.font_database.take_missing_glyph_diagnostics()
    }

    pub(crate) fn font_collection_revision(&self) -> FontCollectionRevision {
        FontCollectionRevision::new(self.font_collection.collection_id(), self.font_generation)
    }

    pub(crate) fn published_font_collection_revision(&self) -> FontCollectionRevision {
        self.font_collection.revision()
    }

    pub(crate) fn font_collection(&self) -> Arc<FontCollectionService> {
        Arc::clone(&self.font_collection)
    }

    /// Refresh a long-lived renderer only when another text owner advanced the
    /// authoritative font lineage. The atomic generation probe keeps the
    /// ordinary frame path free of a shared lock and FontDatabase clone.
    pub(crate) fn refresh_font_collection(&mut self) -> bool {
        if self.font_collection.generation() == self.font_generation {
            return false;
        }
        let (generation, database) = self.font_collection.snapshot();
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
            if let Some(scheduler) = self.sdf_generation_scheduler.as_ref() {
                self.sdf_font_bake.cancel_scheduled_generation(scheduler);
            }
        }
    }

    pub(crate) fn invalidate_font_faces(&mut self) {
        self.bitmap_source_cache
            .discard_all_for_face_invalidation_with_worker_pool(
                self.bitmap_raster_worker_pool.as_ref(),
            );
        self.bitmap_retry_state.discard_all_for_face_invalidation();
        self.bitmap_atlas = GlyphAtlasSet::default();
        if let Some(scheduler) = self.sdf_generation_scheduler.as_ref() {
            self.sdf_font_bake.cancel_scheduled_generation(scheduler);
        }
        self.sdf_font_bake.invalidate_faces();
    }

    pub(crate) fn prepare_idle_bitmap_atlas(&mut self) -> NativeBitmapAtlasPrepareReport {
        self.bitmap_retry_state
            .replace_blocked_glyphs(std::iter::empty());
        let mut report = native_bitmap_atlas_idle_prepare_report(
            &mut self.bitmap_source_cache,
            &mut self.bitmap_retry_state,
        );
        if let Some(pool) = self.bitmap_raster_worker_pool.as_ref() {
            report
                .source_cache
                .record_worker_pool_diagnostics(pool.diagnostics());
        }
        report
    }

    fn advance_bitmap_atlas_frame_index(&mut self) {
        self.bitmap_atlas_frame_index = self.bitmap_atlas_frame_index.saturating_add(1).max(1);
    }

    /// Begins one UI-text prepare frame for SDF/MSDF generation scheduling.
    ///
    /// Native bitmap preparation may be absent on an SDF-only frame, so its atlas clock cannot
    /// define SDF completion draining, retry, or age accounting.
    pub(crate) fn begin_sdf_generation_frame(&mut self) {
        self.sdf_generation_frame_index = self.sdf_generation_frame_index.saturating_add(1).max(1);
    }

    pub(crate) fn prepare_bitmap_atlas<'a, GlyphRuns>(
        &mut self,
        viewport_size: UVec2,
        glyph_runs: GlyphRuns,
    ) -> NativeBitmapAtlasFrame
    where
        GlyphRuns: Clone + IntoIterator<Item = &'a NativeBitmapAtlasGlyphRun>,
    {
        self.advance_bitmap_atlas_frame_index();
        let frame = native_bitmap_atlas_frame(
            &mut self.font_database,
            self.bitmap_raster_worker_pool.as_ref(),
            &mut self.bitmap_source_cache,
            &mut self.bitmap_retry_state,
            std::mem::take(&mut self.bitmap_atlas),
            viewport_size,
            self.bitmap_atlas_frame_index,
            glyph_runs,
        );
        frame
    }

    /// Return atlas ownership only after the renderer has accepted every
    /// texture write needed for newly allocated slots. A failed native handoff
    /// invalidates only the affected pages, so an unwritten slot can never
    /// become a hit while stable pages avoid a per-frame slot-cache clone.
    pub(crate) fn finish_bitmap_atlas_frame(
        &mut self,
        mut frame: NativeBitmapAtlasFrame,
        shadow_commit: GlyphAtlasBitmapPageShadowCommit,
        accept_frame_atlas: bool,
    ) {
        if !accept_frame_atlas {
            let invalidated_page_keys = frame
                .submission
                .run
                .upload_copies
                .iter()
                .map(|copy| copy.page_key)
                .collect::<Vec<_>>();
            let mut atlas = frame.submission.run.atlas;
            let invalidated_raster_keys =
                atlas.invalidate_bitmap_page_upload_state(invalidated_page_keys);
            self.bitmap_source_cache
                .invalidate_raster_keys_for_next_frame(invalidated_raster_keys);
            self.bitmap_atlas = atlas;
            return;
        }
        frame
            .submission
            .run
            .atlas
            .commit_bitmap_page_shadow(shadow_commit);
        self.bitmap_atlas = frame.submission.run.atlas;
    }

    pub(crate) fn build_sdf_atlas(
        &mut self,
        atlas_size: UVec2,
        slots: &[SdfAtlasSlot],
        asset_manager: &ProjectAssetManager,
    ) -> SdfAtlasBake {
        if let Some(scheduler) = self.sdf_generation_scheduler.as_ref() {
            self.sdf_font_bake.build_atlas_from_slots_scheduled(
                atlas_size,
                slots,
                &mut self.font_database,
                asset_manager,
                scheduler,
                self.sdf_generation_frame_index,
            )
        } else {
            self.sdf_font_bake.build_atlas_from_slots(
                atlas_size,
                slots,
                &mut self.font_database,
                asset_manager,
            )
        }
    }

    pub(crate) fn prepare_sdf_runs_cpu<T: SdfTextRun>(
        &mut self,
        texts: &[T],
        asset_manager: &ProjectAssetManager,
    ) -> Vec<SdfRunCpuPreparation> {
        let mut runs = Vec::new();
        self.prepare_sdf_runs_cpu_into(texts, asset_manager, &mut runs);
        runs
    }

    pub(crate) fn prepare_sdf_runs_cpu_into<T: SdfTextRun>(
        &mut self,
        texts: &[T],
        asset_manager: &ProjectAssetManager,
        runs: &mut Vec<SdfRunCpuPreparation>,
    ) {
        self.prepare_sdf_runs_cpu_iter_into(texts.iter(), asset_manager, runs);
    }

    pub(crate) fn prepare_sdf_runs_cpu_iter_into<'a, T, Texts>(
        &mut self,
        texts: Texts,
        asset_manager: &ProjectAssetManager,
        runs: &mut Vec<SdfRunCpuPreparation>,
    ) where
        T: SdfTextRun + 'a,
        Texts: IntoIterator<Item = &'a T>,
    {
        runs.clear();
        runs.extend(texts.into_iter().map(|text| {
            self.sdf_font_bake
                .prepare_run_cpu(text, &mut self.font_database, asset_manager)
        }));
    }

    pub(crate) fn prepare_sdf_decoration_metrics_into<T: SdfTextRun>(
        &mut self,
        texts: &[T],
        asset_manager: &ProjectAssetManager,
        metrics: &mut Vec<TextDecorationMetrics>,
    ) {
        self.prepare_sdf_decoration_metrics_iter_into(texts.iter(), asset_manager, metrics);
    }

    pub(crate) fn prepare_sdf_decoration_metrics_iter_into<'a, T, Texts>(
        &mut self,
        texts: Texts,
        asset_manager: &ProjectAssetManager,
        metrics: &mut Vec<TextDecorationMetrics>,
    ) where
        T: SdfTextRun + 'a,
        Texts: IntoIterator<Item = &'a T>,
    {
        metrics.clear();
        metrics.extend(texts.into_iter().map(|text| {
            self.sdf_font_bake
                .text_decoration_metrics(text, &mut self.font_database, asset_manager)
        }))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::TextRenderState;
    use crate::core::math::UVec2;
    use crate::core::runtime::tasks::TaskPools;
    use crate::text::atlas::{
        GlyphAtlasFormat, GlyphAtlasPageKey, GlyphAtlasPageSpec, GlyphAtlasSet,
    };
    use crate::text::font::{FontCollectionService, runtime_default_font_database_for_test};
    use crate::text::parallel::raster_pool::TextRasterThreadBudgetSource;

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
    fn sdf_generation_frame_index_is_independent_of_native_bitmap_preparation() {
        let mut state = TextRenderState::new(0);

        state.begin_sdf_generation_frame();
        assert_eq!(state.sdf_generation_frame_index, 1);
        assert_eq!(state.bitmap_atlas_frame_index, 0);

        state.advance_bitmap_atlas_frame_index();
        assert_eq!(state.bitmap_atlas_frame_index, 1);
        assert_eq!(state.sdf_generation_frame_index, 1);

        state.begin_sdf_generation_frame();
        assert_eq!(state.sdf_generation_frame_index, 2);
        assert_eq!(state.bitmap_atlas_frame_index, 1);

        state.sdf_generation_frame_index = u64::MAX;
        state.begin_sdf_generation_frame();
        assert_eq!(state.sdf_generation_frame_index, u64::MAX);
    }

    #[test]
    fn process_raster_workers_follow_the_async_compute_budget() {
        let task_pools = TaskPools::process_default();
        let options = TextRenderState::process_raster_worker_options(&task_pools);
        let expected_workers = task_pools.thread_counts().async_compute_threads;

        assert_eq!(options.worker_count, expected_workers);
        assert_eq!(
            options.thread_budget_source,
            TextRasterThreadBudgetSource::TaskPoolAsyncCompute
        );
    }

    #[test]
    fn process_text_state_owns_global_task_pool_sdf_scheduler() {
        let state = TextRenderState::new_with_process_raster_worker_budget();

        assert!(state.sdf_generation_scheduler.is_some());
    }

    #[test]
    fn idle_bitmap_prepare_keeps_persistent_atlas_pages_resident() {
        let mut state = TextRenderState::new(0);
        let page = GlyphAtlasPageSpec::new(
            GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0),
            UVec2::new(64, 64),
        );
        state.bitmap_atlas = GlyphAtlasSet::from_page(page);

        let report = state.prepare_idle_bitmap_atlas();

        assert_eq!(report.source_cache.entry_count, 0);
        assert_eq!(state.bitmap_atlas.page_count(), 1);
    }

    #[test]
    fn renderers_bound_to_one_collection_observe_its_publications() {
        let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
        let mut reader = TextRenderState::new(0);
        let writer = TextRenderState::new(0);
        let previous_project_family = reader
            .font_database()
            .project_default_ui_family_for_test()
            .map(str::to_owned);

        let writer_collection = writer.font_collection();
        let (_, _, changed) = writer_collection
            .mutate(|database| database.set_default_ui_family("Text Render State Refresh Family"));
        assert!(changed);
        assert!(reader.refresh_font_collection());
        assert_eq!(
            reader.font_database().default_ui_family_for_test(),
            Some("Text Render State Refresh Family")
        );
        assert!(!reader.refresh_font_collection());

        let (_, _, changed) =
            writer_collection.mutate(|database| match previous_project_family.as_deref() {
                Some(family) => database.set_default_ui_family(family),
                None => database.clear_default_ui_family(),
            });
        assert!(changed);
        assert!(reader.refresh_font_collection());
    }

    #[test]
    fn renderer_font_mutation_is_isolated_by_collection_service() {
        let first_collection =
            FontCollectionService::from_database(runtime_default_font_database_for_test());
        let second_collection =
            FontCollectionService::from_database(runtime_default_font_database_for_test());
        let first = TextRenderState::new_with_font_collection(0, Arc::clone(&first_collection));
        let mut second = TextRenderState::new_with_font_collection(0, second_collection);

        let (_, _, changed) = first_collection
            .mutate(|database| database.set_default_ui_family("Isolated Renderer Family"));
        assert!(changed);

        assert!(!second.refresh_font_collection());
        assert_ne!(
            second.font_database().default_ui_family_for_test(),
            Some("Isolated Renderer Family")
        );
    }

    #[test]
    fn published_revision_advances_before_render_state_adopts_the_database() {
        let font_collection =
            FontCollectionService::from_database(runtime_default_font_database_for_test());
        let mut state = TextRenderState::new_with_font_collection(0, Arc::clone(&font_collection));
        let adopted_before = state.font_collection_revision();

        let (_, _, changed) = font_collection.mutate(|database| {
            database.set_default_ui_family("Externally Published Renderer Family")
        });

        assert!(changed);
        assert_eq!(state.font_collection_revision(), adopted_before);
        assert_ne!(state.published_font_collection_revision(), adopted_before);
        assert!(state.refresh_font_collection());
        assert_eq!(
            state.font_collection_revision(),
            state.published_font_collection_revision()
        );
    }
}
