use std::{collections::BTreeSet, fs};

use toml::Value;
use zircon_runtime::ui::v2::UiZuiAssetLoader;
use zircon_runtime_interface::ui::v2::{UiV2AssetDocument, UiV2ChildMount, UiV2NodeDefinition};

use super::support::{assert_component, assert_node_class, editor_asset, numeric_prop};

const PAPER_THEME_SELECTORS: &[&str] = &[
    ".MuiPaper-root",
    ".MuiPaper-rounded",
    ".MuiPaper-outlined",
    ".MuiPaper-elevation",
    ".MuiPaper-elevation0",
    ".MuiPaper-elevation1",
    ".MuiPaper-elevation3",
    ".MuiPaper-root.MuiPaper-elevation.MuiPaper-rounded.MuiPaper-elevation3",
    ".material-paper-sample",
    ".material-paper-content",
];

const ALERT_THEME_SELECTORS: &[&str] = &[
    ".MuiAlert-root",
    ".MuiAlert-standard",
    ".MuiAlert-filled",
    ".MuiAlert-outlined",
    ".MuiAlert-colorSuccess",
    ".MuiAlert-colorInfo",
    ".MuiAlert-colorWarning",
    ".MuiAlert-colorError",
    ".MuiAlert-root.MuiAlert-outlined.MuiAlert-colorWarning",
    ".MuiAlert-icon",
    ".MuiAlert-message",
    ".MuiAlert-action",
    ".material-alert-sample",
    ".material-alert-icon",
    ".material-alert-message",
    ".material-alert-action",
];

#[test]
fn material_component_lab_paper_sample_uses_runtime_descriptor_and_theme_selectors() {
    let document = load_zui("assets/ui/editor/material_components/material_paper.zui");
    let sample = node(&document, "sample");

    assert_component(&document, "sample", "Paper");
    assert_node_class(&document, "sample", "MuiPaper-root");
    assert_eq!(str_prop(sample, "className"), Some("material-paper-sample"));
    assert_eq!(str_prop(sample, "component"), Some("div"));
    assert_eq!(str_prop(sample, "variant"), Some("elevation"));
    assert_eq!(numeric_prop(sample.props.get("elevation")), Some(3.0));
    assert_eq!(bool_prop(sample, "square"), Some(false));
    assert_eq!(
        slot_class_name(sample, "content"),
        Some("material-paper-content")
    );
    assert_static_visual(sample, "Paper");
    assert_child_slot(&document, "sample", "paper_content", "content");
    assert_component(&document, "paper_content", "Label");
    assert_node_class(&document, "paper_content", "material-paper-content");

    assert_theme_selectors("Paper", PAPER_THEME_SELECTORS);
}

#[test]
fn material_component_lab_alert_sample_uses_runtime_descriptor_and_theme_selectors() {
    let document = load_zui("assets/ui/editor/material_components/material_alert.zui");
    let sample = node(&document, "sample");

    assert_component(&document, "sample", "Alert");
    assert_node_class(&document, "sample", "MuiAlert-root");
    assert_eq!(str_prop(sample, "className"), Some("material-alert-sample"));
    assert_eq!(str_prop(sample, "text"), Some("Warning"));
    assert_eq!(str_prop(sample, "severity"), Some("warning"));
    assert_eq!(str_prop(sample, "variant"), Some("outlined"));
    assert_eq!(str_prop(sample, "color"), Some("warning"));
    assert_eq!(str_prop(sample, "icon"), Some("warning"));
    assert_eq!(bool_prop(sample, "show_icon"), Some(true));
    assert_eq!(str_prop(sample, "closeText"), Some("Dismiss"));
    assert_eq!(slot_class_name(sample, "icon"), Some("material-alert-icon"));
    assert_eq!(
        slot_class_name(sample, "message"),
        Some("material-alert-message")
    );
    assert_eq!(
        slot_class_name(sample, "action"),
        Some("material-alert-action")
    );
    assert_static_visual(sample, "Alert");

    for (child, slot, component, class_name) in [
        ("alert_icon", "icon", "Icon", "MuiAlert-icon"),
        ("alert_message", "message", "Label", "MuiAlert-message"),
        ("alert_action", "action", "Button", "MuiAlert-action"),
    ] {
        assert_child_slot(&document, "sample", child, slot);
        assert_component(&document, child, component);
        assert_node_class(&document, child, class_name);
    }

    assert_theme_selectors("Alert", ALERT_THEME_SELECTORS);
}

fn load_zui(relative: &str) -> UiV2AssetDocument {
    let path = editor_asset(relative);
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()))
}

fn node<'a>(document: &'a UiV2AssetDocument, node_id: &str) -> &'a UiV2NodeDefinition {
    document
        .nodes
        .get(node_id)
        .unwrap_or_else(|| panic!("Material Lab should contain node `{node_id}`"))
}

fn assert_child_slot(
    document: &UiV2AssetDocument,
    owner_id: &str,
    child_id: &str,
    expected_slot: &str,
) {
    let mount = node(document, owner_id)
        .children
        .iter()
        .find(|mount| mount.node == child_id)
        .unwrap_or_else(|| panic!("`{owner_id}` should mount child `{child_id}`"));
    assert_eq!(
        slot_name(mount),
        Some(expected_slot),
        "`{owner_id}` should mount `{child_id}` into `{expected_slot}`"
    );
}

fn assert_static_visual(node: &UiV2NodeDefinition, label: &str) {
    for prop in [
        "input_interactive",
        "input_clickable",
        "input_hoverable",
        "input_focusable",
    ] {
        assert_eq!(
            bool_prop(node, prop),
            Some(false),
            "{label} visual sample should keep `{prop}` disabled"
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

fn slot_name(mount: &UiV2ChildMount) -> Option<&str> {
    mount.slot.get("name").and_then(Value::as_str)
}

fn str_prop<'a>(node: &'a UiV2NodeDefinition, name: &str) -> Option<&'a str> {
    node.props.get(name).and_then(Value::as_str)
}

fn bool_prop(node: &UiV2NodeDefinition, name: &str) -> Option<bool> {
    node.props.get(name).and_then(Value::as_bool)
}

fn slot_class_name<'a>(node: &'a UiV2NodeDefinition, slot: &str) -> Option<&'a str> {
    node.props
        .get("slotProps")
        .and_then(Value::as_table)
        .and_then(|slot_props| slot_props.get(slot))
        .and_then(Value::as_table)
        .and_then(|props| props.get("className"))
        .and_then(Value::as_str)
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
