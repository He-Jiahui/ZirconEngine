use std::path::Path;

use crate::asset::{FontAsset, FontAssetFaceMetrics, FontAssetRenderStrategy};
use crate::text::font::FontDatabase;
use crate::text::{CompositeFontDescriptor, FontFamilyName, FontScript, SubFontRange, TextStyle};

use super::{
    font_chain_line_metric_envelope, primary_face_covers_all_hard_line_content,
    scaled_layout_extents, SelectedFaceLineExtents,
};

#[test]
fn primary_only_height_certificate_excludes_fallback_content() {
    let mut database = FontDatabase::default();
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
    database
        .register_font_file(assets.join("FiraSans-Regular.ttf"), Some("Primary"), 0)
        .expect("register primary font");
    let style = TextStyle {
        font_family: Some("Primary".to_string()),
        ..TextStyle::default()
    };

    assert!(primary_face_covers_all_hard_line_content(
        &database,
        &style,
        "Latin\r\ncontent\u{2028}only"
    ));
    assert!(!primary_face_covers_all_hard_line_content(
        &database,
        &style,
        "\u{4e16}\u{754c}"
    ));
}

#[test]
fn font_chain_metric_envelope_uses_the_requested_font_asset_owner() {
    let mut database = FontDatabase::default();
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
    database
        .register_font_file(
            assets.join("FiraSans-Regular.ttf"),
            Some("Project Default Sans"),
            0,
        )
        .expect("register project default font");
    assert!(database.set_default_ui_family("Project Default Sans"));
    let owner = "res://fonts/layout-mono.font.toml";
    let asset = FontAsset {
        source: "FiraMono-subset.ttf".to_string(),
        family: Some("Layout Asset Mono".to_string()),
        render_mode: None,
        face_index: 0,
        family_members: Vec::new(),
        variable_instances: Vec::new(),
        fallback_families: Vec::new(),
        composite_font: None,
        render_strategy: FontAssetRenderStrategy::default(),
        metadata: None,
    };
    let registered = database
        .replace_font_asset(owner, &asset, assets.join("FiraMono-subset.ttf"))
        .expect("register layout font asset");
    let asset_face = registered.faces[0];
    let font_size = 20.0;
    let style = TextStyle {
        font: Some(owner.to_string()),
        font_size,
        ..TextStyle::default()
    };

    let envelope = font_chain_line_metric_envelope(&database, &style)
        .expect("font asset chain must produce a line metric envelope");
    let metrics = database
        .face_metrics(asset_face)
        .expect("read asset face metrics")
        .expect("asset face metrics are tracked");
    let (ascent, descent, line_gap) =
        scaled_layout_extents(metrics, font_size).expect("valid asset metrics");

    assert!(
        (envelope.minimum_line_height() - (ascent + descent + line_gap)).abs() < 0.001,
        "the fixed-height certificate must use the FontObject primary face"
    );
}

#[test]
fn unavailable_font_asset_typeface_does_not_change_the_default_metric_envelope() {
    let mut database = FontDatabase::default();
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
    database
        .register_font_file(
            assets.join("FiraSans-Regular.ttf"),
            Some("Unavailable Owner Typeface"),
            0,
        )
        .expect("global homonym should register");
    let runtime_default = database
        .register_font_file(
            assets.join("FiraMono-subset.ttf"),
            Some("Runtime Metric Default"),
            0,
        )
        .expect("runtime metric default should register");
    assert!(database.set_runtime_default_primary_face(runtime_default));
    let font_size = 20.0;
    let style = TextStyle {
        font: Some("res://fonts/unavailable-metrics.font.toml".to_string()),
        font_family: Some("Unavailable Owner Typeface".to_string()),
        font_size,
        ..TextStyle::default()
    };

    let envelope = font_chain_line_metric_envelope(&database, &style)
        .expect("runtime default must provide the recovery metric envelope");
    let metrics = database
        .face_metrics(runtime_default)
        .expect("read runtime default metrics")
        .expect("runtime default metrics are tracked");
    let (ascent, descent, line_gap) =
        scaled_layout_extents(metrics, font_size).expect("valid runtime default metrics");

    assert!((envelope.minimum_line_height() - (ascent + descent + line_gap)).abs() < 0.001);
}

#[test]
fn font_chain_metric_envelope_includes_the_runtime_last_resort_face() {
    let mut database = FontDatabase::default();
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
    let primary = database
        .register_font_file(assets.join("FiraSans-Regular.ttf"), Some("Primary"), 0)
        .expect("register primary font");
    let last_resort = database
        .register_font_file(
            assets.join("FiraMono-subset.ttf"),
            Some("Runtime Last Resort"),
            0,
        )
        .expect("register last-resort font");
    assert!(database.set_runtime_last_resort_face(last_resort));
    let style = TextStyle {
        font_family: Some("Primary".to_string()),
        font_size: 20.0,
        ..TextStyle::default()
    };
    let primary_metrics = database
        .face_metrics(primary)
        .expect("read primary metrics")
        .expect("primary metrics are tracked");
    let last_resort_metrics = database
        .face_metrics(last_resort)
        .expect("read last-resort metrics")
        .expect("last-resort metrics are tracked");
    let (primary_ascent, primary_descent, primary_gap) =
        scaled_layout_extents(primary_metrics, style.font_size).expect("primary extents");
    let (last_resort_ascent, last_resort_descent, _) =
        scaled_layout_extents(last_resort_metrics, style.font_size).expect("last-resort extents");

    let envelope = font_chain_line_metric_envelope(&database, &style)
        .expect("last-resort chain must expose a metric envelope");

    let expected = primary_ascent.max(last_resort_ascent)
        + primary_descent.max(last_resort_descent)
        + primary_gap;
    assert!((envelope.minimum_line_height() - expected).abs() < 0.001);
}

#[test]
fn layout_extents_do_not_promote_windows_clip_metrics() {
    let metrics = FontAssetFaceMetrics {
        units_per_em: 1_000,
        ascender: 700,
        descender: -200,
        line_gap: 100,
        uses_typographic_metrics: false,
        windows_ascender: 1_100,
        windows_descender: 450,
        ..FontAssetFaceMetrics::default()
    };

    let (ascent, descent, line_gap) =
        scaled_layout_extents(metrics, 20.0).expect("valid normalized face metrics");

    assert_eq!((ascent, descent, line_gap), (14.0, 4.0, 2.0));
}

#[test]
fn selected_face_content_uses_primary_face_line_gap() {
    let extents = super::SelectedFaceLineExtents {
        ascent: 14.0,
        descent: 4.0,
        selected_face_line_gap: 6.0,
        primary_face_line_gap: Some(2.0),
        has_face_metrics: true,
        has_primary_face_metrics: true,
    };

    let envelope = extents
        .resolve_content_envelope(12.0)
        .expect("selected face metrics must resolve an envelope");

    assert_eq!(envelope.line_height, 20.0);
    assert_eq!(envelope.baseline_from_top, 15.0);
}

#[test]
fn selected_face_extents_exposes_validated_raw_horizontal_metrics() {
    let extents = super::SelectedFaceLineExtents {
        ascent: 14.0,
        descent: 4.0,
        selected_face_line_gap: 6.0,
        primary_face_line_gap: Some(2.0),
        has_face_metrics: true,
        has_primary_face_metrics: true,
    };

    let metrics = extents
        .raw_horizontal_metrics()
        .expect("selected faces provide raw horizontal metrics");

    assert_eq!(metrics.ascent(), 14.0);
    assert_eq!(metrics.descent(), 4.0);
    assert_eq!(metrics.line_spacing_gap(), 2.0);
}

#[test]
fn font_database_envelope_keeps_primary_spacing_when_fallback_supplies_glyphs() {
    let mut database = FontDatabase::default();
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
    let primary = database
        .register_font_file(assets.join("FiraSans-Regular.ttf"), Some("Primary"), 0)
        .expect("register primary font");
    let fallback = database
        .register_font_file(assets.join("FiraMono-subset.ttf"), Some("Fallback"), 0)
        .expect("register fallback font");
    let primary_metrics = database
        .face_metrics(primary)
        .expect("read primary metrics")
        .expect("primary metrics are tracked");
    let fallback_metrics = database
        .face_metrics(fallback)
        .expect("read fallback metrics")
        .expect("fallback metrics are tracked");
    let font_size = 20.0;
    let requested_line_height = 12.0;
    let (fallback_ascent, fallback_descent, _) =
        scaled_layout_extents(fallback_metrics, font_size).expect("valid fallback metrics");
    let (_, _, primary_gap) =
        scaled_layout_extents(primary_metrics, font_size).expect("valid primary metrics");

    let mut extents = SelectedFaceLineExtents::default();
    let _ = extents.include_face(&database, fallback, font_size);
    extents.include_primary_face(&database, primary, font_size);
    let envelope = extents
        .resolve_content_envelope(requested_line_height)
        .expect("fallback glyph metrics resolve an envelope");

    let expected_height =
        requested_line_height.max(fallback_ascent + fallback_descent + primary_gap);
    let expected_baseline =
        (expected_height - fallback_ascent - fallback_descent).max(0.0) * 0.5 + fallback_ascent;
    assert!((envelope.line_height - expected_height).abs() < 0.001);
    assert!((envelope.baseline_from_top - expected_baseline).abs() < 0.001);
}

#[test]
fn font_chain_height_envelope_includes_eligible_composite_faces_before_coverage() {
    let mut database = FontDatabase::default();
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
    let primary = database
        .register_font_file(assets.join("FiraSans-Regular.ttf"), Some("Primary"), 0)
        .expect("register primary font");
    let fallback = database
        .register_font_file(
            assets.join("FiraMono-subset.ttf"),
            Some("Composite Fallback"),
            0,
        )
        .expect("register composite fallback font");
    database.set_project_composite_font(Some(CompositeFontDescriptor {
        default_family: FontFamilyName::from("Primary"),
        sub_fonts: vec![SubFontRange {
            family: FontFamilyName::from("Composite Fallback"),
            scripts: vec![FontScript::Han],
            ranges: vec![(0x4E00, 0x9FFF)],
            cultures: Vec::new(),
        }],
    }));
    let style = TextStyle {
        font_family: Some("Primary".to_string()),
        font_size: 20.0,
        ..TextStyle::default()
    };
    let primary_metrics = database
        .face_metrics(primary)
        .expect("read primary metrics")
        .expect("primary metrics are tracked");
    let fallback_metrics = database
        .face_metrics(fallback)
        .expect("read fallback metrics")
        .expect("fallback metrics are tracked");
    let (primary_ascent, primary_descent, primary_gap) =
        scaled_layout_extents(primary_metrics, style.font_size).expect("primary extents");
    let (fallback_ascent, fallback_descent, _) =
        scaled_layout_extents(fallback_metrics, style.font_size).expect("fallback extents");

    let envelope = font_chain_line_metric_envelope(&database, &style)
        .expect("eligible chain must expose a metric envelope");

    let expected_minimum_height =
        primary_ascent.max(fallback_ascent) + primary_descent.max(fallback_descent) + primary_gap;
    assert!((envelope.minimum_line_height() - expected_minimum_height).abs() < 0.001);
    assert!(envelope.certifies_uniform_line_height(expected_minimum_height));
    assert!(!envelope.certifies_uniform_line_height(expected_minimum_height - 0.01));

    let report_after_first = database.fallback_cache_report();
    assert_eq!(report_after_first.line_metric_envelope_misses, 1);
    assert_eq!(report_after_first.line_metric_envelope_hits, 0);

    assert_eq!(
        font_chain_line_metric_envelope(&database, &style),
        Some(envelope)
    );
    let report_after_second = database.fallback_cache_report();
    assert_eq!(report_after_second.line_metric_envelope_misses, 1);
    assert_eq!(report_after_second.line_metric_envelope_hits, 1);
    assert_eq!(report_after_second.line_metric_envelope_entry_count, 1);
}
