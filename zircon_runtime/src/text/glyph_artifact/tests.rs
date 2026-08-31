use super::*;
use crate::core::framework::text::{TextDirection, TextGlyphFlags, TextGlyphRotation};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiResolvedTextLayout, UiResolvedTextRun, UiTextCaret, UiTextCaretAffinity, UiTextDirection,
    UiTextRunKind, UiTextWritingMode,
};

mod cluster_geometry;
mod invariant_failures;
mod virtual_replacement;
mod visual_projection_contract;

#[test]
fn glyph_artifact_handle_identity_tracks_render_and_rebuild_state() {
    let artifact = Arc::new(ResolvedTextGlyphArtifact {
        source_text: Arc::from("identity"),
        source_text_origin: 0,
        font_generation: 7,
        font_lease: ResolvedTextGlyphArtifactFontLease::process_default(),
        style: UiResolvedStyle::default(),
        writing_mode: UiTextWritingMode::HorizontalTb,
        lines: Vec::new(),
        logical_virtual_line_sequences: None,
    });
    let changed_generation = Arc::new(ResolvedTextGlyphArtifact {
        font_generation: 8,
        ..artifact.as_ref().clone()
    });
    let changed_source = Arc::new(ResolvedTextGlyphArtifact {
        source_text: Arc::from("different"),
        ..artifact.as_ref().clone()
    });
    let first = register_resolved_text_glyph_artifact(Arc::clone(&artifact));
    let same = register_resolved_text_glyph_artifact(artifact);
    let changed_generation = register_resolved_text_glyph_artifact(changed_generation);
    let changed_source = register_resolved_text_glyph_artifact(changed_source);

    assert_eq!(first, same);
    assert_ne!(first, changed_generation);
    assert_ne!(first, changed_source);
}

#[test]
fn glyph_artifact_source_origin_accepts_only_full_source_or_exact_layout_slice() {
    let layout_source_range = UiTextRange { start: 4, end: 8 };

    assert_eq!(
        source_text_origin("0123456789", layout_source_range),
        Some(0)
    );
    assert_eq!(
        source_text_origin("4567", layout_source_range),
        Some(layout_source_range.start)
    );
    assert_eq!(source_text_origin("abcde", layout_source_range), None);
}

#[test]
fn glyph_artifact_exact_source_slice_preserves_absolute_glyph_ranges() {
    let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
    let source = "Beta";
    let source_range = UiTextRange { start: 4, end: 8 };
    let style = UiResolvedStyle {
        font_size: 16.0,
        line_height: 20.0,
        ..UiResolvedStyle::default()
    };
    let layout_line = UiResolvedTextLine {
        text: source.to_string(),
        placement_frame: UiFrame::default(),
        frame: UiFrame::new(0.0, 0.0, 40.0, 20.0),
        source_range,
        visual_range: UiTextRange { start: 0, end: 4 },
        measured_width: 40.0,
        glyph_advances: vec![10.0; source.chars().count()],
        baseline: 16.0,
        direction: UiTextDirection::LeftToRight,
        runs: vec![UiResolvedTextRun {
            kind: UiTextRunKind::Plain,
            text: source.to_string(),
            source_range,
            visual_range: UiTextRange { start: 0, end: 4 },
            direction: UiTextDirection::LeftToRight,
        }],
        ellipsized: false,
    };
    let layout = UiResolvedTextLayout {
        writing_mode: UiTextWritingMode::HorizontalTb,
        font_size: style.font_size,
        line_height: style.line_height,
        measured_width: layout_line.measured_width,
        measured_height: layout_line.frame.height,
        source_range,
        lines: vec![layout_line],
        ..UiResolvedTextLayout::default()
    };
    let mut provider = SharedTextLayoutSession::new();

    let artifact = build_resolved_text_glyph_artifact(source, &style, &layout, &mut provider)
        .into_result()
        .expect("an exact source slice must shape successfully")
        .expect("an exact source slice must retain an artifact");
    let glyphs = &artifact.lines[0].as_ref().expect("artifact line").glyphs;

    assert_eq!(artifact.source_text_origin, source_range.start);
    assert!(!glyphs.is_empty());
    assert_eq!(
        glyphs.first().map(|glyph| glyph.source_range.start),
        Some(4)
    );
    assert_eq!(glyphs.last().map(|glyph| glyph.source_range.end), Some(8));
    assert!(glyphs.iter().all(|glyph| {
        source_range.start <= glyph.source_range.start && glyph.source_range.end <= source_range.end
    }));
}

#[test]
fn glyph_artifact_projects_a_retained_final_fragment_without_an_artifact_shape_request() {
    let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
    let source = "Reuse";
    let style = UiResolvedStyle {
        font_size: 16.0,
        line_height: 20.0,
        ..UiResolvedStyle::default()
    };
    let source_range = UiTextRange {
        start: 0,
        end: source.len(),
    };
    let mut layout_provider = SharedTextLayoutSession::new();
    let fragment = Arc::new(
        crate::text::layout::shape_horizontal_physical_line_fragment_with_provider(
            source,
            &crate::text::text_style(&style),
            TextDirection::LeftToRight.into(),
            source_range.into(),
            &mut layout_provider,
        )
        .into_result()
        .expect("the retained final fragment must shape"),
    );
    let layout_line = UiResolvedTextLine {
        text: source.to_string(),
        placement_frame: UiFrame::default(),
        frame: UiFrame::new(
            0.0,
            0.0,
            fragment.metrics().width,
            fragment.metrics().line_height,
        ),
        source_range,
        visual_range: source_range,
        measured_width: fragment.metrics().width,
        glyph_advances: fragment.grapheme_advances().to_vec(),
        baseline: fragment.metrics().baseline,
        direction: UiTextDirection::LeftToRight,
        runs: vec![UiResolvedTextRun {
            kind: UiTextRunKind::Plain,
            text: source.to_string(),
            source_range,
            visual_range: source_range,
            direction: UiTextDirection::LeftToRight,
        }],
        ellipsized: false,
    };
    let layout = UiResolvedTextLayout {
        writing_mode: UiTextWritingMode::HorizontalTb,
        font_size: style.font_size,
        line_height: fragment.metrics().line_height,
        measured_width: layout_line.measured_width,
        measured_height: layout_line.frame.height,
        source_range,
        lines: vec![layout_line],
        ..UiResolvedTextLayout::default()
    };
    assert!(
        retained_line_fragment_for_artifact(
            Some(&[Some(Arc::clone(&fragment))]),
            0,
            source,
            0,
            fragment.font_generation() ^ 1,
            &layout.lines[0],
        )
        .is_none(),
        "a fragment from another font generation must not bypass artifact shaping"
    );
    let mut artifact_provider = SharedTextLayoutSession::new();

    let artifact = build_resolved_text_glyph_artifact_with_line_fragments(
        Arc::from(source),
        &style,
        &layout,
        Some(&[Some(fragment)]),
        None,
        &mut artifact_provider,
    )
    .into_result()
    .expect("retained fragment projection must succeed")
    .expect("retained fragment projection must build an artifact");

    assert!(
        artifact.lines[0]
            .as_ref()
            .is_some_and(|line| !line.glyphs.is_empty())
    );
    let report = artifact_provider.cache_report();
    assert_eq!(report.entry_count, 0);
    assert_eq!(report.miss_count, 0);
}

#[test]
fn glyph_artifact_rejects_a_typed_shaping_failure_without_publishing_a_partial_artifact() {
    let source = "invalid";
    let style = UiResolvedStyle {
        font_size: 0.0,
        line_height: 20.0,
        ..UiResolvedStyle::default()
    };
    let source_range = UiTextRange {
        start: 0,
        end: source.len(),
    };
    let layout_line = UiResolvedTextLine {
        text: source.to_string(),
        placement_frame: UiFrame::default(),
        frame: UiFrame::new(0.0, 0.0, 48.0, 20.0),
        source_range,
        visual_range: source_range,
        measured_width: 48.0,
        glyph_advances: vec![8.0; source.len()],
        baseline: 16.0,
        direction: UiTextDirection::LeftToRight,
        runs: vec![UiResolvedTextRun {
            kind: UiTextRunKind::Plain,
            text: source.to_string(),
            source_range,
            visual_range: source_range,
            direction: UiTextDirection::LeftToRight,
        }],
        ellipsized: false,
    };
    let layout = UiResolvedTextLayout {
        writing_mode: UiTextWritingMode::HorizontalTb,
        font_size: style.font_size,
        line_height: style.line_height,
        measured_width: layout_line.measured_width,
        measured_height: layout_line.frame.height,
        source_range,
        lines: vec![layout_line],
        ..UiResolvedTextLayout::default()
    };
    let mut provider = SharedTextLayoutSession::new();

    assert!(matches!(
        build_resolved_text_glyph_artifact(source, &style, &layout, &mut provider),
        TextShapingOutcome::Failed(failure)
            if failure.error() == &TextLayoutError::InvalidFontSize
    ));
    let report = provider.cache_report();
    assert_eq!(report.entry_count, 0);
    assert_eq!(report.insert_count, 0);
}

#[test]
fn glyph_artifact_line_source_ranges_must_remain_owned_by_the_layout() {
    let layout_source_range = UiTextRange { start: 4, end: 12 };
    let mut line = UiResolvedTextLine {
        text: "glyph".to_string(),
        placement_frame: UiFrame::default(),
        frame: UiFrame::new(0.0, 0.0, 40.0, 12.0),
        source_range: UiTextRange { start: 4, end: 9 },
        visual_range: UiTextRange { start: 0, end: 5 },
        measured_width: 40.0,
        glyph_advances: vec![8.0; 5],
        baseline: 9.0,
        direction: UiTextDirection::LeftToRight,
        runs: vec![
            UiResolvedTextRun {
                kind: UiTextRunKind::Plain,
                text: "glyph".to_string(),
                source_range: UiTextRange { start: 4, end: 9 },
                visual_range: UiTextRange { start: 0, end: 5 },
                direction: UiTextDirection::LeftToRight,
            },
            UiResolvedTextRun {
                kind: UiTextRunKind::Plain,
                text: "...".to_string(),
                source_range: UiTextRange { start: 9, end: 9 },
                visual_range: UiTextRange { start: 5, end: 8 },
                direction: UiTextDirection::LeftToRight,
            },
        ],
        ellipsized: true,
    };

    assert!(artifact_line_source_ranges_are_owned_by_layout(
        layout_source_range,
        &line
    ));

    line.source_range.end = 13;
    assert!(!artifact_line_source_ranges_are_owned_by_layout(
        layout_source_range,
        &line
    ));

    line.source_range.end = 9;
    line.runs[0].source_range.start = 3;
    assert!(!artifact_line_source_ranges_are_owned_by_layout(
        layout_source_range,
        &line
    ));
}

#[test]
fn glyph_artifact_build_keeps_internal_shaping_out_of_the_neutral_dto_report() {
    let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
    let source = "Plain artifact";
    let style = UiResolvedStyle {
        font_size: 16.0,
        line_height: 20.0,
        ..UiResolvedStyle::default()
    };
    let layout_line = UiResolvedTextLine {
        text: source.to_string(),
        placement_frame: UiFrame::default(),
        frame: UiFrame::new(0.0, 0.0, 140.0, 20.0),
        source_range: UiTextRange {
            start: 0,
            end: source.len(),
        },
        visual_range: UiTextRange {
            start: 0,
            end: source.len(),
        },
        measured_width: 140.0,
        glyph_advances: vec![10.0; source.chars().count()],
        baseline: 16.0,
        direction: UiTextDirection::LeftToRight,
        runs: vec![UiResolvedTextRun {
            kind: UiTextRunKind::Plain,
            text: source.to_string(),
            source_range: UiTextRange {
                start: 0,
                end: source.len(),
            },
            visual_range: UiTextRange {
                start: 0,
                end: source.len(),
            },
            direction: UiTextDirection::LeftToRight,
        }],
        ellipsized: false,
    };
    let layout = UiResolvedTextLayout {
        writing_mode: UiTextWritingMode::HorizontalTb,
        font_size: style.font_size,
        line_height: style.line_height,
        measured_width: layout_line.measured_width,
        measured_height: layout_line.frame.height,
        source_range: layout_line.source_range,
        lines: vec![layout_line],
        ..UiResolvedTextLayout::default()
    };
    let neutral_projection_before = crate::text::service::current_thread_neutral_projection_count();
    let registration_batch_before =
        crate::text::font::current_thread_font_handle_registration_batch_count();
    let mut provider = SharedTextLayoutSession::new();

    let artifact = build_resolved_text_glyph_artifact(source, &style, &layout, &mut provider)
        .into_result()
        .expect("plain canonical shaping should succeed")
        .expect("plain canonical shaping should build a glyph artifact");
    let neutral_projection_after = crate::text::service::current_thread_neutral_projection_count();
    let registration_batch_after =
        crate::text::font::current_thread_font_handle_registration_batch_count();

    assert!(
        artifact.lines[0]
            .as_ref()
            .is_some_and(|line| !line.glyphs.is_empty())
    );
    assert_eq!(neutral_projection_after, neutral_projection_before);
    assert_eq!(registration_batch_after, registration_batch_before + 1);
}

#[cfg(feature = "profiling")]
#[test]
fn glyph_artifact_build_reports_the_shaped_cache_delta() {
    let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
    let source = "Artifact cache delta";
    let style = UiResolvedStyle {
        font_size: 16.0,
        line_height: 20.0,
        ..UiResolvedStyle::default()
    };
    let layout_line = UiResolvedTextLine {
        text: source.to_string(),
        placement_frame: UiFrame::default(),
        frame: UiFrame::new(0.0, 0.0, 160.0, 20.0),
        source_range: UiTextRange {
            start: 0,
            end: source.len(),
        },
        visual_range: UiTextRange {
            start: 0,
            end: source.len(),
        },
        measured_width: 160.0,
        glyph_advances: vec![10.0; source.chars().count()],
        baseline: 16.0,
        direction: UiTextDirection::LeftToRight,
        runs: vec![UiResolvedTextRun {
            kind: UiTextRunKind::Plain,
            text: source.to_string(),
            source_range: UiTextRange {
                start: 0,
                end: source.len(),
            },
            visual_range: UiTextRange {
                start: 0,
                end: source.len(),
            },
            direction: UiTextDirection::LeftToRight,
        }],
        ellipsized: false,
    };
    let layout = UiResolvedTextLayout {
        writing_mode: UiTextWritingMode::HorizontalTb,
        font_size: style.font_size,
        line_height: style.line_height,
        measured_width: layout_line.measured_width,
        measured_height: layout_line.frame.height,
        source_range: layout_line.source_range,
        lines: vec![layout_line],
        ..UiResolvedTextLayout::default()
    };
    let mut provider = SharedTextLayoutSession::new();
    let _capture_guard = crate::core::diagnostics::profiling::test_capture_lock();
    let mut config = crate::core::diagnostics::profiling::ProfileCaptureConfig::default();
    config.session_id = "glyph-artifact-shaped-cache-delta".to_owned();
    config.max_spans = 1;
    config.max_counters = 10;
    assert!(crate::core::diagnostics::profiling::start_capture(config).active);

    let artifact = build_resolved_text_glyph_artifact(source, &style, &layout, &mut provider)
        .into_result()
        .expect("plain canonical shaping should succeed")
        .expect("plain canonical shaping should build a glyph artifact");

    let snapshot = crate::core::diagnostics::profiling::snapshot();
    assert!(
        !crate::core::diagnostics::profiling::reset_capture().active,
        "artifact build profiling capture must reset before another test starts"
    );
    assert!(
        artifact.lines[0]
            .as_ref()
            .is_some_and(|line| !line.glyphs.is_empty())
    );
    assert_eq!(
        snapshot
            .spans
            .iter()
            .filter(|span| {
                span.category == "text.artifact"
                    && span.name == "build_resolved_text_glyph_artifact"
            })
            .count(),
        1
    );
    for (name, value) in [
        ("artifact_build_line_count", 1.0),
        ("artifact_build_shaped_cache_hit_count", 0.0),
        ("artifact_build_shaped_cache_miss_count", 1.0),
        ("artifact_build_retained_fragment_projection_count", 0.0),
        ("artifact_build_fallback_shape_request_count", 1.0),
        ("artifact_build_font_handle_registration_batch_count", 1.0),
        (
            "artifact_build_font_handle_registration_lock_acquire_count",
            1.0,
        ),
    ] {
        assert_eq!(
            snapshot
                .counters
                .iter()
                .find(|counter| counter.stream == "runtime" && counter.name == name)
                .map(|counter| counter.value),
            Some(value),
            "artifact build must report {name}"
        );
    }
    for name in [
        "artifact_build_font_handle_registration_lock_wait_nanos",
        "artifact_build_font_handle_registration_lock_hold_nanos",
        "artifact_build_font_handle_registration_snapshot_publish_count",
    ] {
        let value = snapshot
            .counters
            .iter()
            .find(|counter| counter.stream == "runtime" && counter.name == name)
            .map(|counter| counter.value)
            .expect("artifact build must report registry timing and publication deltas");
        assert!(
            value.is_finite() && value >= 0.0,
            "artifact build must report a finite non-negative {name}"
        );
    }
}

#[cfg(all(feature = "profiling", not(feature = "profiling-tracy")))]
#[test]
fn glyph_artifact_idle_cpu_profiler_skips_local_registration_measurement() {
    let _capture_guard = crate::core::diagnostics::profiling::test_capture_lock();
    assert!(
        !crate::core::diagnostics::profiling::capture_active(),
        "the idle-profiler contract requires capture to be inactive"
    );
    assert!(
        !artifact_local_profile_metrics_enabled(),
        "idle CPU profiling must not add local font-registration timing to artifact builds"
    );
}

#[test]
fn visual_glyph_artifact_keeps_contextual_arabic_glyphs_in_visual_order() {
    let line = UiResolvedTextLine {
        text: "مالس".to_string(),
        placement_frame: UiFrame::default(),
        frame: UiFrame::new(0.0, 0.0, 40.0, 12.0),
        source_range: UiTextRange { start: 0, end: 8 },
        visual_range: UiTextRange { start: 0, end: 8 },
        measured_width: 40.0,
        glyph_advances: vec![10.0; 4],
        baseline: 9.0,
        direction: UiTextDirection::RightToLeft,
        runs: vec![
            visual_run("م", 6, 8, 0, 2),
            visual_run("ا", 4, 6, 2, 4),
            visual_run("ل", 2, 4, 4, 6),
            visual_run("س", 0, 2, 6, 8),
        ],
        ellipsized: false,
    };

    let glyphs = visual_glyphs_for_line(
        "سلام",
        0,
        &line,
        vec![
            glyph(101, 0..2),
            glyph(102, 2..4),
            glyph(103, 4..6),
            glyph(104, 6..8),
        ],
    );

    assert_eq!(
        glyphs
            .iter()
            .map(|glyph| glyph.glyph_id)
            .collect::<Vec<_>>(),
        vec![104, 103, 102, 101]
    );
}

#[test]
fn visual_glyph_artifact_projects_resolved_advance_to_an_unsplit_ligature() {
    let line = UiResolvedTextLine {
        text: "fi".to_string(),
        placement_frame: UiFrame::default(),
        frame: UiFrame::new(0.0, 0.0, 30.0, 12.0),
        source_range: UiTextRange { start: 0, end: 2 },
        visual_range: UiTextRange { start: 0, end: 2 },
        measured_width: 30.0,
        glyph_advances: vec![12.0, 18.0],
        baseline: 9.0,
        direction: UiTextDirection::LeftToRight,
        runs: vec![zircon_runtime_interface::ui::surface::UiResolvedTextRun {
            kind: UiTextRunKind::Plain,
            text: "fi".to_string(),
            source_range: UiTextRange { start: 0, end: 2 },
            visual_range: UiTextRange { start: 0, end: 2 },
            direction: UiTextDirection::LeftToRight,
        }],
        ellipsized: false,
    };

    let glyphs = visual_glyphs_for_line("fi", 0, &line, vec![glyph(77, 0..2)]);

    assert_eq!(glyphs.len(), 1);
    assert_eq!(glyphs[0].advance, 30.0);
}

#[test]
fn visual_glyph_artifact_preserves_tab_and_justified_space_advances() {
    let line = UiResolvedTextLine {
        text: "a\tb c".to_string(),
        placement_frame: UiFrame::default(),
        frame: UiFrame::new(0.0, 0.0, 91.0, 12.0),
        source_range: UiTextRange { start: 0, end: 5 },
        visual_range: UiTextRange { start: 0, end: 5 },
        measured_width: 91.0,
        glyph_advances: vec![9.0, 40.0, 9.0, 24.0, 9.0],
        baseline: 9.0,
        direction: UiTextDirection::LeftToRight,
        runs: vec![visual_run("a\tb c", 0, 5, 0, 5)],
        ellipsized: false,
    };

    let glyphs = visual_glyphs_for_line(
        "a\tb c",
        0,
        &line,
        vec![
            glyph(1, 0..1),
            glyph(2, 1..2),
            glyph(3, 2..3),
            glyph(4, 3..4),
            glyph(5, 4..5),
        ],
    );

    assert_eq!(
        glyphs.iter().map(|glyph| glyph.advance).collect::<Vec<_>>(),
        line.glyph_advances
    );
}

fn visual_run(
    text: &str,
    source_start: usize,
    source_end: usize,
    visual_start: usize,
    visual_end: usize,
) -> zircon_runtime_interface::ui::surface::UiResolvedTextRun {
    UiResolvedTextRun {
        kind: UiTextRunKind::Plain,
        text: text.to_string(),
        source_range: UiTextRange {
            start: source_start,
            end: source_end,
        },
        visual_range: UiTextRange {
            start: visual_start,
            end: visual_end,
        },
        direction: UiTextDirection::RightToLeft,
    }
}

fn glyph(glyph_id: u32, source_range: std::ops::Range<usize>) -> TextGlyph {
    TextGlyph {
        glyph_id,
        source_range,
        visual_range: 0..0,
        advance: 10.0,
        position: [0.0, 0.0],
        offset: [0.0, 0.0],
        font_face: None,
        font_instance: None,
        rotation: TextGlyphRotation::None,
        bidi_level: 1,
        flags: TextGlyphFlags {
            right_to_left: true,
            ..TextGlyphFlags::default()
        },
        requires_rasterization: true,
    }
}
