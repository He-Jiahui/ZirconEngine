use std::{mem::size_of, sync::Arc};

use super::{
    BackendShapeRequest, HorizontalGlyphMetricSpan, HorizontalLineRawMetrics, Iso15924Tag,
    LineBreakTailoringProfile, ShapedGlyph, ShapedGlyphBreakSafety, ShapedGlyphClusterFlags,
    ShapedGlyphLineBreakOpportunity, ShapedGlyphLineBreakReceipt, ShapedGlyphRotation,
    ShapedGlyphRun, ShapedGlyphScript, ShapedHardLine, TextOrientation, VerticalMode,
    compiled_unicode_data_snapshot_id, normalized_open_type_features,
};
use crate::{
    core::framework::text::{TextDirection, TextLayoutError},
    text::{FontFaceId, OpenTypeFeature, TextRange, TextStyle},
};

#[test]
fn open_type_feature_normalization_keeps_one_last_declared_value_per_tag() {
    let normalized = normalized_open_type_features(&[
        OpenTypeFeature::new(*b"ss01", 1),
        OpenTypeFeature::new(*b"liga", 1),
        OpenTypeFeature::new(*b"liga", 0),
        OpenTypeFeature::new(*b"kern", 0),
        OpenTypeFeature::new(*b"ss01", 2),
    ]);

    assert_eq!(
        normalized,
        vec![
            OpenTypeFeature::new(*b"kern", 0),
            OpenTypeFeature::new(*b"liga", 0),
            OpenTypeFeature::new(*b"ss01", 2),
        ]
    );
    assert_eq!(normalized_open_type_features(&normalized), normalized);
}

#[test]
fn backend_shape_request_canonicalizes_language_before_backend_use() {
    let style = TextStyle {
        language: Some(" ZH_Hans_CN ".to_string()),
        ..TextStyle::default()
    };
    let current_snapshot = compiled_unicode_data_snapshot_id();
    let request_snapshot =
        current_snapshot.with_generation_for_test(current_snapshot.generation().saturating_add(1));
    let canonical = BackendShapeRequest::horizontal(
        "locale",
        &style,
        TextDirection::LeftToRight,
        TextRange { start: 0, end: 6 },
    )
    .with_unicode_data_snapshot_for_test(request_snapshot)
    .canonicalized()
    .expect("valid BCP 47 language tag");

    assert_eq!(canonical.request().language, Some("zh-Hans-CN"));
    assert!(canonical.request().language_is_canonical());
    let fallback_key = canonical
        .request()
        .language_fallback_key()
        .expect("canonical request retains its font fallback identity");
    let fallback_script = fallback_key.script().expect("explicit script is retained");
    let fallback_region = fallback_key.region().expect("explicit region is retained");
    assert_eq!(fallback_key.language().as_str(), "zh");
    assert_eq!(fallback_script.as_str(), "Hans");
    assert_eq!(fallback_region.as_str(), "CN");
    assert_eq!(
        canonical
            .request()
            .explicit_language_script()
            .and_then(|script| script.as_str().map(str::to_owned))
            .as_deref(),
        Some("Hans")
    );
    assert_eq!(
        canonical.request().unicode_data_snapshot(),
        request_snapshot
    );
    let repeated = canonical
        .request()
        .canonicalized()
        .expect("canonical request remains valid");
    assert_eq!(repeated.request().language, Some("zh-Hans-CN"));
    assert_eq!(
        repeated
            .request()
            .explicit_language_script()
            .and_then(|script| script.as_str().map(str::to_owned))
            .as_deref(),
        Some("Hans")
    );
    assert_eq!(
        repeated.request().language_fallback_key(),
        Some(fallback_key)
    );
    assert_eq!(repeated.request().unicode_data_snapshot(), request_snapshot);
}

#[test]
fn backend_shape_request_rejects_invalid_language_before_cache_use() {
    let style = TextStyle {
        language: Some("en--US".to_string()),
        ..TextStyle::default()
    };
    let canonical = BackendShapeRequest::horizontal(
        "locale",
        &style,
        TextDirection::LeftToRight,
        TextRange { start: 0, end: 6 },
    )
    .canonicalized();

    assert!(matches!(canonical, Err(TextLayoutError::InvalidLanguage)));
}

#[test]
fn backend_shape_request_rejects_source_range_that_does_not_cover_text() {
    let style = TextStyle::default();

    let non_zero_absolute_range = BackendShapeRequest::horizontal(
        "locale",
        &style,
        TextDirection::LeftToRight,
        TextRange { start: 11, end: 17 },
    )
    .canonicalized();
    assert!(non_zero_absolute_range.is_ok());

    let mismatched_span = BackendShapeRequest::horizontal(
        "locale",
        &style,
        TextDirection::LeftToRight,
        TextRange { start: 11, end: 18 },
    )
    .canonicalized();
    assert!(matches!(
        mismatched_span,
        Err(TextLayoutError::BidiInvariant)
    ));

    let reversed_range = BackendShapeRequest::horizontal(
        "locale",
        &style,
        TextDirection::LeftToRight,
        TextRange { start: 18, end: 11 },
    )
    .canonicalized();
    assert!(matches!(
        reversed_range,
        Err(TextLayoutError::BidiInvariant)
    ));
}

#[test]
fn iso15924_tag_is_inline_copy_and_serde_remains_string_compatible() {
    let script = ShapedGlyphScript {
        iso15924: Iso15924Tag::parse("Latn").expect("valid ISO15924 fixture"),
    };

    assert_eq!(size_of::<Iso15924Tag>(), 4);
    assert_eq!(script.iso15924, "Latn");
    let json = serde_json::to_string(&script).expect("script serializes");
    assert_eq!(json, r#"{"iso15924":"Latn"}"#);
    assert_eq!(
        serde_json::from_str::<ShapedGlyphScript>(&json).expect("script deserializes"),
        script
    );
    assert!(serde_json::from_str::<ShapedGlyphScript>(r#"{"iso15924":"Latin"}"#).is_err());
    assert!(serde_json::from_str::<ShapedGlyphScript>(r#"{"iso15924":"La1n"}"#).is_err());
}

#[test]
fn cluster_break_safety_roundtrips_and_legacy_flags_default_to_unknown() {
    assert_eq!(size_of::<ShapedGlyphBreakSafety>(), 1);
    let flags = ShapedGlyphClusterFlags {
        cluster_start: true,
        ..ShapedGlyphClusterFlags::default()
    }
    .with_direct_break_safety(true);
    assert_eq!(flags.break_safety, ShapedGlyphBreakSafety::RequiresReshape);
    assert_eq!(
        ShapedGlyphClusterFlags::default()
            .with_direct_break_safety(true)
            .break_safety,
        ShapedGlyphBreakSafety::Unknown
    );
    let json = serde_json::to_string(&flags).expect("cluster flags serialize");
    assert_eq!(
        serde_json::from_str::<ShapedGlyphClusterFlags>(&json).expect("cluster flags deserialize"),
        flags
    );

    let mut legacy = serde_json::to_value(flags).expect("cluster flags convert to value");
    legacy
        .as_object_mut()
        .expect("cluster flags serialize as an object")
        .remove("break_safety");
    assert_eq!(
        serde_json::from_value::<ShapedGlyphClusterFlags>(legacy)
            .expect("legacy cluster flags deserialize")
            .break_safety,
        ShapedGlyphBreakSafety::Unknown
    );
}

#[test]
fn cluster_line_break_receipt_roundtrips_and_legacy_flags_default_to_unknown() {
    assert_eq!(size_of::<LineBreakTailoringProfile>(), 1);
    assert_eq!(size_of::<ShapedGlyphLineBreakOpportunity>(), 1);
    assert_eq!(size_of::<ShapedGlyphLineBreakReceipt>(), 2);
    assert_eq!(size_of::<ShapedGlyphClusterFlags>(), 11);
    let flags = ShapedGlyphClusterFlags {
        cluster_start: true,
        line_break: ShapedGlyphLineBreakReceipt {
            profile: LineBreakTailoringProfile::UnicodeDefault,
            opportunity: ShapedGlyphLineBreakOpportunity::ProviderAllowed,
        },
        ..ShapedGlyphClusterFlags::default()
    };
    let json = serde_json::to_string(&flags).expect("cluster flags serialize");
    assert_eq!(
        serde_json::from_str::<ShapedGlyphClusterFlags>(&json).expect("cluster flags deserialize"),
        flags
    );

    let mut legacy = serde_json::to_value(flags).expect("cluster flags convert to value");
    legacy
        .as_object_mut()
        .expect("cluster flags serialize as an object")
        .remove("line_break");
    assert_eq!(
        serde_json::from_value::<ShapedGlyphClusterFlags>(legacy)
            .expect("legacy cluster flags deserialize")
            .line_break,
        ShapedGlyphLineBreakReceipt::default()
    );
}

#[test]
fn shaped_lines_borrow_absolute_ranges_from_one_shared_source() {
    let source: Arc<str> = Arc::from("alpha beta");
    let raw_metrics =
        HorizontalLineRawMetrics::new(11.0, 3.0, 2.0).expect("finite positive raw metrics");
    let run = ShapedGlyphRun {
        source_text: Arc::clone(&source),
        source_range: TextRange { start: 40, end: 50 },
        unicode_data_snapshot: compiled_unicode_data_snapshot_id(),
        primary_face_id: Some(FontFaceId(7)),
        direction: TextDirection::LeftToRight,
        orientation: TextOrientation::Horizontal,
        vertical_mode: VerticalMode::Mixed,
        include_kerning: true,
        measured_width: 10.0,
        measured_height: 16.0,
        horizontal_composition_receipt: None,
        horizontal_line_raw_metrics: vec![Some(raw_metrics)],
        horizontal_glyph_metric_spans: vec![HorizontalGlyphMetricSpan {
            line_index: 0,
            glyph_start: 0,
            glyph_end: 1,
            metrics: raw_metrics,
        }],
        lines: vec![ShapedHardLine {
            line_index: 0,
            source_range: TextRange { start: 46, end: 50 },
            visual_range: TextRange { start: 6, end: 10 },
            measured_width: 4.0,
            baseline: 12.0,
            line_height: 16.0,
            glyphs: vec![ShapedGlyph {
                glyph_id: 7,
                font_id: None,
                font_instance_id: None,
                source_range: TextRange { start: 46, end: 50 },
                visual_range: TextRange { start: 6, end: 10 },
                advance: 4.0,
                x: 0.0,
                y: 0.0,
                offset_x: 0.0,
                offset_y: 0.0,
                direction: TextDirection::LeftToRight,
                bidi_level: 0,
                cluster_flags: ShapedGlyphClusterFlags {
                    cluster_start: true,
                    ..ShapedGlyphClusterFlags::default()
                },
                rotation: ShapedGlyphRotation::None,
                script: ShapedGlyphScript {
                    iso15924: Iso15924Tag::parse("Latn").expect("valid ISO15924 fixture"),
                },
            }],
        }],
    };
    let cloned = run.clone();
    let json = serde_json::to_string(&run).expect("shaped run serializes");
    let mut legacy_wire = serde_json::to_value(&run).expect("shaped run converts to value");
    legacy_wire
        .as_object_mut()
        .expect("shaped run serializes as an object")
        .remove("unicode_data_snapshot");
    let roundtrip = serde_json::from_str::<ShapedGlyphRun>(&json).expect("shaped run deserializes");
    let mut wire_roundtrip_expected = run.clone();
    wire_roundtrip_expected.horizontal_line_raw_metrics.clear();
    wire_roundtrip_expected
        .horizontal_glyph_metric_spans
        .clear();

    assert_eq!(run.hard_line_text(&run.lines[0]), Some("beta"));
    assert!(Arc::ptr_eq(&run.source_text, &cloned.source_text));
    assert_eq!(roundtrip, wire_roundtrip_expected);
    assert!(serde_json::from_value::<ShapedGlyphRun>(legacy_wire).is_err());
    assert!(!json.contains("horizontal_line_raw_metrics"));
    assert!(!json.contains("horizontal_glyph_metric_spans"));
    assert_eq!(roundtrip.primary_face_id, Some(FontFaceId(7)));
    assert_eq!(roundtrip.hard_line_text(&roundtrip.lines[0]), Some("beta"));
    assert_eq!(roundtrip.horizontal_line_raw_metrics_at(0), None);
    assert_eq!(roundtrip.horizontal_line_raw_metrics_at(1), None);
    let spans = run
        .horizontal_glyph_metric_spans_for_line(0)
        .expect("horizontal span sidecar remains line-aligned");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].glyph_start, 0);
    assert_eq!(spans[0].glyph_end, 1);
    assert_eq!(spans[0].metrics, raw_metrics);
    assert_eq!(roundtrip.horizontal_glyph_metric_spans_for_line(0), None);

    let mut legacy_json = serde_json::to_value(&run).expect("shaped run serializes to JSON");
    legacy_json
        .as_object_mut()
        .expect("shaped run JSON is an object")
        .remove("primary_face_id");
    let legacy = serde_json::from_value::<ShapedGlyphRun>(legacy_json)
        .expect("older shaped run payload still deserializes");
    assert_eq!(legacy.primary_face_id, None);
    assert_eq!(legacy.horizontal_line_raw_metrics_at(0), None);
}
