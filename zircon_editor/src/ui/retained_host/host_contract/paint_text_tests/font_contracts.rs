use std::sync::Arc;

use zircon_runtime_interface::ui::{
    design_tokens::EditorTypographyTokens, surface::UiTextRunPaintStyle,
};

use super::super::draw::{DEFAULT_FONT_SIZE, DEFAULT_LINE_HEIGHT};
use super::super::font::{
    font_face_for_paint_style, font_request_for_face_with_preferences,
    runtime_font_family_for_face, HostTextFontFace,
};
use super::super::raster::rasterize_cached_glyph;
use crate::ui::retained_host::host_contract::paint_theme::{
    HostTextPreferences, HostTextSmoothing, HostUtilityTabTextRole,
};

#[test]
fn retained_text_default_metrics_project_workbench_typography_tokens() {
    assert_eq!(
        DEFAULT_FONT_SIZE,
        EditorTypographyTokens::WORKBENCH_BODY_SIZE
    );
    assert_eq!(
        DEFAULT_LINE_HEIGHT,
        EditorTypographyTokens::WORKBENCH_BODY_SIZE
            * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO
    );
}

#[test]
fn glyph_raster_cache_reuses_bitmap_for_same_glyph_and_size() {
    let first = rasterize_cached_glyph(HostTextFontFace::Ui, 1, DEFAULT_FONT_SIZE, 3.0, 0.0);
    let second = rasterize_cached_glyph(HostTextFontFace::Ui, 1, DEFAULT_FONT_SIZE, 3.0, 0.0);

    assert_eq!(first.metrics.width, second.metrics.width);
    assert!(Arc::ptr_eq(&first.bitmap, &second.bitmap));
}

#[test]
fn retained_text_font_face_tracks_ui_and_code_styles() {
    assert_eq!(
        font_face_for_paint_style(UiTextRunPaintStyle::default()),
        HostTextFontFace::Ui
    );
    assert_eq!(
        font_face_for_paint_style(UiTextRunPaintStyle {
            strong: true,
            ..UiTextRunPaintStyle::default()
        }),
        HostTextFontFace::UiStrong
    );
    assert_eq!(
        font_face_for_paint_style(UiTextRunPaintStyle {
            code: true,
            strong: true,
            ..UiTextRunPaintStyle::default()
        }),
        HostTextFontFace::Mono
    );
}

#[test]
fn retained_ui_runtime_family_resolves_from_preferences_without_platform_paths() {
    let ui_family = runtime_font_family_for_face(HostTextFontFace::Ui);
    let mono_family = runtime_font_family_for_face(HostTextFontFace::Mono);

    assert!(!ui_family.trim().is_empty());
    assert!(!mono_family.trim().is_empty());
    assert!(!ui_family.contains('\\'));
    assert!(!mono_family.contains('\\'));
    assert_ne!(
        ui_family, mono_family,
        "ordinary UI text and code text must not collapse to one runtime family"
    );
}

#[test]
fn retained_text_font_request_uses_global_preferences() {
    let preferences = HostTextPreferences {
        ui_family: "ui-family".to_string(),
        ui_strong_family: "ui-strong-family".to_string(),
        code_family: "code-family".to_string(),
        utility_tab_text_role: HostUtilityTabTextRole::Ui,
        ui_weight: 410,
        strong_weight: 620,
        code_weight: 430,
        smoothing: HostTextSmoothing::Grayscale,
    };

    let ui = font_request_for_face_with_preferences(HostTextFontFace::Ui, &preferences);
    let strong = font_request_for_face_with_preferences(HostTextFontFace::UiStrong, &preferences);
    let mono = font_request_for_face_with_preferences(HostTextFontFace::Mono, &preferences);

    assert_eq!(ui.family, "ui-family");
    assert_eq!(ui.weight, 410);
    assert_eq!(strong.family, "ui-strong-family");
    assert_eq!(strong.weight, 620);
    assert_eq!(mono.family, "code-family");
    assert_eq!(mono.weight, 430);
}

#[test]
fn glyph_raster_cache_keeps_ui_and_mono_faces_separate() {
    let ui = rasterize_cached_glyph(HostTextFontFace::Ui, 1, DEFAULT_FONT_SIZE, 3.0, 0.0);
    let mono = rasterize_cached_glyph(HostTextFontFace::Mono, 1, DEFAULT_FONT_SIZE, 3.0, 0.0);

    assert!(!Arc::ptr_eq(&ui.bitmap, &mono.bitmap));
}
