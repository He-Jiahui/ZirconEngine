use std::{collections::BTreeMap, path::Path};

use crate::ui::binding::EditorUiBindingPayload;

use super::insert_workbench_module_bindings;

#[test]
fn assets_workspace_authored_routes_match_installed_bindings() {
    let source = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "assets/ui/editor/components/workbench/modules/core/assets/workbench_assets_workspace.zui",
    ))
    .expect("Assets workspace asset should be readable");
    let mut bindings = BTreeMap::new();
    insert_workbench_module_bindings(&mut bindings);

    for (event_id, action_id) in [
        (
            "AssetsBrowserTab",
            "workbench.module.assets.browser_tab.select",
        ),
        (
            "AssetsImportTab",
            "workbench.module.assets.import_tab.select",
        ),
        (
            "AssetsValidationTab",
            "workbench.module.assets.validation_tab.select",
        ),
        (
            "AssetsForestRow",
            "workbench.module.assets.forest_row.select",
        ),
        (
            "AssetsMaterialRow",
            "workbench.module.assets.material_row.select",
        ),
        (
            "AssetsTableTree",
            "workbench.module.assets.table_tree.select",
        ),
        (
            "AssetsTableMaterial",
            "workbench.module.assets.table_material.select",
        ),
        (
            "AssetsTableTexture",
            "workbench.module.assets.table_texture.select",
        ),
        ("AssetsOutput", "workbench.module.assets.output.select"),
        ("AssetsImport", "workbench.module.assets.import.invoke"),
    ] {
        let authored_event = format!(
            "id = \"WorkbenchModule/{event_id}\", event = \"Click\", route = \"{action_id}\""
        );
        assert!(
            source.contains(&authored_event),
            "Assets workspace route must remain canonical: {authored_event}"
        );

        let binding = bindings
            .get(&format!("WorkbenchModule/{event_id}"))
            .unwrap_or_else(|| panic!("missing Assets binding for {event_id}"));
        assert!(matches!(
            binding.payload(),
            EditorUiBindingPayload::MenuAction { action_id: actual } if actual == action_id
        ));
    }
}

#[test]
fn assets_workspace_uses_shared_density_and_responsive_side_panes() {
    let source = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "assets/ui/editor/components/workbench/modules/core/assets/workbench_assets_workspace.zui",
    ))
    .expect("Assets workspace asset should be readable");

    for required in [
        "container = { kind = \"HorizontalBox\", gap = 1.0 }",
        "props = { responsive_min_tier = \"narrow\" }\nlayout = { clip = true, container = { kind = \"VerticalBox\", gap = \"$editor.density.gap.small\" }, width = { min = 188.0, preferred = 220.0, max = 250.0, stretch = \"Fixed\" }",
        "props = { responsive_min_tier = \"wide\" }\nlayout = { clip = true, container = { kind = \"VerticalBox\", gap = \"$editor.density.gap.small\" }, width = { min = 210.0, preferred = 260.0, max = 310.0, stretch = \"Fixed\" }",
        "height = { min = 28.0, preferred = 30.0, max = 32.0, stretch = \"Fixed\" }",
    ] {
        assert!(
            source.contains(required),
            "missing Assets workspace responsive density contract: {required}"
        );
    }
    assert!(!source.contains("gap = 10.0"));
    assert!(!source.contains("gap = 8.0"));
    assert!(!source.contains("min = 270.0, preferred = 270.0"));
    assert!(!source.contains("min = 320.0, preferred = 320.0"));
}
