//! 验证内联样式位于样式表级联之上，实例覆盖在构建表面后仍可观察。
use super::super::*;

#[test]
fn ui_v2_inline_style_overrides_cascade_values_in_style_overrides() {
    let mut document = v2_document("asset://ui/tests/style_override_priority.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Button".to_string(),
            control_id: Some("OverrideButton".to_string()),
            classes: vec!["material-button".to_string()],
            style: UiV2StyleDeclarationBlock {
                self_values: BTreeMap::from([(
                    "button_variant".to_string(),
                    Value::String("outlined".to_string()),
                )]),
                slot: BTreeMap::new(),
            },
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "style_override_priority".to_string(),
        rules: vec![style_rule(
            "Button.material-button",
            [("button_variant", "contained")],
        )],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.style_override_priority"),
        &document,
        &compiled,
    )
    .unwrap();
    let node_id = node_id_by_control_id(&surface, "OverrideButton");
    let metadata = surface
        .tree
        .nodes
        .get(&node_id)
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();

    assert_eq!(
        metadata.attributes["button_variant"].as_str(),
        Some("contained")
    );
    assert_eq!(
        metadata.style_overrides["button_variant"].as_str(),
        Some("outlined")
    );
}

#[test]
fn ui_v2_imported_inline_style_tokens_resolve_before_native_metadata() {
    let temp_dir = v2_cache_temp_dir("inline_native_style_tokens");
    let assets_root = temp_dir.join("assets");
    let view_path = assets_root.join("ui/editor/inline.zui");
    let alias_path = assets_root.join("ui/components/old_button.zui");
    let palette_path = assets_root.join("ui/theme/shared_palette.zui");
    for path in [&view_path, &alias_path, &palette_path] {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    }
    std::fs::write(
        &view_path,
        r##"
[asset]
kind = "view"
id = "ui.editor.inline_native_style_tokens"
version = 2
[imports]
widgets = ["res://ui/components/old_button.zui#OldButton"]
styles = ["res://ui/theme/shared_palette.zui"]
[root]
node = "root"
[nodes.root]
component = "OldButton"
control_id = "InlineTokenButton"
classes = ["unreal-control"]
props = { selected = true, checked = false }
style = { self = { background_color = "$unreal_panel", hover_background_color = "$unreal_hover", selected_background_color = "$unreal_primary", font_size = "$unreal_font_size", border_color = "$theme.palette.accent", corner_radius = 4.0, font_family = "Roboto" } }
"##,
    )
    .unwrap();
    std::fs::write(
        &alias_path,
        r##"
[asset]
kind = "component"
id = "ui.components.old_button"
version = 2
[components.OldButton]
root = "button"
[nodes.button]
component = "Button"
props = { background_color = "#DDEEFF", hover_background_color = "#AABBCC", selected_background_color = "#CCDDEE", font_size = 9.0, border_color = "#112233", corner_radius = 2.0, font_family = "OldFont" }
"##,
    )
    .unwrap();
    let palette = r##"
[asset]
kind = "style"
id = "ui.theme.shared_palette"
version = 2
[tokens]
unreal_background = "#151515"
unreal_panel = "#242424"
unreal_hover = "#575757"
unreal_primary = "#0070E0"
unreal_font_size = 13.333333
[[stylesheets]]
id = "inline_native_style_tokens"
[[stylesheets.rules]]
selector = "Button.unreal-control"
set = { self = { background_color = "$unreal_background", font_size = 10.0, corner_radius = 0.0 } }
"##;
    std::fs::write(&palette_path, palette).unwrap();
    let mut cache = UiV2PrototypeStoreFileCache::new();
    let initial = cache.load_store([&view_path]).unwrap();
    assert!(initial.unresolved_imports.is_empty());
    assert!(initial
        .store
        .get("res://ui/theme/shared_palette.zui")
        .is_some());
    assert_eq!(initial.source_receipts.len(), 3);

    std::fs::write(
        &palette_path,
        palette
            .replace("unreal_panel = \"#242424\"", "unreal_panel = \"#2F2F2F\"")
            .replace("unreal_font_size = 13.333333", "unreal_font_size = 14.5"),
    )
    .unwrap();
    let changed = cache.load_store([&view_path]).unwrap();
    assert!(!changed.cache_hit);

    for (outcome, expected_background, expected_font) in [
        (&initial, "#242424", 13.333333),
        (&changed, "#2F2F2F", 14.5),
    ] {
        let authored_root = outcome
            .compiled
            .arena
            .node(outcome.compiled.arena.root.unwrap())
            .unwrap();
        assert_eq!(
            authored_root.style.self_values["background_color"].as_str(),
            Some("$unreal_panel")
        );
        for with_theme in [false, true] {
            let surface = if with_theme {
                UiV2SurfaceBuilder::build_surface_from_compiled_document_with_theme(
                    UiTreeId::new("runtime.ui.v2.inline_native_style_tokens.theme"),
                    outcome.root_document.as_ref(),
                    outcome.compiled.as_ref(),
                    &UiThemeRegistry::default(),
                )
            } else {
                UiV2SurfaceBuilder::build_surface_from_compiled_document(
                    UiTreeId::new("runtime.ui.v2.inline_native_style_tokens"),
                    outcome.root_document.as_ref(),
                    outcome.compiled.as_ref(),
                )
            }
            .unwrap();
            let node_id = node_id_by_control_id(&surface, "InlineTokenButton");
            let metadata = surface
                .tree
                .nodes
                .get(&node_id)
                .unwrap()
                .template_metadata
                .as_ref()
                .unwrap();

            // The static class baseline remains observable independently of inline paint values.
            assert_eq!(
                metadata.attributes["background_color"].as_str(),
                Some("#151515")
            );
            assert_eq!(metadata.attributes["font_size"].as_float(), Some(10.0));
            assert_eq!(
                metadata.style_overrides["background_color"].as_str(),
                Some(expected_background)
            );
            assert_eq!(
                metadata.style_overrides["font_size"].as_float(),
                Some(expected_font)
            );
            assert_eq!(
                metadata.style_overrides["hover_background_color"].as_str(),
                Some("#575757")
            );
            assert_eq!(
                metadata.style_overrides["selected_background_color"].as_str(),
                Some("#0070E0")
            );
            assert_eq!(
                metadata.style_overrides["corner_radius"].as_float(),
                Some(4.0)
            );
            assert_eq!(
                metadata.style_overrides["font_family"].as_str(),
                Some("Roboto")
            );
            assert_eq!(metadata.attributes["selected"].as_bool(), Some(true));
            assert_eq!(metadata.attributes["checked"].as_bool(), Some(false));
            assert_eq!(
                metadata.style_overrides["border_color"].as_str(),
                Some(if with_theme {
                    "#3cc7d6"
                } else {
                    "$theme.palette.accent"
                })
            );
        }
    }
    std::fs::remove_dir_all(temp_dir).unwrap();
}
