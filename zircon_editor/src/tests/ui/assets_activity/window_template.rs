use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use zircon_runtime::ui::v2::{UiV2AssetLoader, UiV2DocumentCompiler, UiV2PrototypeStore};
use zircon_runtime_interface::ui::{layout::UiSlotKind, v2::UiV2AssetError};

fn asset_window_source() -> String {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/ui/editor/windows/asset_window.zui");
    fs::read_to_string(path).expect("asset_window.zui should be readable")
}

fn activity_drawer_window_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets/ui/editor/components/workbench/shell/activity_drawer_window.zui");
    fs::read_to_string(path).expect("activity_drawer_window.zui should be readable")
}

fn ui_layout_editor_window_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets/ui/editor/windows/ui_layout_editor_window.zui");
    fs::read_to_string(path).expect("ui_layout_editor_window.zui should be readable")
}

fn workbench_component_source(relative_path: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets/ui/editor/components/workbench")
        .join(relative_path);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()))
}

#[test]
fn asset_window_uses_activity_drawer_window_with_asset_browser_content() {
    let source = asset_window_source();
    let document =
        UiV2AssetLoader::load_toml_str(&source).expect("asset window .zui asset should parse");

    assert_eq!(
        document.asset.id,
        "res://ui/editor/windows/asset_window.zui"
    );
    assert!(source.contains(
        "res://ui/editor/components/workbench/shell/activity_drawer_window.zui#ActivityDrawerWindow"
    ));
    assert!(source.contains("shell_preset = \"jetbrains_shell\""));
    assert!(source.contains("panel_preset = \"fyrox_panel\""));
    assert!(source.contains("window_model = \"unreal_window_model\""));

    for control in [
        "AssetWindowTreeActivity",
        "AssetWindowDetailsActivity",
        "AssetWindowPreviewActivity",
        "AssetWindowBrowserContent",
    ] {
        assert!(source.contains(control), "missing {control}");
    }
}

#[test]
fn activity_drawer_window_limits_activity_slots_to_container_panels() {
    let source = activity_drawer_window_source();
    let document = UiV2AssetLoader::load_toml_str(&source)
        .expect("activity drawer window .zui asset should parse");
    let component = document
        .components
        .get("ActivityDrawerWindow")
        .expect("activity drawer window component should be declared");
    assert_eq!(component.slots.len(), 7);

    for slot_name in [
        "left_top_activity",
        "left_bottom_activity",
        "right_top_activity",
        "right_bottom_activity",
        "bottom_left_activity",
        "bottom_right_activity",
        "content",
    ] {
        let slot = component
            .slots
            .get(slot_name)
            .unwrap_or_else(|| panic!("activity drawer window should declare `{slot_name}`"));
        assert!(
            slot.multiple,
            "{slot_name} should retain its multi-panel contract"
        );
        assert_eq!(slot.kind, Some(UiSlotKind::Container));
        assert_eq!(slot.accepts, BTreeSet::from(["Container".to_string()]));
        assert!(!slot.accepts_component("IconButton"));
    }
}

#[test]
fn shared_workbench_slot_assets_validate_standardized_children_before_expansion() {
    let panel_header = UiV2AssetLoader::load_toml_str(&workbench_component_source(
        "composites/chrome/workbench_panel_header.zui",
    ))
    .expect("panel header asset should parse");
    let property_editor_row = UiV2AssetLoader::load_toml_str(&workbench_component_source(
        "composites/inputs/workbench_property_editor_row.zui",
    ))
    .expect("property editor row asset should parse");
    let tab_strip = UiV2AssetLoader::load_toml_str(&workbench_component_source(
        "primitives/inputs/workbench_tab_strip.zui",
    ))
    .expect("tab strip asset should parse");
    let mut consumer = UiV2AssetLoader::load_toml_str(
        r#"
[asset]
kind = "view"
id = "test://workbench/shared_slot_contracts.zui"
version = 2

[imports]
widgets = [
  "res://ui/editor/components/workbench/composites/chrome/workbench_panel_header.zui#WorkbenchPanelHeader",
  "res://ui/editor/components/workbench/composites/inputs/workbench_property_editor_row.zui#WorkbenchPropertyEditorRow",
  "res://ui/editor/components/workbench/primitives/inputs/workbench_tab_strip.zui#WorkbenchTabStrip",
]

[root]
node = "root"

[nodes.root]
component = "VerticalGroup"
children = [{ node = "header" }, { node = "property" }, { node = "tabs" }]

[nodes.header]
component = "WorkbenchPanelHeader"
control_id = "SharedSlotHeader"
children = [{ node = "header_title", slot = { name = "title" } }, { node = "header_action", slot = { name = "actions" } }]

[nodes.header_title]
component = "WorkbenchCaption"
control_id = "SharedSlotHeaderTitle"

[nodes.header_action]
component = "WorkbenchCaption"
control_id = "SharedSlotHeaderAction"

[nodes.property]
component = "WorkbenchPropertyEditorRow"
control_id = "SharedSlotProperty"
children = [{ node = "property_value", slot = { name = "value" } }]

[nodes.property_value]
component = "WorkbenchField"
control_id = "SharedSlotPropertyValue"

[nodes.tabs]
component = "WorkbenchTabStrip"
control_id = "SharedSlotTabs"
children = [{ node = "tab_one" }, { node = "tab_two" }]

[nodes.tab_one]
component = "WorkbenchTab"
control_id = "SharedSlotTabOne"

[nodes.tab_two]
component = "WorkbenchTab"
control_id = "SharedSlotTabTwo"
"#,
    )
    .expect("shared workbench slot consumer should parse");
    let mut prototypes = UiV2PrototypeStore::new();
    prototypes.insert(panel_header);
    prototypes.insert(property_editor_row);
    prototypes.insert(tab_strip);

    let compiled = UiV2DocumentCompiler::compile_with_prototype_store(&consumer, &prototypes)
        .expect("shared Workbench atoms should satisfy the declared slot contracts");
    for control_id in [
        "SharedSlotHeaderTitle",
        "SharedSlotHeaderAction",
        "SharedSlotPropertyValue",
        "SharedSlotTabOne",
        "SharedSlotTabTwo",
    ] {
        assert!(
            compiled
                .arena
                .nodes
                .iter()
                .any(|node| node.control_id.as_deref() == Some(control_id)),
            "expanded shared-slot consumer should retain `{control_id}`"
        );
    }

    consumer
        .nodes
        .get_mut("header_title")
        .expect("shared header title should be declared")
        .component = "WorkbenchToggle".to_string();
    let error = UiV2DocumentCompiler::compile_with_prototype_store(&consumer, &prototypes)
        .expect_err("WorkbenchToggle must not fill the Panel Header title slot");
    assert!(matches!(
        error,
        UiV2AssetError::SlotDoesNotAcceptComponent {
            component,
            slot_name,
            child_component,
            ..
        } if component == "WorkbenchPanelHeader"
            && slot_name == "title"
            && child_component == "WorkbenchToggle"
    ));
}

#[test]
fn asset_window_compiles_with_the_activity_drawer_container_slot_contract() {
    let activity_drawer = UiV2AssetLoader::load_toml_str(&activity_drawer_window_source())
        .expect("activity drawer window .zui asset should parse");
    let asset_window = UiV2AssetLoader::load_toml_str(&asset_window_source())
        .expect("asset window .zui asset should parse");
    let mut prototypes = UiV2PrototypeStore::new();
    prototypes.insert(activity_drawer);

    let compiled = UiV2DocumentCompiler::compile_with_prototype_store(&asset_window, &prototypes)
        .expect("asset window container panels should satisfy the activity drawer slot contract");

    for control_id in [
        "AssetWindowTreeActivity",
        "AssetWindowCollectionsActivity",
        "AssetWindowDetailsActivity",
        "AssetWindowPreviewActivity",
        "AssetWindowImportLogActivity",
        "AssetWindowDependencyActivity",
        "AssetWindowBrowserContent",
    ] {
        assert!(
            compiled
                .arena
                .nodes
                .iter()
                .any(|node| node.control_id.as_deref() == Some(control_id)),
            "compiled asset window should retain `{control_id}`"
        );
    }
}

#[test]
fn asset_window_rejects_a_non_container_activity_slot_fill() {
    let activity_drawer = UiV2AssetLoader::load_toml_str(&activity_drawer_window_source())
        .expect("activity drawer window .zui asset should parse");
    let mut asset_window = UiV2AssetLoader::load_toml_str(&asset_window_source())
        .expect("asset window .zui asset should parse");
    asset_window
        .nodes
        .get_mut("tree")
        .expect("asset window tree activity should be declared")
        .component = "IconButton".to_string();
    let mut prototypes = UiV2PrototypeStore::new();
    prototypes.insert(activity_drawer);

    let error = UiV2DocumentCompiler::compile_with_prototype_store(&asset_window, &prototypes)
        .expect_err("IconButton must not fill a container-only activity slot");

    assert!(matches!(
        error,
        UiV2AssetError::SlotDoesNotAcceptComponent {
            component,
            slot_name,
            child_component,
            ..
        } if component == "ActivityDrawerWindow"
            && slot_name == "left_top_activity"
            && child_component == "IconButton"
    ));
}

#[test]
fn ui_layout_editor_window_compiles_with_the_activity_drawer_container_slot_contract() {
    let activity_drawer = UiV2AssetLoader::load_toml_str(&activity_drawer_window_source())
        .expect("activity drawer window .zui asset should parse");
    let ui_layout_editor = UiV2AssetLoader::load_toml_str(&ui_layout_editor_window_source())
        .expect("UI layout editor window .zui asset should parse");
    let mut prototypes = UiV2PrototypeStore::new();
    prototypes.insert(activity_drawer);

    let compiled =
        UiV2DocumentCompiler::compile_with_prototype_store(&ui_layout_editor, &prototypes)
            .expect("UI layout editor panels should satisfy the activity drawer slot contract");

    for control_id in [
        "UILayoutEditorPaletteActivity",
        "UILayoutEditorHierarchyActivity",
        "UILayoutEditorInspectorActivity",
        "UILayoutEditorStyleActivity",
        "UILayoutEditorDiagnosticsActivity",
        "UILayoutEditorLayoutDebugActivity",
        "UILayoutEditorContent",
    ] {
        assert!(
            compiled
                .arena
                .nodes
                .iter()
                .any(|node| node.control_id.as_deref() == Some(control_id)),
            "compiled UI layout editor should retain `{control_id}`"
        );
    }
}
