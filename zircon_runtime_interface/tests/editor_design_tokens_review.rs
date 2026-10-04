//! 对照工作区主题 fixture 与 `EditorDesignTokens::workbench_dark`，核验 token 值、级联、覆盖权限和 serde 默认值的一致性。
use std::path::PathBuf;

use toml::Value;
use zircon_runtime_interface::ui::design_tokens::{
    EditorChromeTokens, EditorControlTokens, EditorDensityTokens, EditorDesignTokens,
    EditorTypographyTokens,
};

fn editor_theme() -> Value {
    let repo = std::env::var_os("ZUI_LAYOUT_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".."));
    let path = repo.join("zircon_editor/assets/ui/editor/theme/editor_tokens.zui");
    toml::from_str(&std::fs::read_to_string(path).expect("read product Editor theme"))
        .expect("parse product Editor theme")
}

#[test]
fn default_editor_settings_match_the_zui_theme_contract() {
    let theme = editor_theme();
    let tokens = EditorDesignTokens::workbench_dark();
    let typography: EditorTypographyTokens = theme["typography"].clone().try_into().unwrap();
    let controls: EditorControlTokens = theme["controls"].clone().try_into().unwrap();
    let density: EditorDensityTokens = theme["density"].clone().try_into().unwrap();
    let chrome: EditorChromeTokens = theme["chrome"].clone().try_into().unwrap();
    assert_eq!(tokens.typography, typography);
    assert_eq!(tokens.controls, controls);
    assert_eq!(tokens.density, density);
    assert_eq!(tokens.chrome, chrome);
    let cascade = tokens.cascade_token_values();
    for (state, role) in theme["state_roles"].as_table().unwrap() {
        let name = theme["names"]["state_roles"][state].as_str().unwrap();
        let palette = theme["names"]["palette"][role.as_str().unwrap()]
            .as_str()
            .unwrap();
        assert_eq!(
            cascade[name],
            Value::String(format!("${palette}")),
            "{state}"
        );
    }
}

#[test]
fn default_editor_palette_matches_every_named_zui_color() {
    let theme = editor_theme();
    let cascade = EditorDesignTokens::workbench_dark().cascade_token_values();
    for (field, name) in theme["names"]["palette"].as_table().unwrap() {
        let color = field
            .strip_prefix("surface_")
            .and_then(|suffix| suffix.parse::<usize>().ok())
            .map(|index| &theme["palette"]["surface"][index])
            .unwrap_or_else(|| &theme["palette"][field]);
        assert_eq!(
            cascade.get(name.as_str().unwrap()),
            Some(color),
            "palette.{field}"
        );
    }
}

#[test]
fn serialized_editor_overrides_remain_authoritative() {
    let mut custom = EditorDesignTokens::workbench_dark();
    custom.typography.body_size = 16.0;
    custom.typography.ui_family = "Project UI".into();
    custom.typography.heading_size = 18.0;
    custom.density.gap_group = 28.0;
    custom.controls.control_radius = 2.0;
    let serialized = serde_json::to_vec(&custom).unwrap();
    let restored: EditorDesignTokens = serde_json::from_slice(&serialized).unwrap();
    assert_eq!(restored, custom);
    assert_eq!(
        restored.cascade_token_values()["editor.typography.body.size"],
        Value::Float(16.0)
    );
    assert_eq!(
        restored.cascade_token_values()["editor.typography.heading.size"],
        Value::Float(18.0)
    );
    assert_eq!(
        restored.cascade_token_values()["editor.density.gap.group"],
        Value::Float(28.0)
    );
}

#[test]
fn heading_and_group_spacing_reach_named_tokens_and_theme_consumers() {
    let tokens = EditorDesignTokens::workbench_dark();
    let theme = editor_theme();
    let cascade = tokens.cascade_token_values();
    for (section, field, expected) in [
        ("typography", "heading_size", 16.0),
        ("density", "gap_group", 24.0),
    ] {
        let name = theme["names"][section][field].as_str().unwrap();
        assert_eq!(cascade[name], Value::Float(expected));
        assert_eq!(theme[section][field], Value::Float(expected));
        assert_eq!(
            cascade[&format!("--{}", name.replace('.', "-"))],
            Value::String(format!("${name}"))
        );
    }
    let resolved = tokens.to_theme_document();
    let heading = resolved
        .typography
        .iter()
        .find(|item| item.variant == "heading")
        .unwrap();
    assert_eq!(heading.size, 16.0);
    assert_eq!(heading.weight, tokens.typography.strong_weight);
    assert!(resolved.spacing.contains(&24.0));
}

#[test]
fn existing_theme_payloads_default_heading_and_group_spacing() {
    let mut value = serde_json::to_value(EditorDesignTokens::workbench_dark()).unwrap();
    value["typography"]
        .as_object_mut()
        .unwrap()
        .remove("heading_size");
    value["density"]
        .as_object_mut()
        .unwrap()
        .remove("gap_group");
    let restored: EditorDesignTokens = serde_json::from_value(value).unwrap();
    assert_eq!(restored.typography.heading_size, 16.0);
    assert_eq!(restored.density.gap_group, 24.0);
}
