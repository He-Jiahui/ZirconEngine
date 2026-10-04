use std::sync::Arc;

use super::TextRenderState;
use crate::core::math::UVec2;
use crate::core::runtime::tasks::TaskPools;
use crate::text::atlas::{GlyphAtlasFormat, GlyphAtlasPageKey, GlyphAtlasPageSpec, GlyphAtlasSet};
use crate::text::font::{runtime_default_font_database_for_test, FontCollectionService};
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

    let (_, _, changed) = font_collection
        .mutate(|database| database.set_default_ui_family("Externally Published Renderer Family"));

    assert!(changed);
    assert_eq!(state.font_collection_revision(), adopted_before);
    assert_ne!(state.published_font_collection_revision(), adopted_before);
    assert!(state.refresh_font_collection());
    assert_eq!(
        state.font_collection_revision(),
        state.published_font_collection_revision()
    );
}
