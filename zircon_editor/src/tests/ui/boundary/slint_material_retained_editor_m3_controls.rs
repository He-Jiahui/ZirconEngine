use std::{fs, path::PathBuf};

use toml::Value;
use zircon_runtime::ui::v2::UiZuiAssetLoader;

const MIGRATION_DOC: &str = "docs/ui-and-layout/slint-material-retained-editor-migration.md";
const THEME: &str = "zircon_editor/assets/ui/theme/editor_material.v2.ui.toml";
const BUTTONS_ZUI: &str = "zircon_editor/assets/ui/editor/material_components/material_buttons.zui";
const FAB_ZUI: &str =
    "zircon_editor/assets/ui/editor/material_components/material_floating_action_button.zui";

const M3_SOURCE_FILES: &[&str] = &[
    "dev/material-rust-template/material-1.0/ui/components/base_button.slint",
    "dev/material-rust-template/material-1.0/ui/components/filled_button.slint",
    "dev/material-rust-template/material-1.0/ui/components/outline_button.slint",
    "dev/material-rust-template/material-1.0/ui/components/text_button.slint",
    "dev/material-rust-template/material-1.0/ui/components/tonal_button.slint",
    "dev/material-rust-template/material-1.0/ui/components/elevated_button.slint",
    "dev/material-rust-template/material-1.0/ui/components/icon_button.slint",
    "dev/material-rust-template/material-1.0/ui/components/floating_action_button.slint",
];

const REQUIRED_M3_DOC_SYMBOLS: &[&str] = &[
    "button_horizontal_padding: MaterialStyleMetrics.padding_24",
    "button_vertical_padding: MaterialStyleMetrics.padding_10",
    "min_layout_width: MaterialStyleMetrics.size_40",
    "icon_size: MaterialStyleMetrics.icon_size_18",
    "hover when base.has_hover && !base.pressed && !base.enter_pressed",
    "root.inline ? MaterialStyleMetrics.icon_size_18 : MaterialStyleMetrics.icon_size_24",
    "clip_ripple: !root.inline",
    "export enum FABStyle",
    "level: 3",
    "root.style == FABStyle.small ? MaterialStyleMetrics.border_radius_12",
];

const EXPECTED_STRING_TOKENS: &[(&str, &str)] = &[
    (
        "slint_material_button_variants",
        "filled,outlined,text,tonal,elevated",
    ),
    ("slint_material_button_radius_policy", "height_half"),
    (
        "slint_material_button_hover_elevations",
        "filled=1,tonal=2,elevated=3",
    ),
    ("slint_material_icon_button_display_background", "false"),
    (
        "slint_material_icon_button_clip_ripple_contract",
        "clip_unless_inline",
    ),
    ("slint_material_fab_styles", "small,standard,large"),
];

const EXPECTED_NUMBER_TOKENS: &[(&str, f64)] = &[
    ("slint_material_base_button_horizontal_padding", 24.0),
    ("slint_material_base_button_vertical_padding", 10.0),
    ("slint_material_base_button_spacing", 8.0),
    ("slint_material_base_button_min_layout_width", 40.0),
    ("slint_material_base_button_min_layout_height", 40.0),
    ("slint_material_base_button_icon_size", 18.0),
    ("slint_material_icon_button_inline_icon_size", 18.0),
    ("slint_material_icon_button_default_icon_size", 24.0),
    ("slint_material_icon_button_inline_min_layout", 18.0),
    ("slint_material_icon_button_default_min_layout", 40.0),
    ("slint_material_fab_small_min_layout", 40.0),
    ("slint_material_fab_standard_min_layout", 56.0),
    ("slint_material_fab_large_min_layout", 96.0),
    ("slint_material_fab_small_radius", 12.0),
    ("slint_material_fab_standard_radius", 16.0),
    ("slint_material_fab_large_radius", 28.0),
    ("slint_material_fab_elevation_level", 3.0),
    ("slint_material_fab_hover_elevation_level", 4.0),
];

#[test]
fn m3_button_icon_fab_source_facts_are_documented_and_tokenized() {
    let migration_doc = workspace_file(MIGRATION_DOC);
    let theme = workspace_toml(THEME);
    let tokens = theme
        .get("tokens")
        .and_then(Value::as_table)
        .expect("editor material theme declares [tokens]");
    let mut failures = Vec::new();

    for source_file in M3_SOURCE_FILES {
        if !migration_doc.contains(source_file) {
            failures.push(format!("migration doc must reference `{source_file}`"));
        }
    }

    for source_symbol in REQUIRED_M3_DOC_SYMBOLS {
        if !migration_doc.contains(source_symbol) {
            failures.push(format!(
                "migration doc must preserve source behavior `{source_symbol}`"
            ));
        }
    }

    for (token, expected) in EXPECTED_STRING_TOKENS {
        match tokens.get(*token).and_then(Value::as_str) {
            Some(value) if value == *expected => {}
            Some(value) => failures.push(format!(
                "`{token}` must be `{expected}` from Slint M3 control behavior, got `{value}`"
            )),
            None => failures.push(format!("`{token}` must be a string token")),
        }
        if !migration_doc.contains(token) {
            failures.push(format!("M3 token `{token}` must be documented"));
        }
    }

    for (token, expected) in EXPECTED_NUMBER_TOKENS {
        match tokens.get(*token).and_then(value_as_number) {
            Some(value) if (value - expected).abs() < f64::EPSILON => {}
            Some(value) => failures.push(format!(
                "`{token}` must be {expected} from Slint M3 control behavior, got {value}"
            )),
            None => failures.push(format!("`{token}` must be a numeric token")),
        }
        if !migration_doc.contains(token) {
            failures.push(format!("M3 token `{token}` must be documented"));
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn material_button_and_fab_prototypes_carry_m3_source_derived_metadata() {
    let buttons = zui_document(BUTTONS_ZUI);
    let button_sample = buttons.nodes.get("sample").expect("button sample node");
    assert_eq!(
        buttons
            .nodes
            .get("meta_variant")
            .and_then(|node| node.props.get("text"))
            .and_then(Value::as_str),
        Some("filled / outlined / text / tonal / elevated")
    );
    assert_string_prop(button_sample, "slint_material_button_variant", "filled");
    assert_number_prop(button_sample, "button_horizontal_padding", 24.0);
    assert_number_prop(button_sample, "button_vertical_padding", 10.0);
    assert_number_prop(button_sample, "button_spacing", 8.0);
    assert_number_prop(button_sample, "min_layout_width", 40.0);
    assert_number_prop(button_sample, "min_layout_height", 40.0);
    assert_number_prop(button_sample, "icon_size", 18.0);
    assert_number_prop(button_sample, "hover_elevation", 1.0);
    assert_bool_prop(button_sample, "state_layer_enabled", true);
    assert_bool_prop(button_sample, "ripple_enabled", true);
    assert_bool_prop(button_sample, "clip_ripple", true);

    let fab = zui_document(FAB_ZUI);
    assert_eq!(
        fab.nodes
            .get("meta_variant")
            .and_then(|node| node.props.get("text"))
            .and_then(Value::as_str),
        Some("small / standard / large")
    );
    for (node_id, style, min_layout, radius, padding, icon_size) in [
        ("fab_circular", "standard", 56.0, 16.0, 14.0, 24.0),
        ("fab_small", "small", 40.0, 12.0, 10.0, 24.0),
        ("fab_extended", "large", 96.0, 28.0, 30.0, 36.0),
    ] {
        let node = fab
            .nodes
            .get(node_id)
            .unwrap_or_else(|| panic!("FAB prototype should define `{node_id}`"));
        assert_string_prop(node, "fab_style", style);
        assert_number_prop(node, "min_layout_width", min_layout);
        assert_number_prop(node, "min_layout_height", min_layout);
        assert_number_prop(node, "corner_radius", radius);
        assert_number_prop(node, "button_horizontal_padding", padding);
        assert_number_prop(node, "button_vertical_padding", padding);
        assert_number_prop(node, "icon_size", icon_size);
        assert_number_prop(node, "elevation", 3.0);
        assert_number_prop(node, "hover_elevation", 4.0);
        assert_bool_prop(node, "state_layer_enabled", true);
        assert_bool_prop(node, "ripple_enabled", true);
        assert_bool_prop(node, "clip_ripple", true);
    }
}

fn zui_document(relative: &str) -> zircon_runtime_interface::ui::v2::UiV2AssetDocument {
    let source = workspace_file(relative);
    UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("{relative} should load as .zui: {error}"))
}

fn assert_string_prop(
    node: &zircon_runtime_interface::ui::v2::UiV2NodeDefinition,
    prop: &str,
    expected: &str,
) {
    assert_eq!(
        node.props.get(prop).and_then(Value::as_str),
        Some(expected),
        "{} should set `{prop}`",
        node.control_id.as_deref().unwrap_or("<node>")
    );
}

fn assert_number_prop(
    node: &zircon_runtime_interface::ui::v2::UiV2NodeDefinition,
    prop: &str,
    expected: f64,
) {
    assert_eq!(
        node.props.get(prop).and_then(value_as_number),
        Some(expected),
        "{} should set `{prop}`",
        node.control_id.as_deref().unwrap_or("<node>")
    );
}

fn assert_bool_prop(
    node: &zircon_runtime_interface::ui::v2::UiV2NodeDefinition,
    prop: &str,
    expected: bool,
) {
    assert_eq!(
        node.props.get(prop).and_then(Value::as_bool),
        Some(expected),
        "{} should set `{prop}`",
        node.control_id.as_deref().unwrap_or("<node>")
    );
}

fn workspace_toml(relative: &str) -> Value {
    toml::from_str(&workspace_file(relative))
        .unwrap_or_else(|error| panic!("{relative} parses: {error}"))
}

fn workspace_file(relative: &str) -> String {
    fs::read_to_string(workspace_root().join(relative))
        .unwrap_or_else(|error| panic!("{relative} should be readable: {error}"))
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_editor lives directly under the workspace root")
        .to_path_buf()
}

fn value_as_number(value: &Value) -> Option<f64> {
    value
        .as_float()
        .or_else(|| value.as_integer().map(|integer| integer as f64))
}
