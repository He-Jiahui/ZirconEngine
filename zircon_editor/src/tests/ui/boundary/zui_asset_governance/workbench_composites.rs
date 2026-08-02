use std::{collections::BTreeSet, fs};

use toml::Value;

use super::support::{collect_zui_files, editor_asset_root};

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
        "slots = { value = { required = true, multiple = false, kind = \"linear\", accepts = [\"WorkbenchCheckbox\", \"WorkbenchDropdown\", \"WorkbenchField\", \"WorkbenchNumberField\", \"WorkbenchRangeSlider\", \"WorkbenchSlider\", \"WorkbenchToggle\"] } }",
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
        "slots = { title = { required = true, multiple = false, kind = \"linear\", accepts = [\"WorkbenchCaption\", \"WorkbenchSectionTitle\"] }, actions = { multiple = true, kind = \"linear\", accepts = [\"WorkbenchButton\", \"WorkbenchCaption\", \"WorkbenchChip\", \"WorkbenchDropdown\", \"WorkbenchIconButton\", \"WorkbenchToggle\"] } }",
        "component = \"HorizontalGroup\"",
        "classes = [\"workbench-panel-header\", \"workbench-panel-toolbar\"]",
        "component = \"Slot\"",
        "name = \"title\"",
        "name = \"actions\"",
        "styles = [\"res://ui/editor/theme/editor_tokens.zui\"]",
        "background_color = \"$editor.surface.3\"",
        "border_color = \"$editor.separator.soft\"",
        "border_width = \"$editor.control.border_width\"",
        "corner_radius = \"$editor.control.radius.control\"",
        "layout_padding_left = \"$editor.density.gap.medium\"",
        "layout_padding_right = \"$editor.density.gap.medium\"",
        "container = { kind = \"HorizontalBox\", gap = \"$editor.density.gap.small\" }",
        "height = { min = \"$editor.control.height.dense\", preferred = \"$editor.control.height.dense\", max = \"$editor.control.height.compact\", stretch = \"Fixed\" }",
    ] {
        assert!(
            source.contains(required),
            "panel header must preserve the compact Unreal toolbar/header contract: {required}"
        );
    }
    for forbidden in [
        "foreground_color =",
        "font_size =",
        "font_weight =",
        "position =",
        "#111416",
        "gap = 2.0",
        "min = 28.0",
        "max = 30.0",
    ] {
        assert!(
            !source.contains(forbidden),
            "panel header must inherit shared tokens and relative layout: {forbidden}"
        );
    }
}

#[test]
fn workbench_diagnostic_row_exposes_required_status_item_slots() {
    let component_path = editor_asset_root()
        .join("ui/editor/components/workbench/composites/feedback/workbench_diagnostic_row.zui");
    let component_source = fs::read_to_string(&component_path)
        .unwrap_or_else(|error| panic!("read `{}`: {error}", component_path.display()));
    let catalog_path = editor_asset_root().join("ui/editor/components/catalog.toml");
    let catalog_source = fs::read_to_string(&catalog_path)
        .unwrap_or_else(|error| panic!("read `{}`: {error}", catalog_path.display()));

    let slot_contract = "slots = { severity = { required = true, multiple = false, kind = \"linear\", accepts = [\"WorkbenchStatusItem\"] }, message = { required = true, multiple = false, kind = \"linear\", accepts = [\"WorkbenchStatusItem\"] } }";
    assert!(
        component_source.contains(slot_contract),
        "diagnostic rows must expose required WorkbenchStatusItem severity/message slots"
    );
    assert!(
        component_source.contains("height = { min = \"$editor.density.row_height\", preferred = \"$editor.density.row_height\", max = \"$editor.density.row_height\", stretch = \"Fixed\" }"),
        "diagnostic rows must inherit the shared 28px row-height token"
    );
    for forbidden in [
        "background_color =",
        "border_color =",
        "foreground_color =",
        "font_size =",
        "font_weight =",
        "min = 22.0, preferred = 24.0, max = 26.0",
    ] {
        assert!(
            !component_source.contains(forbidden),
            "diagnostic rows must inherit shared visual tokens: {forbidden}"
        );
    }

    let catalog: Value = toml::from_str(&catalog_source)
        .unwrap_or_else(|error| panic!("parse `{}`: {error}", catalog_path.display()));
    let diagnostic = catalog
        .get("components")
        .and_then(Value::as_array)
        .and_then(|components| {
            components.iter().find(|component| {
                component.get("component_id").and_then(Value::as_str)
                    == Some("WorkbenchDiagnosticRow")
            })
        })
        .unwrap_or_else(|| panic!("catalog must declare WorkbenchDiagnosticRow"));
    assert_eq!(
        diagnostic.get("document_id").and_then(Value::as_str),
        Some(
            "res://ui/editor/components/workbench/composites/feedback/workbench_diagnostic_row.zui"
        )
    );
    assert_eq!(
        diagnostic.get("binding_namespace").and_then(Value::as_str),
        Some("WorkbenchDiagnosticRow")
    );
    assert_eq!(
        diagnostic.get("tier").and_then(Value::as_str),
        Some("composite")
    );

    let slots = diagnostic
        .get("slots")
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("diagnostic-row catalog must declare slots"));
    for slot_name in ["severity", "message"] {
        let slot = slots
            .iter()
            .find(|slot| slot.get("name").and_then(Value::as_str) == Some(slot_name))
            .unwrap_or_else(|| panic!("diagnostic-row catalog must declare `{slot_name}`"));
        assert_eq!(slot.get("kind").and_then(Value::as_str), Some("linear"));
        assert_eq!(slot.get("required").and_then(Value::as_bool), Some(true));
        assert_ne!(slot.get("multiple").and_then(Value::as_bool), Some(true));
        let accepts = slot
            .get("accepts")
            .and_then(Value::as_array)
            .unwrap_or_else(|| panic!("diagnostic-row `{slot_name}` must declare accepts"));
        assert_eq!(accepts.len(), 1);
        assert_eq!(
            accepts.first().and_then(Value::as_str),
            Some("WorkbenchStatusItem")
        );
    }
}

#[test]
fn workbench_named_slots_are_typed_and_catalog_backed() {
    let asset_root = editor_asset_root();
    let catalog_path = asset_root.join("ui/editor/components/catalog.toml");
    let catalog_source = fs::read_to_string(&catalog_path)
        .unwrap_or_else(|error| panic!("read `{}`: {error}", catalog_path.display()));
    let catalog: Value = toml::from_str(&catalog_source)
        .unwrap_or_else(|error| panic!("parse `{}`: {error}", catalog_path.display()));
    let catalog_components = catalog
        .get("components")
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("catalog must declare components"));
    let workbench_root = asset_root.join("ui/editor/components/workbench");
    let mut typed_component_count = 0usize;

    for path in collect_zui_files(&workbench_root) {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
        let document: Value = toml::from_str(&source)
            .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));
        let Some(components) = document.get("components").and_then(Value::as_table) else {
            continue;
        };

        for (component_id, component) in components {
            let Some(asset_slots) = component.get("slots").and_then(Value::as_table) else {
                continue;
            };
            if asset_slots.is_empty() {
                continue;
            }
            typed_component_count += 1;

            let catalog_component = catalog_components
                .iter()
                .find(|candidate| {
                    candidate.get("component_id").and_then(Value::as_str) == Some(component_id)
                })
                .unwrap_or_else(|| {
                    panic!(
                        "{} component `{component_id}` must be catalog-backed",
                        path.display()
                    )
                });
            let catalog_slots = catalog_component
                .get("slots")
                .and_then(Value::as_array)
                .unwrap_or_else(|| {
                    panic!("catalog component `{component_id}` must declare its named slots")
                });
            assert_eq!(
                catalog_slots.len(),
                asset_slots.len(),
                "{} component `{component_id}` slot count must match catalog",
                path.display()
            );

            for (slot_name, asset_slot) in asset_slots {
                let catalog_slot = catalog_slots
                    .iter()
                    .find(|slot| slot.get("name").and_then(Value::as_str) == Some(slot_name))
                    .unwrap_or_else(|| {
                        panic!("catalog component `{component_id}` must declare slot `{slot_name}`")
                    });
                assert_eq!(
                    asset_slot.get("kind").and_then(Value::as_str),
                    catalog_slot.get("kind").and_then(Value::as_str),
                    "{component_id}.{slot_name} kind must match catalog"
                );
                assert_eq!(
                    asset_slot
                        .get("required")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                    catalog_slot
                        .get("required")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                    "{component_id}.{slot_name} required flag must match catalog"
                );
                assert_eq!(
                    asset_slot
                        .get("multiple")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                    catalog_slot
                        .get("multiple")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                    "{component_id}.{slot_name} multiple flag must match catalog"
                );
                let asset_accepts = asset_slot
                    .get("accepts")
                    .and_then(Value::as_array)
                    .unwrap_or_else(|| {
                        panic!("{component_id}.{slot_name} must declare accepted children")
                    })
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<BTreeSet<_>>();
                let catalog_accepts = catalog_slot
                    .get("accepts")
                    .and_then(Value::as_array)
                    .unwrap_or_else(|| {
                        panic!("catalog {component_id}.{slot_name} must declare accepted children")
                    })
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<BTreeSet<_>>();
                assert!(
                    !asset_accepts.is_empty(),
                    "{component_id}.{slot_name} must not fall back to Any"
                );
                assert_eq!(
                    asset_accepts, catalog_accepts,
                    "{component_id}.{slot_name} accepted children must match catalog"
                );
            }
        }
    }

    assert_eq!(
        typed_component_count, 5,
        "workbench typed-slot coverage should change deliberately"
    );
}
