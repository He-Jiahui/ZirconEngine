use std::sync::Arc;

use zircon_runtime_interface::ui::event_ui::UiNodeId;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiTextAlign, UiTextDirection, UiTextRange, UiTextWrap, UiTextWritingMode,
};

use super::ResolvedScreenSpaceUiTextBatches;
use crate::graphics::scene::scene_renderer::ui::render::{
    ScreenSpaceUiGlyphArtifactLine, ScreenSpaceUiTextBatch, ScreenSpaceUiTextRouteIdentity,
};
use crate::text::sdf::SdfMode;
use crate::text::ResolvedTextGlyphArtifact;

#[test]
fn stale_font_generation_artifact_batch_is_rejected_without_a_layout_session() {
    let current_collection = crate::text::font::FontCollectionService::from_database(
        crate::text::font::runtime_default_font_database_for_test(),
    );
    let current_revision = current_collection.revision();
    let stale_generation = current_revision.generation().wrapping_add(1);
    let mut resolved = ResolvedScreenSpaceUiTextBatches::from_explicit_batches(
        &[artifact_batch_with_font_lease(
            stale_generation,
            crate::text::ResolvedTextGlyphArtifactFontLease::capture(
                current_collection.collection_snapshot(),
            ),
        )],
        &[],
    );
    let constructions_before = crate::text::current_thread_text_layout_session_construction_count();

    resolved.reconcile_after_font_load(false, current_revision, &current_collection);

    let constructions_after = crate::text::current_thread_text_layout_session_construction_count();
    assert!(resolved.native_texts().is_empty());
    assert_eq!(
        resolved.post_layout_stale_artifact_batch_rejection_count(),
        1
    );
    assert_eq!(constructions_after, constructions_before);
}

#[test]
fn same_generation_artifact_from_foreign_collection_is_rejected() {
    let current_collection = crate::text::font::FontCollectionService::from_database(
        crate::text::font::runtime_default_font_database_for_test(),
    );
    let foreign_collection = crate::text::font::FontCollectionService::from_database(
        crate::text::font::runtime_default_font_database_for_test(),
    );
    assert_eq!(
        current_collection.generation(),
        foreign_collection.generation()
    );
    let generation = current_collection.generation();
    let mut resolved = ResolvedScreenSpaceUiTextBatches::from_explicit_batches(
        &[artifact_batch_with_font_lease(
            generation,
            crate::text::ResolvedTextGlyphArtifactFontLease::capture(
                foreign_collection.collection_snapshot(),
            ),
        )],
        &[],
    );

    resolved.reconcile_after_font_load(false, current_collection.revision(), &current_collection);

    assert!(resolved.native_texts().is_empty());
    assert_eq!(
        resolved.post_layout_stale_artifact_batch_rejection_count(),
        1
    );
}

#[test]
fn renderer_fallback_shapes_with_the_owned_font_collection() {
    let font_collection = crate::text::font::FontCollectionService::from_database(
        crate::text::font::runtime_default_font_database_for_test(),
    );
    assert_ne!(
        font_collection.collection_id(),
        crate::text::font::shared_font_collection_service().collection_id()
    );
    let mut batch = artifact_batch_with_font_lease(
        font_collection.generation(),
        crate::text::ResolvedTextGlyphArtifactFontLease::capture(
            font_collection.collection_snapshot(),
        ),
    );
    batch.glyph_artifact_line = None;
    batch.source_range = None;
    batch.glyph_advances.clear();
    batch.preserve_shaped_glyphs = false;
    let mut resolved = ResolvedScreenSpaceUiTextBatches::from_explicit_batches(&[batch], &[]);

    resolved.reconcile_after_font_load(false, font_collection.revision(), &font_collection);

    let shaped = &resolved.native_texts()[0].shaped_glyphs;
    assert!(!shaped.is_empty());
    assert!(shaped.iter().all(|glyph| {
        glyph
            .font_id
            .is_some_and(|font| font.collection == font_collection.collection_id())
    }));
}

fn artifact_batch_with_font_lease(
    font_generation: u64,
    font_lease: crate::text::ResolvedTextGlyphArtifactFontLease,
) -> ScreenSpaceUiTextBatch {
    let frame = UiFrame::new(0.0, 0.0, 24.0, 24.0);
    ScreenSpaceUiTextBatch {
        route_identity: ScreenSpaceUiTextRouteIdentity::new(
            "runtime.ui.text.stale-artifact-rejection.test",
            UiNodeId::new(1),
            None,
        ),
        command_generation: 1,
        raster_scale: 1.0,
        text: "stale".to_string(),
        frame,
        clip_frame: None,
        source_range: Some(UiTextRange { start: 0, end: 5 }),
        is_source_isomorphic_layout_line: false,
        glyph_advances: vec![24.0],
        shaped_glyphs: Vec::new(),
        preserve_shaped_glyphs: true,
        glyph_artifact_line: Some(ScreenSpaceUiGlyphArtifactLine {
            artifact: Arc::new(ResolvedTextGlyphArtifact {
                source_text: Arc::from("stale"),
                source_text_origin: 0,
                font_generation,
                font_lease,
                style: UiResolvedStyle::default(),
                writing_mode: UiTextWritingMode::HorizontalTb,
                lines: vec![None],
                logical_virtual_line_sequences: None,
            }),
            line_index: 0,
            font_generation,
            glyph_range: 0..0,
        }),
        layout_error: None,
        color: [1.0; 4],
        background_color: None,
        font: None,
        font_family: None,
        language: None,
        font_weight: UiResolvedStyle::DEFAULT_FONT_WEIGHT,
        font_size: 16.0,
        line_height: 20.0,
        text_align: UiTextAlign::Left,
        text_direction: UiTextDirection::LeftToRight,
        writing_mode: UiTextWritingMode::HorizontalTb,
        wrap: UiTextWrap::None,
        style: Default::default(),
        distance_field_mode: SdfMode::Sdf,
        text_effects: Default::default(),
        text_decorations: Default::default(),
        text_decoration_baseline: None,
        clip_transform: None,
    }
}
