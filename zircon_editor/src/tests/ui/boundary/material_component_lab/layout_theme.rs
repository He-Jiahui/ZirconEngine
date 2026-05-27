use std::{collections::BTreeSet, fs};

use toml::Value;
use zircon_runtime::ui::v2::UiZuiAssetLoader;
use zircon_runtime_interface::ui::v2::UiV2NodeDefinition;

use super::support::{assert_component, editor_asset, numeric_prop};

const MASONRY_THEME_SELECTORS: &[&str] = &[
    ".MuiMasonry-root",
    ".MuiMasonry-columnsConfigured",
    ".MuiMasonry-spacingConfigured",
    ".MuiMasonry-sequential",
    ".MuiMasonry-ssrDefaults",
    ".MuiMasonry-root.MuiMasonry-columnsConfigured.MuiMasonry-spacingConfigured.MuiMasonry-sequential.MuiMasonry-ssrDefaults",
    ".MuiMasonry-item",
    ".material-masonry-sample",
    ".material-masonry-tile",
];

#[test]
fn material_component_lab_masonry_sample_uses_runtime_descriptor_and_theme_selectors() {
    let path = editor_asset("assets/ui/editor/material_components/material_masonry.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()));
    let document = UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("{} should load as .zui: {error}", path.display()));
    let sample = document
        .nodes
        .get("sample")
        .unwrap_or_else(|| panic!("{} should define a sample node", path.display()));

    assert_component(&document, "sample", "Masonry");
    assert!(
        sample
            .classes
            .iter()
            .any(|class| class == "MuiMasonry-root"),
        "{} sample should expose the local MUI Lab root utility class",
        path.display()
    );
    assert_eq!(
        str_prop(sample, "className"),
        Some("material-masonry-sample")
    );
    assert_eq!(numeric_prop(sample.props.get("columns")), Some(4.0));
    assert_eq!(str_prop(sample, "spacing"), Some("1"));
    assert_eq!(bool_prop(sample, "sequential"), Some(true));
    assert_eq!(numeric_prop(sample.props.get("defaultColumns")), Some(4.0));
    assert_eq!(str_prop(sample, "defaultHeight"), Some("180"));
    assert_eq!(str_prop(sample, "defaultSpacing"), Some("1"));
    assert_eq!(sample.children.len(), 8);

    let layout = sample
        .layout
        .as_ref()
        .expect("Masonry sample should define layout");
    let container = layout
        .get("container")
        .and_then(Value::as_table)
        .expect("Masonry sample should define a Masonry container");
    assert_eq!(
        container.get("kind").and_then(Value::as_str),
        Some("Masonry")
    );
    assert_eq!(numeric_prop(container.get("columns")), Some(4.0));
    assert_eq!(numeric_prop(container.get("gap")), Some(8.0));
    assert_eq!(
        container.get("sequential").and_then(Value::as_bool),
        Some(true)
    );

    let theme_source =
        fs::read_to_string(editor_asset("assets/ui/theme/editor_material.v2.ui.toml"))
            .expect("Editor Material theme should be readable");
    let selectors = theme_selectors(&theme_source);
    for selector in MASONRY_THEME_SELECTORS {
        assert!(
            selectors.contains(*selector),
            "Editor Material theme should style Masonry selector `{selector}`"
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
