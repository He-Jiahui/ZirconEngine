use super::*;

#[test]
fn mui_slot_props_apply_to_root_and_named_slot_children() {
    let style = UiAssetLoader::load_toml_str(MUI_WEB_STYLE_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(MUI_WEB_SLOT_PROPS_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_style_import("asset://ui/tests/mui_web_style.ui", style)
        .unwrap();

    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;
    let child = root.children.first().expect("start icon child");

    assert_eq!(bool_attr(root, "disabled"), Some(true));
    assert_classes(root, &["MuiButton-root", "Mui-disabled"]);

    assert_eq!(
        child
            .slot_attributes
            .get("mui_slot")
            .and_then(Value::as_str),
        Some("startIcon")
    );
    assert_eq!(str_attr(child, "text"), Some("Slot Prop"));
    assert_eq!(str_attr(child, "text_tone"), Some("info"));
    assert_eq!(str_attr(child, "surface_variant"), Some("success"));
    assert_eq!(str_attr(child, "mui_slot_component"), Some("IconButton"));
    assert_classes(
        child,
        &["MuiLabel-root", "MuiButton-startIcon", "slot-extra"],
    );
}

#[test]
fn mui_native_customization_aliases_match_web_prop_names() {
    let style = UiAssetLoader::load_toml_str(MUI_WEB_STYLE_TOML).unwrap();
    let layout =
        UiAssetLoader::load_toml_str(MUI_WEB_NATIVE_CUSTOMIZATION_ALIAS_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_style_import("asset://ui/tests/mui_web_style.ui", style)
        .unwrap();

    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;
    let child = root.children.first().expect("start icon child");

    assert_eq!(str_attr(root, "text"), Some("SX Alias Wins"));
    assert_eq!(float_attr(root, "border_width"), Some(4.0));
    assert_eq!(bool_attr(root, "disabled"), Some(true));
    assert_eq!(table_str_attr(root, "background", "color"), Some("#444444"));
    assert_classes(
        root,
        &[
            "MuiButton-root",
            "MuiButton-contained",
            "MuiButton-colorSecondary",
            "MuiButton-sizeSmall",
            "Mui-disabled",
            "root-extra",
            "root-alias",
            "classes-root",
        ],
    );

    assert_eq!(str_attr(child, "text"), Some("Plain Slot"));
    assert_eq!(str_attr(child, "text_tone"), Some("info"));
    assert_eq!(str_attr(child, "surface_variant"), Some("success"));
    assert_eq!(str_attr(child, "mui_slot_component"), Some("IconButton"));
    assert_classes(
        child,
        &[
            "MuiLabel-root",
            "MuiButton-startIcon",
            "slot-extra",
            "slot-class",
            "classes-start",
        ],
    );
}
