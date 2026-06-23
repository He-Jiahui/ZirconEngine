use super::*;

#[test]
fn mui_sx_merges_as_high_priority_style_override_and_state_selectors_match() {
    let style = UiAssetLoader::load_toml_str(MUI_WEB_STYLE_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(MUI_WEB_SX_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_style_import("asset://ui/tests/mui_web_style.ui", style)
        .unwrap();

    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;

    assert_eq!(str_attr(root, "text"), Some("SX Wins"));
    assert_eq!(str_attr(root, "text_tone"), Some("warning"));
    assert_eq!(str_attr(root, "validation_level"), Some("success"));
    assert_eq!(float_attr(root, "border_width"), Some(3.0));
    assert_eq!(float_attr(root, "corner_radius"), Some(6.0));
    assert_eq!(table_str_attr(root, "background", "color"), Some("#333333"));

    assert_eq!(
        root.style_overrides.get("text").and_then(Value::as_str),
        Some("SX Wins")
    );
    assert_eq!(
        root.style_overrides
            .get("background")
            .and_then(|background| background.get("color"))
            .and_then(Value::as_str),
        Some("#333333")
    );
    assert_eq!(
        root.style_overrides
            .get("border_width")
            .and_then(Value::as_float),
        Some(3.0)
    );

    assert_classes(
        root,
        &[
            "MuiButton-root",
            "MuiButton-contained",
            "MuiButton-colorPrimary",
            "MuiButton-sizeMedium",
            "Mui-hovered",
        ],
    );
}

#[test]
fn mui_state_classes_match_stylesheet_rules() {
    let style = UiAssetLoader::load_toml_str(MUI_WEB_STYLE_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(MUI_WEB_STATE_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_style_import("asset://ui/tests/mui_web_style.ui", style)
        .unwrap();

    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;

    assert_eq!(str_attr(root, "text"), Some("Disabled State"));
    assert_eq!(str_attr(root, "surface_variant"), Some("danger"));
    assert_classes(
        root,
        &[
            "MuiButton-root",
            "MuiButton-outlined",
            "MuiButton-colorSecondary",
            "MuiButton-sizeSmall",
            "Mui-disabled",
            "custom-mui-class",
        ],
    );
}

#[test]
fn mui_readonly_alias_generates_mui_state_class() {
    let style = UiAssetLoader::load_toml_str(MUI_WEB_STYLE_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(MUI_WEB_READONLY_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_style_import("asset://ui/tests/mui_web_style.ui", style)
        .unwrap();

    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;

    assert_eq!(bool_attr(root, "readOnly"), Some(true));
    assert_eq!(str_attr(root, "text_tone"), Some("muted"));
    assert_classes(root, &["MuiInputBase-root", "Mui-readOnly"]);
}

#[test]
fn mui_icon_utility_classes_match_local_mui_selectors() {
    let style = UiAssetLoader::load_toml_str(MUI_WEB_STYLE_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(MUI_WEB_ICON_UTILITY_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_style_import("asset://ui/tests/mui_web_style.ui", style)
        .unwrap();

    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;
    let icon = &root.children[0];
    let svg_icon = &root.children[1];

    assert_eq!(str_attr(icon, "text_tone"), Some("icon-primary-large"));
    assert_classes(
        icon,
        &[
            "MuiIcon-root",
            "MuiIcon-colorPrimary",
            "MuiIcon-fontSizeLarge",
        ],
    );
    assert_no_classes(icon, &["MuiIcon-sizeMedium", "MuiIcon-default"]);

    assert_eq!(str_attr(svg_icon, "text_tone"), Some("svg-secondary-large"));
    assert_classes(
        svg_icon,
        &[
            "MuiSvgIcon-root",
            "MuiSvgIcon-colorSecondary",
            "MuiSvgIcon-fontSizeLarge",
            "svg-icon-extra",
        ],
    );
    assert_no_classes(svg_icon, &["MuiSvgIcon-sizeMedium", "MuiSvgIcon-default"]);
}
