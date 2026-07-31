use crate::ui::style::{
    UiRgbaColor, UiThemeControlSizes, UiThemeDocument, UiThemeShape, UiThemeTokenRef,
};

#[test]
fn ui_theme_document_defaults_cover_editor_dark_theme_tokens() {
    let theme = UiThemeDocument::dark();

    assert_eq!(theme.id, "zircon.dark");
    assert_eq!(theme.palette.surface.len(), 4);
    assert_eq!(
        theme.palette.accent,
        UiRgbaColor::from_u8(60, 199, 214, 255)
    );
    assert_eq!(theme.shape, UiThemeShape::default());
    assert_eq!(theme.control_sizes, UiThemeControlSizes::default());
    assert!(
        theme
            .typography
            .iter()
            .any(|variant| variant.variant == "body" && variant.family == "Inter")
    );
}

#[test]
fn ui_theme_document_round_trips_with_sparse_defaults() {
    let theme: UiThemeDocument = serde_json::from_str(
        r#"{
            "id":"custom.dark",
            "palette":{"accent":{"red":0.1,"green":0.2,"blue":0.3,"alpha":1.0}},
            "control_sizes":{"compact_height":30.0}
        }"#,
    )
    .unwrap();

    assert_eq!(theme.id, "custom.dark");
    assert_eq!(theme.palette.accent, UiRgbaColor::new(0.1, 0.2, 0.3, 1.0));
    assert_eq!(
        theme.palette.surface[0],
        UiRgbaColor::from_u8(17, 20, 22, 255)
    );
    assert_eq!(theme.control_sizes.compact_height, 30.0);
    assert_eq!(theme.control_sizes.default_height, 40.0);

    let round_trip: UiThemeDocument =
        serde_json::from_str(&serde_json::to_string(&theme).unwrap()).unwrap();
    assert_eq!(round_trip, theme);
}

#[test]
fn ui_theme_token_ref_is_stable_string_contract() {
    let token = UiThemeTokenRef::new("palette.surface.1");

    assert_eq!(token.as_str(), "palette.surface.1");
    assert_eq!(
        serde_json::to_string(&token).unwrap(),
        r#""palette.surface.1""#
    );
}
