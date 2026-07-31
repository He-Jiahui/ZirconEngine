use super::super::*;
use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;

#[test]
fn ui_v2_style_specificity_and_pseudo_state_are_resolved() {
    let mut document = v2_document("asset://ui/tests/style.v2.ui", "root");
    document.tokens.insert(
        "material.primary".to_string(),
        Value::String("#6750a4".to_string()),
    );
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Button".to_string(),
            control_id: Some("RunButton".to_string()),
            classes: vec!["primary".to_string()],
            state: BTreeMap::from([("hovered".to_string(), Value::Boolean(true))]),
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "editor_material".to_string(),
        rules: vec![
            style_rule("Button", [("fg", "#111111")]),
            style_rule(".primary", [("fg", "#222222")]),
            style_rule("Button.primary:hover", [("fg", "$material.primary")]),
            style_rule("#RunButton", [("radius", "6")]),
        ],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let resolved = UiV2StyleResolver::resolve(&document, &compiled.arena).unwrap();
    let root = resolved.nodes.get("root").unwrap();

    assert_eq!(root.self_values["fg"].as_str(), Some("#6750a4"));
    assert_eq!(root.self_values["radius"].as_str(), Some("6"));
}

#[test]
fn ui_v2_style_id_specificity_exceeds_any_number_of_class_selectors() {
    let mut document = v2_document("asset://ui/tests/style_id_specificity.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Button".to_string(),
            control_id: Some("RunButton".to_string()),
            classes: vec![
                "one".to_string(),
                "two".to_string(),
                "three".to_string(),
                "four".to_string(),
                "five".to_string(),
                "six".to_string(),
                "seven".to_string(),
                "eight".to_string(),
                "nine".to_string(),
                "ten".to_string(),
            ],
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "specificity".to_string(),
        rules: vec![
            style_rule("#RunButton", [("foreground", "#ffffff")]),
            style_rule(
                ".one.two.three.four.five.six.seven.eight.nine.ten",
                [("foreground", "#ff0000")],
            ),
        ],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let resolved = UiV2StyleResolver::resolve(&document, &compiled.arena).unwrap();

    assert_eq!(
        resolved.nodes["root"].self_values["foreground"].as_str(),
        Some("#ffffff")
    );
}

#[test]
fn ui_v2_style_resolver_registers_editor_tokens_for_css_custom_properties() {
    let mut document = v2_document("asset://ui/tests/editor_token_style.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Button".to_string(),
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "editor_tokens".to_string(),
        rules: vec![style_rule(
            "Button",
            [
                ("background", "var(--editor-surface-1)"),
                ("focused_background", "var(--editor-state-focused)"),
            ],
        )],
    });
    document.tokens.insert(
        "editor.obsolete".to_string(),
        Value::String("#ff00ff".to_string()),
    );
    document.tokens.insert(
        "--editor-obsolete".to_string(),
        Value::String("$editor.obsolete".to_string()),
    );
    UiV2StyleResolver::register_editor_design_tokens(
        &mut document,
        &EditorDesignTokens::workbench_dark(),
    );
    assert!(!document.tokens.contains_key("editor.obsolete"));
    assert!(!document.tokens.contains_key("--editor-obsolete"));

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let resolved = UiV2StyleResolver::resolve(&document, &compiled.arena).unwrap();
    let root = &resolved.nodes["root"];

    assert_eq!(root.self_values["background"].as_str(), Some("#171a1d"));
    assert_eq!(
        root.self_values["focused_background"].as_str(),
        Some("#173942")
    );
    assert_eq!(
        root.style_tokens
            .get("focused_background")
            .map(String::as_str),
        Some("token.--editor-state-focused -> token.editor.state.focused -> token.editor.surface.selected")
    );
}

#[test]
fn ui_v2_style_resolver_can_resolve_theme_tokens_when_registry_is_supplied() {
    let mut document = v2_document("asset://ui/tests/theme_style.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Button".to_string(),
            control_id: Some("ThemeButton".to_string()),
            classes: vec!["primary".to_string()],
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "theme_material".to_string(),
        rules: vec![style_rule(
            "Button.primary",
            [
                ("background", "$theme.palette.accent"),
                ("foreground", "var(theme.palette.text.primary)"),
            ],
        )],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let unresolved = UiV2StyleResolver::resolve(&document, &compiled.arena).unwrap();
    assert_eq!(
        unresolved.nodes["root"].self_values["background"].as_str(),
        Some("$theme.palette.accent")
    );

    let resolved = UiV2StyleResolver::resolve_with_theme(
        &document,
        &compiled.arena,
        &UiThemeRegistry::default(),
    )
    .unwrap();
    let root = &resolved.nodes["root"];

    assert_eq!(root.self_values["background"].as_str(), Some("#3cc7d6"));
    assert_eq!(root.self_values["foreground"].as_str(), Some("#e8ecee"));
    assert_eq!(
        root.style_tokens.get("background").map(String::as_str),
        Some("theme.palette.accent")
    );
    assert_eq!(
        root.style_tokens.get("foreground").map(String::as_str),
        Some("theme.palette.text.primary")
    );
}

#[test]
fn ui_v2_surface_builder_resolves_theme_tokens_for_static_and_runtime_rules() {
    let mut document = v2_document("asset://ui/tests/theme_surface.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Button".to_string(),
            control_id: Some("ThemeSurfaceButton".to_string()),
            classes: vec!["primary".to_string()],
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "theme_surface_material".to_string(),
        rules: vec![
            style_rule(
                "Button.primary",
                [("background", "$theme.palette.surface.1")],
            ),
            style_rule(
                "Button.primary:hover",
                [("background", "$theme.palette.accent")],
            ),
        ],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document_with_theme(
        UiTreeId::new("runtime.ui.v2.theme_surface"),
        &document,
        &compiled,
        &UiThemeRegistry::default(),
    )
    .unwrap();
    let node_id = node_id_by_control_id(&surface, "ThemeSurfaceButton");

    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#171a1d")
    );
    assert_eq!(
        runtime_style_token(&surface, node_id, "background"),
        Some("theme.palette.surface.1")
    );

    assert!(surface.component_states.set_hovered(node_id, true));
    surface.mark_component_state_render_dirty(node_id).unwrap();
    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#3cc7d6")
    );
    assert_eq!(
        runtime_style_token(&surface, node_id, "background"),
        Some("theme.palette.accent")
    );
}
