use super::*;

fn test_font_revision(generation: u64) -> FontCollectionRevision {
    FontCollectionRevision::new(
        crate::text::font::shared_font_collection_handle(),
        generation,
    )
}

#[test]
fn screen_space_ui_text_segment_plan_reuses_exact_text_inputs_across_plan_identity() {
    let plan = Arc::new(PlannedScreenSpaceUi::default());
    let same = Arc::clone(&plan);
    let replacement = Arc::new(PlannedScreenSpaceUi::default());
    let current = Arc::clone(&plan);

    assert!(segment_plan_reused(Some(&current), &same));
    assert!(segment_plan_reused(Some(&current), &replacement));
    assert!(!segment_plan_reused(None, &same));
}

#[test]
fn text_segment_product_reuse_requires_text_inputs_viewport_and_font_revision_identity() {
    let plan = Arc::new(PlannedScreenSpaceUi::default());
    let revision = test_font_revision(7);
    let entry = ScreenSpaceUiTextSegmentProductEntry {
        plan: Arc::clone(&plan),
        viewport_size: UVec2::new(800, 600),
        font_revision: revision,
        product: Arc::new(ScreenSpaceUiTextSegmentProduct {
            resolved_texts: Default::default(),
            resolved_report: Default::default(),
            native_glyph_runs: Default::default(),
            native_glyph_dependencies: Default::default(),
            native_glyph_dependency_keys: Arc::from([]),
            input_batch_counts: [0; 3],
            auto_routes: Arc::from([]),
        }),
    };

    assert!(segment_product_entry_reused(
        &entry,
        &plan,
        UVec2::new(800, 600),
        revision,
    ));
    assert!(!segment_product_entry_reused(
        &entry,
        &plan,
        UVec2::new(801, 600),
        revision,
    ));
    assert!(!segment_product_entry_reused(
        &entry,
        &plan,
        UVec2::new(800, 600),
        test_font_revision(8),
    ));
    assert!(segment_product_entry_reused(
        &entry,
        &Arc::new(PlannedScreenSpaceUi::default()),
        UVec2::new(800, 600),
        revision,
    ));
}

#[test]
fn text_segment_product_reuse_rejects_foreign_collection_at_same_generation() {
    let plan = Arc::new(PlannedScreenSpaceUi::default());
    let first_collection = crate::text::font::FontCollectionService::from_database(
        crate::text::font::runtime_default_font_database_for_test(),
    );
    let foreign_collection = crate::text::font::FontCollectionService::from_database(
        crate::text::font::runtime_default_font_database_for_test(),
    );
    assert_eq!(
        first_collection.generation(),
        foreign_collection.generation()
    );
    let revision = first_collection.revision();
    let entry = ScreenSpaceUiTextSegmentProductEntry {
        plan: Arc::clone(&plan),
        viewport_size: UVec2::new(800, 600),
        font_revision: revision,
        product: Arc::new(ScreenSpaceUiTextSegmentProduct {
            resolved_texts: Default::default(),
            resolved_report: Default::default(),
            native_glyph_runs: Default::default(),
            native_glyph_dependencies: Default::default(),
            native_glyph_dependency_keys: Arc::from([]),
            input_batch_counts: [0; 3],
            auto_routes: Arc::from([]),
        }),
    };

    assert!(!segment_product_entry_reused(
        &entry,
        &plan,
        UVec2::new(800, 600),
        foreign_collection.revision(),
    ));
}

#[test]
fn text_frame_product_generation_advances_only_when_a_product_is_published() {
    let mut counter = 0;
    let first = ScreenSpaceUiTextFrameProductGeneration::next(&mut counter);
    let second = ScreenSpaceUiTextFrameProductGeneration::next(&mut counter);

    assert_ne!(first, second);
    assert_eq!(counter, 2);
}

#[test]
fn text_stable_source_delta_reuses_frame_product_arc_and_generation() {
    let mut generation_counter = 0;
    let generation = ScreenSpaceUiTextFrameProductGeneration::next(&mut generation_counter);
    let product = Arc::new(ScreenSpaceUiTextFrameProduct {
        generation,
        change_journal: ScreenSpaceUiTextFrameChangeJournal::for_test_initial(generation),
        segment_products: Arc::from([]),
        resolved_report: Default::default(),
        native_font_ids: Default::default(),
        input_batch_counts: [0; 3],
        active_native_glyph_dependency_count: 0,
        native_reverse_instance_entry_count: 0,
        native_reverse_segment_entry_count: 0,
        native_run_count: 0,
        sdf_run_count: 0,
    });
    let mut cache = ScreenSpaceUiTextSegmentCache {
        frame_generation: Some(7),
        frame_product: Some(Arc::clone(&product)),
        ..Default::default()
    };

    let reused = cache.reuse_frame_product_for_source_generation(8);

    assert!(Arc::ptr_eq(&product, &reused));
    assert_eq!(reused.generation(), generation);
    assert_eq!(cache.frame_generation, Some(8));
}

#[test]
fn text_frame_aggregate_index_patches_one_leaf_without_recomposing_all_segments() {
    let plans = (0..3)
        .map(|_| Arc::new(PlannedScreenSpaceUi::default()))
        .collect::<Vec<_>>();
    let mut entries = plans
        .iter()
        .enumerate()
        .map(|(index, plan)| ScreenSpaceUiTextSegmentProductEntry {
            plan: Arc::clone(plan),
            viewport_size: UVec2::new(800, 600),
            font_revision: test_font_revision(7),
            product: Arc::new(empty_segment_product([index + 1, 0, 0])),
        })
        .collect::<Vec<_>>();
    let mut index = ScreenSpaceUiTextFrameAggregateIndex::default();
    index.rebuild(&entries);

    assert_eq!(index.root().input_batch_counts, [6, 0, 0]);
    let replacement = Arc::new(empty_segment_product([10, 2, 1]));
    entries[1].product = Arc::clone(&replacement);
    assert!(index.patch(1, &replacement));

    assert_eq!(index.root().input_batch_counts, [14, 2, 1]);
    assert_eq!(index.leaf_count(), 3);
    assert!(!index.patch(3, &replacement));
}

fn empty_segment_product(input_batch_counts: [usize; 3]) -> ScreenSpaceUiTextSegmentProduct {
    ScreenSpaceUiTextSegmentProduct {
        resolved_texts: Default::default(),
        resolved_report: Default::default(),
        native_glyph_runs: Default::default(),
        native_glyph_dependencies: Default::default(),
        native_glyph_dependency_keys: Arc::from([]),
        input_batch_counts,
        auto_routes: Arc::from([]),
    }
}
