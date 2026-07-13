use std::fs;

use super::support::editor_asset_root;

#[test]
fn workbench_transport_controls_match_unreal_animation_scrub_density() {
    let path = editor_asset_root().join(
        "ui/editor/components/workbench/composites/animation/workbench_transport_controls.zui",
    );
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));

    assert_eq!(
        source
            .matches("component = \"WorkbenchIconButton\"")
            .count(),
        6,
        "transport controls should contain six shared icon-button atoms"
    );
    assert_eq!(
        source.matches("layout_icon_size = 20.0").count(),
        6,
        "Unreal Animation scrub controls use 20x20 playback brushes"
    );
    for edge in ["left", "right", "top", "bottom"] {
        let authored = format!("layout_padding_{edge} = 2.0");
        assert_eq!(
            source.matches(&authored).count(),
            6,
            "Unreal Animation.PlayControlsButton uses 2px padding on every edge: {authored}"
        );
    }
    assert_eq!(
        source.matches("preferred = 28.0").count(),
        7,
        "six 20px glyph buttons plus the root lane should keep compact 28px control height"
    );
}

#[test]
fn workbench_property_editor_row_exposes_unreal_name_and_value_slots() {
    let path = editor_asset_root()
        .join("ui/editor/components/workbench/composites/inputs/workbench_property_editor_row.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));

    for required in [
        "[components.WorkbenchPropertyEditorRow]",
        "slots = { value = { multiple = false } }",
        "component = \"PropertyRow\"",
        "component = \"Slot\"",
        "name = \"value\"",
        "min = 60.0, preferred = 105.0, max = 105.0, stretch = \"Fixed\"",
        "layout = { width = { stretch = \"Stretch\" }, height = { stretch = \"Stretch\" } }",
    ] {
        assert!(
            source.contains(required),
            "property editor row must expose a bounded name column and stretch value slot: {required}"
        );
    }
    for forbidden in [
        "background_color =",
        "border_color =",
        "foreground_color =",
        "font_size =",
        "font_weight =",
    ] {
        assert!(
            !source.contains(forbidden),
            "property editor row must inherit shared painter tokens: {forbidden}"
        );
    }
}

#[test]
fn workbench_panel_header_exposes_compact_title_and_action_slots() {
    let path = editor_asset_root()
        .join("ui/editor/components/workbench/composites/chrome/workbench_panel_header.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));

    for required in [
        "[components.WorkbenchPanelHeader]",
        "slots = { title = { multiple = false }, actions = { multiple = true } }",
        "component = \"HorizontalGroup\"",
        "classes = [\"workbench-panel-toolbar\"]",
        "component = \"Slot\"",
        "name = \"title\"",
        "name = \"actions\"",
        "container = { kind = \"HorizontalBox\", gap = 2.0 }",
        "height = { min = 28.0, preferred = 28.0, max = 30.0, stretch = \"Fixed\" }",
    ] {
        assert!(
            source.contains(required),
            "panel header must preserve the compact Unreal toolbar/header contract: {required}"
        );
    }
    for forbidden in [
        "background_color =",
        "border_color =",
        "foreground_color =",
        "font_size =",
        "font_weight =",
        "position =",
    ] {
        assert!(
            !source.contains(forbidden),
            "panel header must inherit shared tokens and relative layout: {forbidden}"
        );
    }
}
