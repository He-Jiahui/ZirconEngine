use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Instant;

use zircon_runtime::core::framework::text::{TextGlyph, TextGlyphFlags, TextGlyphRotation};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::artifact::artifact_glyph_geometry;
use super::super::layout_text_run;
use super::super::layout_text_run_with_layout_policy;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_text::HostTextLayoutPolicy;

#[test]
fn artifact_glyph_geometry_uses_the_runtime_pen_position_and_offset() {
    let glyph = TextGlyph {
        glyph_id: 42,
        source_range: 0..1,
        visual_range: 0..1,
        advance: 9.0,
        position: [5.0, 2.0],
        offset: [-1.5, 0.5],
        font_face: None,
        font_instance: None,
        rotation: TextGlyphRotation::None,
        bidi_level: 0,
        flags: TextGlyphFlags::default(),
        requires_rasterization: true,
    };

    let positioned = artifact_glyph_geometry(&glyph, 10.0, 20.0, 16.0, 3)
        .expect("finite u16 runtime glyph should be positionable");

    assert_eq!(positioned.glyph_index, 42);
    assert_eq!(positioned.px, 16.0);
    assert_eq!(positioned.x, 13.5);
    assert_eq!(positioned.origin_x, 13.5);
    assert_eq!(positioned.y, 22.5);
    assert_eq!(positioned.raster_font_index, Some(3));
}

#[cfg(feature = "profiling")]
#[test]
fn artifact_glyph_projection_emits_one_host_painter_profile_span() {
    let _capture_guard = zircon_runtime::core::diagnostics::profiling::test_capture_lock();
    let mut config = zircon_runtime::core::diagnostics::profiling::ProfileCaptureConfig::default();
    config.session_id = "retained-artifact-glyph-projection".to_owned();
    config.max_spans = 16;
    config.max_counters = 16;
    assert!(zircon_runtime::core::diagnostics::profiling::start_capture(config).active);

    let layout = layout_text_run(
        &FrameRect {
            x: 7.0,
            y: 3.0,
            width: 180.0,
            height: 28.0,
        },
        "Runtime artifact projection",
        16.0,
        20.0,
        UiTextRunPaintStyle::default(),
    );
    assert!(
        !layout.glyphs.is_empty(),
        "profile fixture must produce a text layout"
    );
    assert!(
        !layout.artifact_raster_fonts.is_empty(),
        "profile fixture must consume the exact runtime glyph artifact"
    );

    let snapshot = zircon_runtime::core::diagnostics::profiling::snapshot();
    assert!(
        !zircon_runtime::core::diagnostics::profiling::reset_capture().active,
        "artifact projection profiling capture must reset before another test starts"
    );
    assert_eq!(
        snapshot
            .spans
            .iter()
            .filter(|span| {
                span.category == "host_painter" && span.name == "runtime_artifact_glyph_projection"
            })
            .count(),
        1
    );
    assert_eq!(
        snapshot
            .spans
            .iter()
            .filter(|span| { span.category == "text.surface" && span.name == "shape_text_line" })
            .count(),
        0,
        "exact runtime artifact projection must not fall back to a second surface shaping pass"
    );
    assert_eq!(
        profile_counter_total(
            &snapshot,
            "retained_text_artifact_projection_layout_hit_count"
        ),
        1.0
    );
    assert_eq!(
        profile_counter_total(
            &snapshot,
            "retained_text_artifact_projection_layout_miss_count"
        ),
        0.0
    );
    assert!(profile_counter_total(&snapshot, "retained_text_artifact_candidate_glyph_count") > 0.0);
    assert_eq!(
        profile_counter_total(&snapshot, "retained_text_surface_shape_line_count"),
        0.0
    );
    assert_eq!(
        profile_counter_total(&snapshot, "retained_text_shaped_glyph_copy_count"),
        0.0
    );
    assert_eq!(
        profile_counter_total(&snapshot, "retained_text_shaped_glyph_copy_line_count"),
        0.0
    );
    assert_eq!(
        profile_span_count(&snapshot, "text.surface", "artifact_raster_face_resolution"),
        1
    );
}

#[cfg(feature = "profiling")]
#[test]
fn artifact_multiline_projection_resolves_faces_once_per_layout() {
    let _capture_guard = zircon_runtime::core::diagnostics::profiling::test_capture_lock();
    let mut config = zircon_runtime::core::diagnostics::profiling::ProfileCaptureConfig::default();
    config.session_id = "retained-artifact-multiline-face-resolution".to_owned();
    config.max_spans = 64;
    config.max_counters = 64;
    assert!(zircon_runtime::core::diagnostics::profiling::start_capture(config).active);

    let layout = layout_text_run_with_layout_policy(
        &FrameRect {
            x: 317.0,
            y: 119.0,
            width: 96.0,
            height: 240.0,
        },
        "Alpha Bravo Charlie Delta Echo Foxtrot Golf Hotel",
        16.0,
        20.0,
        UiTextRunPaintStyle::default(),
        HostTextLayoutPolicy::WordWrap,
    );
    assert!(
        !layout.glyphs.is_empty(),
        "multiline artifact fixture must produce retained glyphs"
    );
    assert!(
        !layout.artifact_raster_fonts.is_empty(),
        "multiline artifact fixture must retain exact runtime faces"
    );

    let snapshot = zircon_runtime::core::diagnostics::profiling::snapshot();
    assert!(
        !zircon_runtime::core::diagnostics::profiling::reset_capture().active,
        "artifact profiling capture must reset before another test starts"
    );
    assert!(
        profile_counter_total(&snapshot, "retained_text_artifact_candidate_line_count") > 1.0,
        "fixture must create more than one visual line"
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
        1,
        "all visual lines in one artifact layout must share one face-resolution batch"
    );
}

#[cfg(feature = "profiling")]
fn profile_counter_total(
    snapshot: &zircon_runtime::core::diagnostics::profiling::ProfileSnapshot,
    name: &str,
) -> f64 {
    snapshot
        .counters
        .iter()
        .filter(|counter| counter.stream == "editor" && counter.name == name)
        .map(|counter| counter.value)
        .sum()
}

#[cfg(feature = "profiling")]
fn profile_span_count(
    snapshot: &zircon_runtime::core::diagnostics::profiling::ProfileSnapshot,
    category: &str,
    name: &str,
) -> usize {
    snapshot
        .spans
        .iter()
        .filter(|span| span.category == category && span.name == name)
        .count()
}

#[test]
#[ignore = "managed Text02 direct-artifact capture-inactive 31-sample scale timing evidence; no machine-time acceptance threshold"]
fn artifact_glyph_projection_time_evidence_reports_capture_inactive_p50_p95() {
    const SAMPLE_COUNT: usize = 31;
    const SCALE_UNITS: [usize; 4] = [1, 100, 1_000, 10_000];
    static CACHE_MISS_SEQUENCE: AtomicU32 = AtomicU32::new(0);

    assert!(
        !zircon_runtime::core::diagnostics::profiling::feature_enabled()
            && !cfg!(feature = "profiling-tracy"),
        "Text02 wall-clock evidence requires profiling and Tracy features to stay disabled"
    );
    for workload in ArtifactProjectionBenchmarkWorkload::ALL {
        for unit_count in SCALE_UNITS {
            let text = workload.unit().repeat(unit_count);
            let first_sample_x = CACHE_MISS_SEQUENCE.fetch_add(
                u32::try_from(SAMPLE_COUNT).expect("sample count must fit cache sequence"),
                Ordering::Relaxed,
            );

            let mut samples_ns = Vec::with_capacity(SAMPLE_COUNT);
            for sample_index in 0..SAMPLE_COUNT {
                // Position is part of the current host cache key. Vary it so this measures the
                // direct artifact projection rather than returning a cached PaintTextLayout.
                let rect = workload.frame(first_sample_x + sample_index as u32);
                let started = Instant::now();
                let layout = layout_text_run_with_layout_policy(
                    &rect,
                    &text,
                    16.0,
                    20.0,
                    UiTextRunPaintStyle::default(),
                    HostTextLayoutPolicy::WordWrap,
                );
                samples_ns.push(started.elapsed().as_nanos());
                assert!(
                    !layout.glyphs.is_empty(),
                    "{} at {unit_count} units must produce a retained layout",
                    workload.label()
                );
                assert!(
                    !layout.artifact_raster_fonts.is_empty(),
                    "{} at {unit_count} units must retain exact runtime artifact faces",
                    workload.label()
                );
            }

            println!(
                "TEXT02_ARTIFACT_PROJECTION_TIME workload={} units={unit_count} samples={SAMPLE_COUNT} capture=inactive cache_key=varied p50_ns={} p95_ns={}",
                workload.label(),
                percentile_ns(&mut samples_ns, 50),
                percentile_ns(&mut samples_ns, 95),
            );
        }
    }
}

#[cfg(feature = "profiling")]
#[test]
#[ignore = "managed Text02 direct-artifact counter topology evidence"]
fn artifact_glyph_projection_counter_evidence_reports_reuse_without_second_shaping() {
    const SAMPLE_COUNT: usize = 31;
    const SCALE_UNITS: [usize; 4] = [1, 100, 1_000, 10_000];
    static CACHE_MISS_SEQUENCE: AtomicU32 = AtomicU32::new(0);

    let _capture_guard = zircon_runtime::core::diagnostics::profiling::test_capture_lock();
    for workload in ArtifactProjectionBenchmarkWorkload::ALL {
        for unit_count in SCALE_UNITS {
            let text = workload.unit().repeat(unit_count);
            let first_sample_x = CACHE_MISS_SEQUENCE.fetch_add(
                u32::try_from(SAMPLE_COUNT).expect("sample count must fit cache sequence"),
                Ordering::Relaxed,
            );
            let mut config =
                zircon_runtime::core::diagnostics::profiling::ProfileCaptureConfig::default();
            config.session_id = format!(
                "retained-artifact-projection-{}-{unit_count}",
                workload.label()
            );
            config.max_spans = 4_096;
            config.max_counters = 4_096;
            assert!(zircon_runtime::core::diagnostics::profiling::start_capture(config).active);

            for sample_index in 0..SAMPLE_COUNT {
                let rect = workload.frame(first_sample_x + sample_index as u32);
                let layout = layout_text_run_with_layout_policy(
                    &rect,
                    &text,
                    16.0,
                    20.0,
                    UiTextRunPaintStyle::default(),
                    HostTextLayoutPolicy::WordWrap,
                );
                assert!(
                    !layout.glyphs.is_empty(),
                    "{} at {unit_count} units must produce a retained layout",
                    workload.label()
                );
                assert!(
                    !layout.artifact_raster_fonts.is_empty(),
                    "{} at {unit_count} units must retain exact runtime artifact faces",
                    workload.label()
                );
            }

            let snapshot = zircon_runtime::core::diagnostics::profiling::snapshot();
            assert!(
                !zircon_runtime::core::diagnostics::profiling::reset_capture().active,
                "artifact scale profiling capture must reset before the next workload"
            );
            let expected_samples = SAMPLE_COUNT as f64;
            assert_eq!(
                profile_counter_total(
                    &snapshot,
                    "retained_text_artifact_projection_layout_hit_count"
                ),
                expected_samples
            );
            assert_eq!(
                profile_counter_total(
                    &snapshot,
                    "retained_text_artifact_projection_layout_miss_count"
                ),
                0.0
            );
            assert!(
                profile_counter_total(&snapshot, "retained_text_artifact_candidate_glyph_count")
                    > 0.0
            );
            assert_eq!(
                profile_counter_total(&snapshot, "retained_text_surface_shape_line_count"),
                0.0
            );
            assert_eq!(
                profile_counter_total(&snapshot, "retained_text_shaped_glyph_copy_count"),
                0.0
            );
            assert_eq!(
                profile_counter_total(&snapshot, "retained_text_shaped_glyph_copy_line_count"),
                0.0
            );
            assert_eq!(
                profile_span_count(&snapshot, "text.surface", "artifact_face_snapshot"),
                SAMPLE_COUNT
            );
            assert_eq!(
                profile_span_count(&snapshot, "text.surface", "artifact_raster_face_resolution"),
                SAMPLE_COUNT,
                "each exact artifact layout must resolve all visual lines through one face batch"
            );

            println!(
                "TEXT02_ARTIFACT_PROJECTION_COUNTERS workload={} units={unit_count} samples={SAMPLE_COUNT} cache_key=varied artifact_projection_hits={} surface_shape_lines={} shaped_glyph_copies={}",
                workload.label(),
                profile_counter_total(
                    &snapshot,
                    "retained_text_artifact_projection_layout_hit_count"
                ),
                profile_counter_total(&snapshot, "retained_text_surface_shape_line_count"),
                profile_counter_total(&snapshot, "retained_text_shaped_glyph_copy_count"),
            );
        }
    }
}

#[derive(Clone, Copy)]
enum ArtifactProjectionBenchmarkWorkload {
    Latin,
    Cjk,
    Rtl,
    Ligature,
    WrappedLabel,
}

impl ArtifactProjectionBenchmarkWorkload {
    const ALL: [Self; 5] = [
        Self::Latin,
        Self::Cjk,
        Self::Rtl,
        Self::Ligature,
        Self::WrappedLabel,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::Latin => "latin",
            Self::Cjk => "cjk",
            Self::Rtl => "rtl",
            Self::Ligature => "ligature",
            Self::WrappedLabel => "wrapped_label",
        }
    }

    const fn unit(self) -> &'static str {
        match self {
            Self::Latin => "A",
            Self::Cjk => "汉",
            Self::Rtl => "ب",
            Self::Ligature => "office",
            Self::WrappedLabel => "Alpha Bravo ",
        }
    }

    fn frame(self, cache_miss_sequence: u32) -> FrameRect {
        let (width, height) = match self {
            Self::WrappedLabel => (160.0, 1_000_000.0),
            Self::Latin | Self::Cjk | Self::Rtl | Self::Ligature => (1_000_000.0, 48.0),
        };
        FrameRect {
            x: 8.0 + cache_miss_sequence as f32,
            y: 4.0,
            width,
            height,
        }
    }
}

fn percentile_ns(samples: &mut [u128], percentile: usize) -> u128 {
    assert!(
        !samples.is_empty(),
        "percentile requires at least one sample"
    );
    assert!((1..=100).contains(&percentile));
    samples.sort_unstable();
    let index = samples
        .len()
        .saturating_mul(percentile)
        .div_ceil(100)
        .saturating_sub(1);
    samples[index]
}
