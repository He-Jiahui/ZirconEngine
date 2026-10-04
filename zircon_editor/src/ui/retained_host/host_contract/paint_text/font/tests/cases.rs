use super::*;
use crate::ui::retained_host::host_contract::paint_theme::{
    HostTextSmoothing, HostUtilityTabTextRole,
};

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
fn runtime_font_family_is_projected_from_preferences_without_backend_resolution() {
    let ui_family = runtime_font_family_for_face(HostTextFontFace::Ui);
    let mono_family = runtime_font_family_for_face(HostTextFontFace::Mono);

    assert!(!ui_family.trim().is_empty());
    assert!(!mono_family.trim().is_empty());
    assert!(!ui_family.contains('\\'));
    assert!(!mono_family.contains('\\'));
}

#[test]
fn runtime_text_style_for_face_projects_the_canonical_runtime_request() {
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

#[test]
fn runtime_font_request_generation_includes_face_role_and_runtime_generation() {
    assert_ne!(
        runtime_font_request_generation(HostTextFontFace::Ui),
        runtime_font_request_generation(HostTextFontFace::Mono)
    );
    assert_eq!(
        runtime_text_metrics_generation(),
        [
            runtime_font_request_generation(HostTextFontFace::Ui),
            runtime_font_request_generation(HostTextFontFace::UiStrong),
            runtime_font_request_generation(HostTextFontFace::Mono),
        ]
    );
}
