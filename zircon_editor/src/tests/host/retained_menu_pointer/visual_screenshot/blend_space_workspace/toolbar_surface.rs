use super::*;

#[test]
fn blend_space_title_and_preview_toolbars_share_the_panel_header_surface_contract() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = std::fs::read_to_string(manifest.join(
        "assets/ui/editor/components/workbench/modules/extensions/animation/\
             workbench_extension_blend_space_workspace.zui",
    ))
    .expect("Blend Space workspace asset should be readable");
    let parsed = toml::from_str::<toml::Value>(&source)
        .expect("Blend Space workspace asset should parse as TOML");
    let nodes = parsed
        .get("nodes")
        .and_then(toml::Value::as_table)
        .expect("Blend Space workspace should declare nodes");

    let center_header = nodes
        .get("blend_space_center_header")
        .and_then(toml::Value::as_table)
        .expect("Blend Space center header should exist");
    assert_eq!(
        center_header.get("component").and_then(toml::Value::as_str),
        Some("WorkbenchPanelHeader"),
        "the primary title bar should compose the shared panel-header surface"
    );
    let center_children = center_header
        .get("children")
        .and_then(toml::Value::as_array)
        .expect("Blend Space center header should mount slotted children");
    for (child_id, slot_name) in [
        ("blend_space_center_title", "title"),
        ("blend_space_asset_summary", "actions"),
        ("blend_space_preview_button", "actions"),
        ("blend_space_apply_button", "actions"),
    ] {
        assert!(
            center_children.iter().any(|child| {
                child.get("node").and_then(toml::Value::as_str) == Some(child_id)
                    && child
                        .get("slot")
                        .and_then(toml::Value::as_table)
                        .and_then(|slot| slot.get("name"))
                        .and_then(toml::Value::as_str)
                        == Some(slot_name)
            }),
            "Blend Space center header must mount `{child_id}` through the `{slot_name}` slot"
        );
    }
    assert!(
        !nodes.contains_key("blend_space_header_fill"),
        "the shared stretch title slot must replace the feature-local center-header spacer"
    );

    let preview_toolbar = nodes
        .get("blend_space_preview_toolbar")
        .and_then(toml::Value::as_table)
        .expect("Blend Space preview toolbar should exist");
    let props = preview_toolbar
        .get("props")
        .and_then(toml::Value::as_table)
        .expect("Blend Space preview toolbar should declare surface props");

    for (property, expected) in [
        ("background_color", "$editor.surface.3"),
        ("border_color", "$editor.separator.soft"),
        ("border_width", "$editor.control.border_width"),
        ("corner_radius", "$editor.control.radius.control"),
        ("layout_padding_left", "$editor.density.gap.medium"),
        ("layout_padding_right", "$editor.density.gap.medium"),
    ] {
        assert_eq!(
            props.get(property).and_then(toml::Value::as_str),
            Some(expected),
            "Blend Space preview toolbar must source `{property}` from the shared panel-header token contract"
        );
    }
    let layout = preview_toolbar
        .get("layout")
        .and_then(toml::Value::as_table)
        .expect("Blend Space preview toolbar should declare layout");
    assert_eq!(
        layout
            .get("container")
            .and_then(toml::Value::as_table)
            .and_then(|container| container.get("gap"))
            .and_then(toml::Value::as_str),
        Some("$editor.density.gap.small"),
        "Blend Space preview toolbar must source its internal gap from the shared density tokens"
    );
}
