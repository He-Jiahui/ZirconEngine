use std::{collections::BTreeSet, fs};

use toml::Value;
use zircon_runtime::ui::v2::UiZuiAssetLoader;
use zircon_runtime_interface::ui::v2::UiV2NodeDefinition;

use super::support::{assert_component, assert_node_class, editor_asset};

const ICON_THEME_SELECTORS: &[&str] = &[
    ".MuiIcon-root",
    ".MuiIcon-colorPrimary",
    ".MuiIcon-colorSecondary",
    ".MuiIcon-colorAction",
    ".MuiIcon-colorError",
    ".MuiIcon-colorDisabled",
    ".MuiIcon-fontSizeInherit",
    ".MuiIcon-fontSizeSmall",
    ".MuiIcon-fontSizeMedium",
    ".MuiIcon-fontSizeLarge",
    ".MuiIcon-root.MuiIcon-colorPrimary.MuiIcon-fontSizeLarge",
    ".material-icon-sample",
];

const SVG_ICON_THEME_SELECTORS: &[&str] = &[
    ".MuiSvgIcon-root",
    ".MuiSvgIcon-colorPrimary",
    ".MuiSvgIcon-colorSecondary",
    ".MuiSvgIcon-colorAction",
    ".MuiSvgIcon-colorError",
    ".MuiSvgIcon-colorDisabled",
    ".MuiSvgIcon-fontSizeInherit",
    ".MuiSvgIcon-fontSizeSmall",
    ".MuiSvgIcon-fontSizeMedium",
    ".MuiSvgIcon-fontSizeLarge",
    ".MuiSvgIcon-root.MuiSvgIcon-colorSecondary.MuiSvgIcon-fontSizeLarge",
    ".material-material-icon-sample",
];

#[test]
fn material_component_lab_icon_sample_uses_runtime_descriptor_and_theme_selectors() {
    let path = editor_asset("assets/ui/editor/material_components/material_icons.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    let document = UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
    let sample = document
        .nodes
        .get("sample")
        .unwrap_or_else(|| panic!("{} should define a sample node", path.display()));

    assert_component(&document, "sample", "Icon");
    assert_node_class(&document, "sample", "MuiIcon-root");
    assert_eq!(str_prop(sample, "className"), Some("material-icon-sample"));
    assert_eq!(str_prop(sample, "baseClassName"), Some("material-icons"));
    assert_eq!(str_prop(sample, "component"), Some("span"));
    assert_eq!(str_prop(sample, "text"), Some("folder"));
    assert_eq!(str_prop(sample, "icon"), Some("folder"));
    assert_eq!(str_prop(sample, "color"), Some("primary"));
    assert_eq!(str_prop(sample, "fontSize"), Some("large"));
    assert_static_visual(sample);

    assert_theme_selectors("Icon", ICON_THEME_SELECTORS);
}

#[test]
fn material_component_lab_material_icon_sample_uses_svg_icon_descriptor_and_theme_selectors() {
    let path = editor_asset("assets/ui/editor/material_components/material_material_icons.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    let document = UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
    let sample = document
        .nodes
        .get("sample")
        .unwrap_or_else(|| panic!("{} should define a sample node", path.display()));

    assert_component(&document, "sample", "SvgIcon");
    assert_node_class(&document, "sample", "MuiSvgIcon-root");
    assert_eq!(
        str_prop(sample, "className"),
        Some("material-material-icon-sample")
    );
    assert_eq!(str_prop(sample, "component"), Some("svg"));
    assert_eq!(str_prop(sample, "text"), Some("AddCircle"));
    assert_eq!(str_prop(sample, "icon"), Some("AddCircle"));
    assert_eq!(str_prop(sample, "color"), Some("secondary"));
    assert_eq!(str_prop(sample, "fontSize"), Some("large"));
    assert_eq!(str_prop(sample, "htmlColor"), Some("#35c7d0"));
    assert_eq!(str_prop(sample, "viewBox"), Some("0 0 24 24"));
    assert_eq!(str_prop(sample, "titleAccess"), Some("Add circle"));
    assert_eq!(bool_prop(sample, "inheritViewBox"), Some(false));
    assert_static_visual(sample);

    assert_theme_selectors("SvgIcon", SVG_ICON_THEME_SELECTORS);
}

fn assert_static_visual(node: &UiV2NodeDefinition) {
    for prop in [
        "input_interactive",
        "input_clickable",
        "input_hoverable",
        "input_focusable",
    ] {
        assert_eq!(
            bool_prop(node, prop),
            Some(false),
            "Icon visual sample should keep `{prop}` disabled"
        );
    }
}

fn assert_theme_selectors(label: &str, expected_selectors: &[&str]) {
    let theme_source =
        fs::read_to_string(editor_asset("assets/ui/theme/editor_material.v2.ui.toml"))
            .expect("Editor Material theme should be readable");
    let selectors = theme_selectors(&theme_source);
    for selector in expected_selectors {
        assert!(
            selectors.contains(*selector),
            "Editor Material theme should style {label} selector `{selector}`"
        );
    }
}

fn str_prop<'a>(node: &'a UiV2NodeDefinition, name: &str) -> Option<&'a str> {
    node.props.get(name).and_then(Value::as_str)
}

fn bool_prop(node: &UiV2NodeDefinition, name: &str) -> Option<bool> {
    node.props.get(name).and_then(Value::as_bool)
}

fn theme_selectors(source: &str) -> BTreeSet<String> {
    toml::from_str::<Value>(source)
        .expect("Editor Material theme should parse as TOML")
        .get("stylesheets")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .flat_map(|stylesheet| {
            stylesheet
                .get("rules")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter_map(|rule| rule.get("selector").and_then(Value::as_str))
        .map(ToOwned::to_owned)
        .collect()
}
