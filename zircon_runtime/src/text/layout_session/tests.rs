use super::*;
use crate::text::FontFaceId;
use crate::text::font::{
    font_handle_registry_report, runtime_default_font_database_for_test,
    shared_font_database_test_serial_guard,
};
use crate::text::shaping::{
    TextShapingFailureCode, TextShapingFailureDependency, TextShapingFailureDisposition,
    TextShapingFailurePhase, TextShapingFailureReceipt,
};
use crate::text::{TextLayoutGeometryBudget, TextLayoutGeometryOwner};

#[test]
fn session_snapshots_geometry_budget_and_resets_rejection_diagnostics_per_frame() {
    let font_collection =
        FontCollectionService::from_database(runtime_default_font_database_for_test());
    let budget = TextLayoutGeometryBudget::new(64.0, 128.0).expect("valid budget");
    let mut session = SharedTextLayoutSession::new_with_font_collection_and_geometry_budget(
        font_collection,
        budget,
    );
    let violation = budget
        .checked_add_accumulated(96.0, 33.0)
        .expect_err("sum exceeds the session budget");

    assert_eq!(session.geometry_budget(), budget);
    assert_eq!(
        session.reject_geometry(
            TextLayoutGeometryOwner::TableColumnTracks,
            violation,
            Some((7, 19)),
            4,
        ),
        TextLayoutError::GeometryTooLarge
    );
    let receipt = session
        .geometry_report()
        .last_rejection
        .expect("rejection receipt");
    assert_eq!(receipt.owner, TextLayoutGeometryOwner::TableColumnTracks);
    assert_eq!(receipt.source_range, Some((7, 19)));
    assert_eq!(receipt.attempted_extent, 129.0);
    assert_eq!(receipt.admitted_extent, 128.0);
    assert_eq!(receipt.work_units, 4);

    session.begin_frame(1);
    assert_eq!(session.geometry_report(), Default::default());
}

#[test]
fn rich_parser_cache_isolated_by_surface_session_owner() {
    let mut first = SharedTextLayoutSession::new();
    let second = SharedTextLayoutSession::new();
    let markup = "[b]session owner[/b]";

    let first_artifact = first
        .compile_rich_text(markup, RichTextFormat::BbCodeV1)
        .expect("first session compiles");
    let repeated = first
        .compile_rich_text(markup, RichTextFormat::BbCodeV1)
        .expect("first session cache hit");
    let second_artifact = second
        .compile_rich_text(markup, RichTextFormat::BbCodeV1)
        .expect("second session compiles independently");

    assert!(Arc::ptr_eq(&first_artifact, &repeated));
    assert!(!Arc::ptr_eq(&first_artifact, &second_artifact));
    assert!(
        first
            .lookup_compiled_rich_text(markup, RichTextFormat::BbCodeV1)
            .is_some()
    );
    assert!(
        second
            .lookup_compiled_rich_text(markup, RichTextFormat::BbCodeV1)
            .is_some()
    );

    first.clear();
    assert!(
        first
            .lookup_compiled_rich_text(markup, RichTextFormat::BbCodeV1)
            .is_none()
    );
    assert!(
        second
            .lookup_compiled_rich_text(markup, RichTextFormat::BbCodeV1)
            .is_some()
    );
}

#[test]
fn unkeyed_hard_line_windows_do_not_retain_a_full_line_index() {
    let mut session = SharedTextLayoutSession::new();

    let (line_count, window) =
        session.unretained_hard_line_count_and_window("zero\none\ntwo", 1..2);

    assert_eq!(line_count, 3);
    assert_eq!(window[0].content, 5..8);
    assert_eq!(session.hard_line_index_report().entry_count, 0);
    assert_eq!(session.hard_line_index_report().unkeyed_bypass_count, 1);
}

#[test]
fn session_cache_and_prewarm_follow_the_owned_font_collection_generation() {
    let first_collection =
        FontCollectionService::from_database(runtime_default_font_database_for_test());
    let second_collection =
        FontCollectionService::from_database(runtime_default_font_database_for_test());
    let initial_generation = first_collection.generation();
    let (published_generation, _, _) = first_collection
        .mutate(|database| database.set_default_ui_family("Session Scoped Font Family"));
    assert!(published_generation > initial_generation);
    assert_ne!(published_generation, second_collection.generation());

    let mut first_session =
        SharedTextLayoutSession::new_with_font_collection(Arc::clone(&first_collection));
    let second_session =
        SharedTextLayoutSession::new_with_font_collection(Arc::clone(&second_collection));
    assert!(Arc::ptr_eq(
        &first_session.font_collection,
        &first_collection
    ));
    assert!(Arc::ptr_eq(
        &second_session.font_collection,
        &second_collection
    ));

    let pool = TaskPool::new(crate::core::runtime::tasks::TaskPoolDescriptor::compute());
    let style = TextStyle::default();
    let paragraph = TextShapeParagraph::horizontal(
        "owned collection",
        style.clone(),
        TextDirection::LeftToRight,
        TextRange { start: 0, end: 16 },
    );
    let report = first_session.prewarm_horizontal_paragraphs(&pool, &[paragraph], 1);
    assert_eq!(report.inserted_count, 1);

    let canonical = BackendShapeRequest::horizontal(
        "owned collection",
        &style,
        TextDirection::LeftToRight,
        TextRange { start: 0, end: 16 },
    )
    .canonicalized()
    .expect("canonical request");
    let request = canonical.request();
    let owned_lookup = ShapedRunCacheLookupKey::from_request_in_font_collection(
        &request,
        first_collection.collection_id(),
        published_generation,
    );
    let foreign_lookup = ShapedRunCacheLookupKey::from_request_in_font_collection(
        &request,
        second_collection.collection_id(),
        second_collection.generation(),
    );

    assert!(
        first_session
            .shaped_runs
            .get_with_lookup(&owned_lookup, request.text)
            .is_some()
    );
    assert!(
        first_session
            .shaped_runs
            .get_with_lookup(&foreign_lookup, request.text)
            .is_none()
    );
}

#[test]
fn session_routes_detailed_runs_through_canonical_service() {
    let mut session = SharedTextLayoutSession::new();
    let style = TextStyle::default();
    let run = session
        .shape_horizontal_range_with_kerning(
            "Canonical",
            &style,
            TextDirection::LeftToRight,
            TextRange { start: 11, end: 20 },
            true,
        )
        .into_result()
        .expect("shape canonical run");

    assert_eq!(run.source_range.start, 11);
    assert!(run.measured_width > 0.0);
    assert!(run.lines.iter().any(|line| !line.glyphs.is_empty()));
    assert!(
        run.lines
            .iter()
            .flat_map(|line| &line.glyphs)
            .all(|glyph| { glyph.source_range.start >= 11 && glyph.source_range.end <= 20 })
    );
}

#[test]
fn font_handle_batch_session_keeps_canonical_run_without_framework_roundtrip() {
    let _shared_font_database = shared_font_database_test_serial_guard();
    let before = font_handle_registry_report();
    let mut session = SharedTextLayoutSession::new();
    let style = TextStyle::default();

    let run = session
        .shape_horizontal_range_with_kerning(
            "Batch resolution",
            &style,
            TextDirection::LeftToRight,
            TextRange { start: 0, end: 16 },
            true,
        )
        .into_result()
        .expect("shape font-handle run");
    let after = font_handle_registry_report();
    let glyph_count = run
        .lines
        .iter()
        .map(|line| line.glyphs.len())
        .sum::<usize>();

    assert!(glyph_count > 1);
    assert_eq!(
        after.registration_batch_count, before.registration_batch_count,
        "the internal session must not project backend identities into framework handles"
    );
    assert_eq!(
        after.resolution_batch_count, before.resolution_batch_count,
        "the internal session must not resolve framework handles back into backend identities"
    );
}

#[test]
fn session_preserves_typed_failure_without_materializing_an_empty_run() {
    let mut session = SharedTextLayoutSession::new();
    let style = TextStyle {
        font_size: 0.0,
        ..TextStyle::default()
    };

    let outcome = session.shape_horizontal_range(
        "invalid",
        &style,
        TextDirection::LeftToRight,
        TextRange { start: 0, end: 7 },
    );
    assert!(matches!(
        outcome,
        TextShapingOutcome::Failed(failure)
            if failure.error() == &TextLayoutError::InvalidFontSize
    ));
    assert!(session.shaped_runs.is_empty());
}

#[test]
fn invalid_font_size_failure_cannot_alias_a_valid_one_pixel_shape() {
    let mut session = SharedTextLayoutSession::new();
    let invalid_style = TextStyle {
        font_size: 0.0,
        ..TextStyle::default()
    };
    let valid_style = TextStyle {
        font_size: 1.0,
        ..TextStyle::default()
    };
    let range = TextRange { start: 0, end: 5 };

    let invalid =
        session.shape_horizontal_range("alias", &invalid_style, TextDirection::LeftToRight, range);
    let valid =
        session.shape_horizontal_range("alias", &valid_style, TextDirection::LeftToRight, range);

    assert!(matches!(
        invalid,
        TextShapingOutcome::Failed(failure)
            if failure.error() == &TextLayoutError::InvalidFontSize
    ));
    assert!(matches!(
        valid,
        TextShapingOutcome::Ready(run) if run.lines.iter().any(|line| !line.glyphs.is_empty())
    ));
}

#[test]
fn failed_shaping_outcome_never_enters_the_session_cache() {
    let mut session = SharedTextLayoutSession::new();
    let style = TextStyle::default();
    let request = BackendShapeRequest::horizontal(
        "retryable source",
        &style,
        TextDirection::LeftToRight,
        TextRange { start: 0, end: 16 },
    );
    let lookup = ShapedRunCacheLookupKey::from_request(&request);
    let outcome = session.consume_shaping_outcome(
        &lookup,
        lookup.font_database_generation(),
        TextShapingOutcome::failed(TextLayoutError::BidiInvariant),
    );

    assert!(matches!(
        outcome,
        TextShapingOutcome::Failed(failure)
            if failure.error() == &TextLayoutError::BidiInvariant
    ));
    assert!(session.shaped_runs.is_empty());
    assert_eq!(session.shaped_runs.report().insert_count, 0);
}

#[test]
fn session_preserves_request_owned_shaping_failure_receipt() {
    let mut session = SharedTextLayoutSession::new();
    let style = TextStyle::default();
    let request = BackendShapeRequest::horizontal(
        "receipt source",
        &style,
        TextDirection::LeftToRight,
        TextRange { start: 0, end: 14 },
    );
    let lookup = ShapedRunCacheLookupKey::from_request(&request);
    let receipt = TextShapingFailureReceipt {
        code: TextShapingFailureCode::BackendFaceParse,
        phase: TextShapingFailurePhase::FontLoad,
        source_range: Some(TextRange { start: 2, end: 8 }),
        face: Some(FontFaceId(9)),
        dependency: TextShapingFailureDependency::FontFace,
        disposition: TextShapingFailureDisposition::AlternateBackend,
        budget: None,
    };

    let outcome = session.consume_shaping_outcome(
        &lookup,
        lookup.font_database_generation(),
        TextShapingOutcome::failed_with_receipt(TextLayoutError::ShapingFailed, receipt),
    );

    assert_eq!(outcome.failure_receipt(), Some(receipt));
    assert!(session.shaped_runs.is_empty());
    assert_eq!(session.shaped_runs.report().insert_count, 0);
}

#[test]
fn artifact_query_preserves_invalid_font_failure_without_materializing_an_empty_run() {
    let mut session = SharedTextLayoutSession::new();
    let style = TextStyle {
        font_size: 0.0,
        ..TextStyle::default()
    };

    let outcome = session.shape_horizontal_range(
        "invalid",
        &style,
        TextDirection::LeftToRight,
        TextRange { start: 0, end: 7 },
    );

    assert!(matches!(
        outcome,
        TextShapingOutcome::Failed(failure)
            if failure.error() == &TextLayoutError::InvalidFontSize
    ));
    assert!(session.shaped_runs.is_empty());
}

#[test]
fn prewarm_reports_invalid_requests_without_caching_or_materializing_a_fallback() {
    let mut session = SharedTextLayoutSession::new();
    let pool = TaskPool::new(crate::core::runtime::tasks::TaskPoolDescriptor::compute());
    let paragraph = TextShapeParagraph::horizontal(
        "invalid prewarm",
        TextStyle {
            font_size: 0.0,
            ..TextStyle::default()
        },
        TextDirection::LeftToRight,
        TextRange { start: 0, end: 15 },
    );

    let report = session.prewarm_horizontal_paragraphs(&pool, &[paragraph], 1);
    let outcome = session.shape_horizontal_range(
        "invalid prewarm",
        &TextStyle {
            font_size: 0.0,
            ..TextStyle::default()
        },
        TextDirection::LeftToRight,
        TextRange { start: 0, end: 15 },
    );
    assert_eq!(report.shaped_count, 0);
    assert_eq!(report.invalid_request_count, 1);
    assert_eq!(report.inserted_count, 0);
    assert!(matches!(
        outcome,
        TextShapingOutcome::Failed(failure)
            if failure.error() == &TextLayoutError::InvalidFontSize
    ));
    assert!(session.shaped_runs.is_empty());
}

#[test]
fn prewarm_merges_backend_route_receipts_into_the_session_owner() {
    let mut session = SharedTextLayoutSession::new();
    let pool = TaskPool::new(crate::core::runtime::tasks::TaskPoolDescriptor::compute());
    let paragraph = TextShapeParagraph::horizontal(
        "session prewarm receipt",
        TextStyle::default(),
        TextDirection::LeftToRight,
        TextRange { start: 0, end: 23 },
    );

    let report = session.prewarm_horizontal_paragraphs(&pool, &[paragraph], 1);

    assert_eq!(
        report.shaping_diagnostics.backend_routes.direct_run_count,
        1
    );
    assert_eq!(
        session
            .diagnostics_report()
            .shaping
            .backend_routes
            .direct_run_count,
        1
    );
}

#[test]
fn cache_miss_merges_transient_font_resolution_receipt_into_session_owner() {
    let mut session = SharedTextLayoutSession::new();
    let style = TextStyle::default();

    let outcome = session.shape_horizontal_range(
        "font resolution receipt",
        &style,
        TextDirection::LeftToRight,
        TextRange { start: 0, end: 23 },
    );

    assert!(matches!(outcome, TextShapingOutcome::Ready(_)));
    let font_resolution = session.diagnostics_report().shaping.font_resolution;
    assert_eq!(font_resolution.primary_text_request_count, 1);
    assert_eq!(font_resolution.primary_text_fast_path_count, 1);
    assert_eq!(font_resolution.resolution_request_count, 0);
}

#[test]
fn session_source_uses_canonical_runs_without_framework_roundtrip() {
    let source = include_str!("../layout_session.rs");

    assert!(!source.contains(concat!("TextShape", "Result")));
    assert!(!source.contains(concat!("resolve_font_handle", "_batch")));
    assert!(!source.contains(concat!("project_shape", "_result")));
    assert!(source.contains("shape_backend_request_at_stable_generation"));
}

#[test]
fn diagnostics_are_owner_local_and_reset_at_the_frame_boundary() {
    let mut first = SharedTextLayoutSession::new();
    let second = SharedTextLayoutSession::new();
    let baseline = second.diagnostics_report();

    first.record_layout_error(&TextLayoutError::InvalidLanguage);

    assert_ne!(first.diagnostics_report(), baseline);
    assert_eq!(second.diagnostics_report(), baseline);

    first.begin_frame(17);

    assert_eq!(first.diagnostics_report(), baseline);
}

#[test]
fn table_layout_work_report_is_owner_local_and_resets_at_the_frame_boundary() {
    let mut first = SharedTextLayoutSession::new();
    let second = SharedTextLayoutSession::new();

    first.record_table_layout_attempt(120, 3);
    first.record_table_layout_tracks(2, 2);
    first.record_table_preferred_cell_layout(10);
    first.record_table_final_cell_layout(10);
    first.record_table_layout_output(4, 3);

    assert_ne!(
        first.table_layout_work_report(),
        second.table_layout_work_report()
    );
    assert_eq!(
        first.table_layout_work_report().table_layout_attempt_count,
        1
    );
    assert_eq!(first.table_layout_work_report().published_line_count, 4);

    first.begin_frame(18);

    assert_eq!(
        first.table_layout_work_report(),
        second.table_layout_work_report()
    );
}

#[test]
fn session_diagnostics_classify_whole_run_alternate_backend_recovery() {
    let mut session = SharedTextLayoutSession::new();
    let style = TextStyle::default();
    let request = BackendShapeRequest::horizontal(
        "alternate route",
        &style,
        TextDirection::LeftToRight,
        TextRange { start: 0, end: 15 },
    );
    let lookup = ShapedRunCacheLookupKey::from_request(&request);
    let mut run = Arc::unwrap_or_clone(
        shape_request_outcome(request)
            .into_result()
            .expect("test input must shape at a stable generation"),
    );
    let receipt = TextShapingFailureReceipt {
        code: TextShapingFailureCode::BackendFaceParse,
        phase: TextShapingFailurePhase::FontLoad,
        source_range: Some(TextRange { start: 2, end: 8 }),
        face: Some(FontFaceId(9)),
        dependency: TextShapingFailureDependency::FontFace,
        disposition: TextShapingFailureDisposition::AlternateBackend,
        budget: None,
    };
    run.horizontal_composition_receipt =
        Some(Box::new(crate::text::TextHorizontalCompositionReceipt {
            alternate_ranges: Vec::new(),
            first_failure: receipt,
        }));

    let outcome = session.consume_shaping_outcome(
        &lookup,
        lookup.font_database_generation(),
        TextShapingOutcome::Ready(GenerationTaggedShapedRun {
            run: Arc::new(run),
            font_generation: lookup.font_database_generation(),
            request_diagnostics: Default::default(),
        }),
    );

    assert!(matches!(outcome, TextShapingOutcome::Ready(_)));
    let diagnostics = session.diagnostics_report();
    assert_eq!(diagnostics.shaping.backend_routes.alternate_run_count, 1);
    assert_eq!(diagnostics.shaping.backend_routes.direct_run_count, 0);
    assert_eq!(diagnostics.shaping.backend_routes.hybrid_run_count, 0);
    assert_eq!(diagnostics.shaping.failures.observed_count, 1);
    assert_eq!(
        diagnostics
            .shaping
            .failures
            .count(TextShapingFailureCode::BackendFaceParse),
        1
    );
    assert_eq!(diagnostics.shaping.failures.last_failure, Some(receipt));
}
