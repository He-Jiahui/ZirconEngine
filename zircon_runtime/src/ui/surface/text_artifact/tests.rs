use std::sync::Arc;

use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine, UiResolvedTextRun, UiTextDirection,
    UiTextRange, UiTextRunKind, UiTextWritingMode,
};

use super::*;
use crate::core::framework::text::{TextGlyph, TextGlyphFlags, TextGlyphRotation};
use crate::text::{
    ResolvedTextGlyphArtifact, ResolvedTextGlyphArtifactLine, SharedTextLayoutSession,
    build_resolved_text_glyph_artifact, register_resolved_text_glyph_artifact,
};

#[test]
fn artifact_line_view_borrows_only_an_exact_current_layout_line() {
    let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
    let line = test_line();
    let glyphs = vec![test_glyph()];
    let artifact = Arc::new(ResolvedTextGlyphArtifact {
        source_text: Arc::from("Text"),
        source_text_origin: 0,
        font_generation: crate::text::font::shared_font_database_generation(),
        font_lease: crate::text::ResolvedTextGlyphArtifactFontLease::process_default(),
        style: UiResolvedStyle::default(),
        writing_mode: UiTextWritingMode::HorizontalTb,
        lines: vec![Some(ResolvedTextGlyphArtifactLine {
            glyphs,
            layout_line: line.clone(),
        })],
        logical_virtual_line_sequences: None,
    });
    let layout = UiResolvedTextLayout {
        font_size: 16.0,
        line_height: 20.0,
        measured_width: line.measured_width,
        measured_height: line.frame.height,
        source_range: line.source_range,
        lines: vec![line.clone()],
        rich_text_artifact: Some(register_resolved_text_glyph_artifact(Arc::clone(&artifact))),
        ..UiResolvedTextLayout::default()
    };

    let view = resolved_text_glyph_artifact_line(&layout, 0).expect("current matching artifact");
    let artifact_glyphs = artifact.lines[0]
        .as_ref()
        .expect("test artifact line")
        .glyphs
        .as_slice();
    assert_eq!(
        view.glyphs().expect("current artifact glyphs").as_ptr(),
        artifact_glyphs.as_ptr()
    );
    assert_eq!(view.glyphs(), Some(artifact_glyphs));
    assert_eq!(view.layout_line(), Some(&line));

    let mut mismatched_layout = layout.clone();
    mismatched_layout.lines[0].frame.width += 1.0;
    assert!(resolved_text_glyph_artifact_line(&mismatched_layout, 0).is_none());

    let mut synthetic_layout = layout;
    synthetic_layout.lines[0].ellipsized = true;
    assert!(resolved_text_glyph_artifact_line(&synthetic_layout, 0).is_none());
}

#[test]
fn artifact_line_view_rejects_a_stale_font_generation() {
    let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
    let line = test_line();
    let artifact = Arc::new(ResolvedTextGlyphArtifact {
        source_text: Arc::from("Text"),
        source_text_origin: 0,
        font_generation: crate::text::font::shared_font_database_generation().wrapping_add(1),
        font_lease: crate::text::ResolvedTextGlyphArtifactFontLease::process_default(),
        style: UiResolvedStyle::default(),
        writing_mode: UiTextWritingMode::HorizontalTb,
        lines: vec![Some(ResolvedTextGlyphArtifactLine {
            glyphs: vec![test_glyph()],
            layout_line: line.clone(),
        })],
        logical_virtual_line_sequences: None,
    });
    let layout = UiResolvedTextLayout {
        font_size: 16.0,
        line_height: 20.0,
        measured_width: line.measured_width,
        measured_height: line.frame.height,
        source_range: line.source_range,
        lines: vec![line],
        rich_text_artifact: Some(register_resolved_text_glyph_artifact(artifact)),
        ..UiResolvedTextLayout::default()
    };

    assert!(resolved_text_glyph_artifact_line(&layout, 0).is_none());
}

#[test]
fn artifact_line_view_retains_its_exact_font_generation_after_publication() {
    let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
    let view = built_runtime_artifact_view();
    let leased_generation = view.font_generation();
    let (_, database) = crate::text::font::shared_font_database_snapshot();
    let published_generation = crate::text::font::force_publish_shared_font_database(&database);

    assert!(published_generation > leased_generation);
    assert!(view.glyphs().is_some());
    assert!(view.layout_line().is_some());
    assert_eq!(view.font_generation(), leased_generation);
    assert!(view.raster_faces().is_some());
}

#[test]
fn artifact_line_view_snapshots_the_exact_runtime_face_for_each_raster_glyph() {
    let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
    let view = built_runtime_artifact_view();
    let glyph = view
        .glyphs()
        .expect("current glyphs")
        .iter()
        .find(|glyph| glyph.requires_rasterization)
        .expect("raster glyph");
    let snapshot = view.face_snapshot().expect("shared font snapshot");
    assert_eq!(snapshot.font_generation(), view.font_generation());
    let faces = view
        .raster_faces_from_snapshot(&snapshot)
        .expect("face snapshot");
    let face = faces.face_for(glyph).expect("glyph face");

    assert_eq!(face.font_face(), glyph.font_face.expect("font face handle"));
    assert_eq!(face.font_instance(), glyph.font_instance);
    assert_eq!(face.font_generation(), view.font_generation());
    assert!(!face.bytes().is_empty());
    assert!(Arc::ptr_eq(&face.bytes(), &face.bytes()));
}

#[test]
fn artifact_layout_faces_resolve_a_multiline_sequence_with_one_registry_snapshot() {
    let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
    let (first, second) = built_two_line_runtime_artifact_views();
    let first_glyph = first
        .glyphs()
        .expect("first current glyphs")
        .iter()
        .find(|glyph| glyph.requires_rasterization)
        .expect("first raster glyph");
    let second_glyph = second
        .glyphs()
        .expect("second current glyphs")
        .iter()
        .find(|glyph| glyph.requires_rasterization)
        .expect("second raster glyph");
    assert!(first.shares_artifact_layout_with(&second));

    let before = crate::text::font::font_handle_registry_report();
    let faces = first
        .artifact_raster_faces()
        .expect("whole artifact face table");
    let after = crate::text::font::font_handle_registry_report();

    assert_eq!(
        after.resolution_batch_count,
        before.resolution_batch_count + 1,
        "one resolved layout must use one font-handle batch"
    );
    assert_eq!(
        after.resolution_snapshot_acquire_count,
        before.resolution_snapshot_acquire_count + 1,
        "one resolved layout must acquire one registry snapshot"
    );
    assert_eq!(
        after.resolution_unique_pair_count,
        before.resolution_unique_pair_count + faces.faces().len() as u64,
        "repeated visual lines must share their exact shaped face pair"
    );
    assert!(faces.face_for(first_glyph).is_some());
    assert!(faces.face_for(second_glyph).is_some());
}

#[test]
fn artifact_layout_without_raster_glyphs_skips_font_database_and_handle_registry_resolution() {
    let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
    let view = built_face_free_runtime_artifact_view();

    let before = crate::text::font::font_handle_registry_report();
    let faces = view
        .artifact_raster_faces()
        .expect("empty raster face table");
    let after = crate::text::font::font_handle_registry_report();

    assert!(faces.faces().is_empty());
    assert_eq!(after.resolution_batch_count, before.resolution_batch_count);
    assert_eq!(
        after.resolution_snapshot_acquire_count,
        before.resolution_snapshot_acquire_count
    );
    assert_eq!(
        after.resolution_unique_pair_count,
        before.resolution_unique_pair_count
    );
}

#[cfg(feature = "profiling")]
#[test]
fn face_free_artifact_raster_faces_skip_the_font_database_snapshot_span() {
    let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
    let view = built_face_free_runtime_artifact_view();
    let expected_scanned_glyph_count = view.glyphs().expect("current artifact glyphs").len() as f64;
    let _capture_guard = crate::core::diagnostics::profiling::test_capture_lock();
    let mut config = crate::core::diagnostics::profiling::ProfileCaptureConfig::default();
    config.session_id = "surface-text-artifact-face-free".to_owned();
    config.max_spans = 2;
    config.max_counters = 3;
    assert!(crate::core::diagnostics::profiling::start_capture(config).active);

    let faces = view
        .artifact_raster_faces()
        .expect("face-free artifact must resolve without a database snapshot");

    let snapshot = crate::core::diagnostics::profiling::snapshot();
    assert!(
        !crate::core::diagnostics::profiling::reset_capture().active,
        "artifact face profiling capture must reset before another test starts"
    );
    assert!(faces.faces().is_empty());
    assert_eq!(
        snapshot
            .spans
            .iter()
            .filter(|span| {
                span.category == "text.surface" && span.name == "artifact_face_snapshot"
            })
            .count(),
        0,
        "face-free artifact resolution must not clone the font database"
    );
    for (name, value) in [
        (
            "artifact_raster_face_scanned_glyph_count",
            expected_scanned_glyph_count,
        ),
        ("artifact_raster_face_candidate_glyph_count", 0.0),
        ("artifact_raster_face_unique_pair_count", 0.0),
    ] {
        assert_eq!(
            snapshot
                .counters
                .iter()
                .find(|counter| counter.stream == "runtime" && counter.name == name)
                .map(|counter| counter.value),
            Some(value),
            "face-free artifact resolution must report {name}"
        );
    }
}

#[cfg(feature = "profiling")]
#[test]
fn artifact_raster_face_snapshot_emits_one_surface_profile_span() {
    let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
    let view = built_runtime_artifact_view();
    let _capture_guard = crate::core::diagnostics::profiling::test_capture_lock();
    let mut config = crate::core::diagnostics::profiling::ProfileCaptureConfig::default();
    config.session_id = "surface-text-artifact-face-snapshot".to_owned();
    config.max_spans = 2;
    config.max_counters = 3;
    assert!(crate::core::diagnostics::profiling::start_capture(config).active);

    let artifact_glyphs = view.glyphs().expect("current artifact glyphs");
    let expected_scanned_glyph_count = artifact_glyphs.len() as f64;
    let expected_raster_glyph_count = artifact_glyphs
        .iter()
        .filter(|glyph| glyph.requires_rasterization)
        .count() as f64;
    let faces = view.raster_faces().expect("face snapshot");

    let snapshot = crate::core::diagnostics::profiling::snapshot();
    assert!(
        !crate::core::diagnostics::profiling::reset_capture().active,
        "artifact face profiling capture must reset before another test starts"
    );
    assert_eq!(
        snapshot
            .spans
            .iter()
            .filter(|span| {
                span.category == "text.surface" && span.name == "artifact_face_snapshot"
            })
            .count(),
        1
    );
    assert_eq!(
        snapshot
            .spans
            .iter()
            .filter(|span| {
                span.category == "text.surface" && span.name == "artifact_raster_face_resolution"
            })
            .count(),
        1
    );
    for (name, value) in [
        (
            "artifact_raster_face_scanned_glyph_count",
            expected_scanned_glyph_count,
        ),
        (
            "artifact_raster_face_candidate_glyph_count",
            expected_raster_glyph_count,
        ),
        (
            "artifact_raster_face_unique_pair_count",
            faces.faces().len() as f64,
        ),
    ] {
        assert_eq!(
            snapshot
                .counters
                .iter()
                .find(|counter| counter.stream == "runtime" && counter.name == name)
                .map(|counter| counter.value),
            Some(value),
            "artifact face resolution must report {name}"
        );
    }
}

fn built_runtime_artifact_view() -> UiResolvedTextGlyphArtifactLine {
    let line = test_line();
    let style = UiResolvedStyle {
        font_size: 16.0,
        line_height: 20.0,
        ..UiResolvedStyle::default()
    };
    let mut layout = UiResolvedTextLayout {
        font_size: style.font_size,
        line_height: style.line_height,
        measured_width: line.measured_width,
        measured_height: line.frame.height,
        source_range: line.source_range,
        lines: vec![line],
        ..UiResolvedTextLayout::default()
    };
    let mut provider = SharedTextLayoutSession::new();
    let artifact = Arc::new(
        build_resolved_text_glyph_artifact("Text", &style, &layout, &mut provider)
            .into_result()
            .expect("canonical text artifact shaping")
            .expect("canonical text artifact"),
    );
    layout.rich_text_artifact = Some(register_resolved_text_glyph_artifact(artifact));

    resolved_text_glyph_artifact_line(&layout, 0).expect("current artifact line")
}

fn built_face_free_runtime_artifact_view() -> UiResolvedTextGlyphArtifactLine {
    let source = built_runtime_artifact_view();
    let mut artifact = (*source.artifact).clone();
    for line in artifact.lines.iter_mut().flatten() {
        for glyph in &mut line.glyphs {
            glyph.requires_rasterization = false;
        }
    }
    UiResolvedTextGlyphArtifactLine {
        artifact: Arc::new(artifact),
        line_index: 0,
        font_collection: source.font_collection,
        font_handles: source.font_handles,
    }
}

fn built_two_line_runtime_artifact_views() -> (
    UiResolvedTextGlyphArtifactLine,
    UiResolvedTextGlyphArtifactLine,
) {
    let source = built_runtime_artifact_view();
    let duplicated_line = source.artifact.lines[0]
        .as_ref()
        .expect("source artifact line")
        .clone();
    let mut artifact = (*source.artifact).clone();
    artifact.lines.push(Some(duplicated_line.clone()));
    let artifact = Arc::new(artifact);
    let layout = UiResolvedTextLayout {
        font_size: artifact.style.font_size,
        line_height: artifact.style.line_height,
        measured_width: duplicated_line.layout_line.measured_width,
        measured_height: duplicated_line.layout_line.frame.height * 2.0,
        source_range: duplicated_line.layout_line.source_range,
        lines: vec![
            duplicated_line.layout_line.clone(),
            duplicated_line.layout_line,
        ],
        rich_text_artifact: Some(register_resolved_text_glyph_artifact(artifact)),
        ..UiResolvedTextLayout::default()
    };

    (
        resolved_text_glyph_artifact_line(&layout, 0).expect("first current artifact line"),
        resolved_text_glyph_artifact_line(&layout, 1).expect("second current artifact line"),
    )
}

#[test]
fn artifact_line_view_rejects_a_raster_glyph_without_a_runtime_font_face() {
    let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
    let line = test_line();
    let font_collection = crate::text::font::shared_font_collection_snapshot();
    let font_handles = crate::text::font::font_handle_resolver_snapshot(&font_collection);
    let artifact = Arc::new(ResolvedTextGlyphArtifact {
        source_text: Arc::from("Text"),
        source_text_origin: 0,
        font_generation: font_collection.generation(),
        font_lease: crate::text::ResolvedTextGlyphArtifactFontLease::process_default(),
        style: UiResolvedStyle::default(),
        writing_mode: UiTextWritingMode::HorizontalTb,
        lines: vec![Some(ResolvedTextGlyphArtifactLine {
            glyphs: vec![test_glyph()],
            layout_line: line,
        })],
        logical_virtual_line_sequences: None,
    });
    let view = UiResolvedTextGlyphArtifactLine {
        artifact,
        line_index: 0,
        font_collection,
        font_handles,
    };

    assert!(view.raster_faces().is_none());
}

fn test_line() -> UiResolvedTextLine {
    UiResolvedTextLine {
        text: "Text".to_owned(),
        placement_frame: UiFrame::default(),
        frame: UiFrame::new(4.0, 8.0, 32.0, 20.0),
        source_range: UiTextRange { start: 0, end: 4 },
        visual_range: UiTextRange { start: 0, end: 4 },
        measured_width: 32.0,
        glyph_advances: vec![8.0; 4],
        baseline: 16.0,
        direction: UiTextDirection::LeftToRight,
        runs: vec![UiResolvedTextRun {
            kind: UiTextRunKind::Plain,
            text: "Text".to_owned(),
            source_range: UiTextRange { start: 0, end: 4 },
            visual_range: UiTextRange { start: 0, end: 4 },
            direction: UiTextDirection::LeftToRight,
        }],
        ellipsized: false,
    }
}

fn test_glyph() -> TextGlyph {
    TextGlyph {
        glyph_id: 1,
        source_range: 0..4,
        visual_range: 0..4,
        advance: 32.0,
        position: [0.0, 0.0],
        offset: [0.0, 0.0],
        font_face: None,
        font_instance: None,
        rotation: TextGlyphRotation::None,
        bidi_level: 0,
        flags: TextGlyphFlags::default(),
        requires_rasterization: true,
    }
}
