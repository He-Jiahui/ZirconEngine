use super::*;
use crate::ui::retained_host::host_contract::paint_theme::{
    HostTextSmoothing, HostUtilityTabTextRole,
};

#[test]
fn ui_face_keeps_proportional_widths_while_code_face_stays_mono() {
    let px = 13.0;
    let ui_font = font_for_face(HostTextFontFace::Ui).expect("ui retained-host font");
    let mono_font = font_for_face(HostTextFontFace::Mono).expect("mono retained-host font");
    let ui_i = ui_font.metrics('i', px).advance_width;
    let ui_w = ui_font.metrics('W', px).advance_width;
    let mono_i = mono_font.metrics('i', px).advance_width;
    let mono_w = mono_font.metrics('W', px).advance_width;

    assert!(
        ui_w > ui_i * 1.5,
        "UI face must resolve to a proportional editor font role"
    );
    assert!(
        (mono_w - mono_i).abs() < 0.25,
        "code face must resolve to a fixed-width editor font role"
    );
}

#[test]
fn font_request_for_face_uses_text_preferences_without_platform_paths() {
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
fn runtime_font_family_is_resolved_from_font_preferences() {
    let ui_family = runtime_font_family_for_face(HostTextFontFace::Ui);
    let mono_family = runtime_font_family_for_face(HostTextFontFace::Mono);

    assert!(!ui_family.trim().is_empty());
    assert!(!mono_family.trim().is_empty());
    assert!(!ui_family.contains('\\'));
    assert!(!mono_family.contains('\\'));
}

#[test]
fn embedded_fallback_keeps_the_requested_runtime_family() {
    let request = HostTextFontRequest {
        face: HostTextFontFace::Ui,
        family: "requested-ui-family".to_string(),
        weight: 400,
    };

    let font = embedded_font_for_request(&request);

    assert_eq!(font.runtime_family, "requested-ui-family");
    assert!(font.font.is_some());
}

#[test]
fn unavailable_host_font_preserves_runtime_family_without_embedded_panic() {
    let request = HostTextFontRequest {
        face: HostTextFontFace::Ui,
        family: "requested-ui-family".to_string(),
        weight: 400,
    };

    let font = unavailable_host_font(&request);

    assert!(font.font.is_none());
    assert!(font.bytes.is_empty());
    assert_eq!(font.runtime_family, "requested-ui-family");
}

#[test]
fn runtime_text_style_for_face_projects_the_same_font_request() {
    let style = runtime_text_style_for_face(
        HostTextFontFace::UiStrong,
        11.0,
        14.0,
        UiTextWrap::None,
        UiTextOverflow::Ellipsis,
    );
    let request = font_request_for_face(HostTextFontFace::UiStrong);

    assert_eq!(style.font_family.as_deref(), Some(request.family.as_str()));
    assert_eq!(style.font_weight, request.weight);
    assert_eq!(style.font_size, 11.0);
    assert_eq!(style.line_height, 14.0);
    assert_eq!(style.wrap, UiTextWrap::None);
    assert_eq!(style.text_overflow, UiTextOverflow::Ellipsis);
}
