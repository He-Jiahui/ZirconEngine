use std::{collections::BTreeMap, path::PathBuf};

use zircon_runtime::ui::v2::UiZuiAssetLoader;
use zircon_runtime_interface::ui::v2::UiV2AssetKind;

use super::support::{
    asset_relative_path, check_fixed_axis_contract, check_interactive_material_contract,
    collect_import_graph_files, collect_ui_files, effective_root,
    has_scrollable_or_bounded_viewport, import_graph, imports_editor_design_system_theme,
    is_bounded_collection_exception, is_collection_heavy, is_component_library,
    is_material_import_pending_surface, load_documents, root_has_responsive_contract, visit_nodes,
    EDITOR_BASE_THEME_ZUI, MATERIAL_THEME_ZUI, PRIMARY_EDITOR_PANE_TEMPLATES,
    STRICT_WORKBENCH_THEME_ZUI,
};

const EXPECTED_VIEW_SURFACES: [&str; 41] = [
    "editor/asset_browser.zui",
    "editor/assets_activity.zui",
    "editor/component_showcase.zui",
    "editor/console.zui",
    "editor/fyrox_panel_demo_window.zui",
    "editor/hierarchy.zui",
    "editor/host/animation_graph_body.zui",
    "editor/host/animation_sequence_body.zui",
    "editor/host/asset_surface_controls.zui",
    "editor/host/build_export_desktop_body.zui",
    "editor/host/console_body.zui",
    "editor/host/editor_main_frame.zui",
    "editor/host/floating_window_source.zui",
    "editor/host/generated_bottom_body.zui",
    "editor/host/hierarchy_body.zui",
    "editor/host/inspector_body.zui",
    "editor/host/inspector_surface_controls.zui",
    "editor/host/module_plugins_body.zui",
    "editor/host/pane_surface_controls.zui",
    "editor/host/performance_timeline_body.zui",
    "editor/host/runtime_diagnostics_body.zui",
    "editor/host/scene_viewport_toolbar.zui",
    "editor/host/startup_welcome_controls.zui",
    "editor/host/workbench_shell.zui",
    "editor/inspector.zui",
    "editor/layout_demo_window.zui",
    "editor/material_component_lab.zui",
    "editor/material_demo_window.zui",
    "editor/product_binding_fixture.zui",
    "editor/project_overview.zui",
    "editor/ui_asset_editor.zui",
    "editor/welcome.zui",
    "editor/windows/asset_window.zui",
    "editor/windows/ui_layout_editor_window.zui",
    "editor/windows/workbench_window.zui",
    "editor/workbench_activity_rail.zui",
    "editor/workbench_dock_header.zui",
    "editor/workbench_menu_chrome.zui",
    "editor/workbench_menu_popup.zui",
    "editor/workbench_page_chrome.zui",
    "editor/workbench_status_bar.zui",
];

#[test]
fn editor_base_control_geometry_uses_central_design_tokens() {
    for token in [
        "$editor.control.border_width",
        "$editor.control.radius.small",
        "$editor.control.radius.panel",
    ] {
        assert!(
            EDITOR_BASE_THEME_ZUI.contains(token),
            "editor base theme must consume the central control token `{token}`"
        );
    }

    for raw_metric in [
        "width = 1.0",
        "radius = 5.0",
        "radius = 6.0",
        "radius = 8.0",
    ] {
        assert!(
            !EDITOR_BASE_THEME_ZUI.contains(raw_metric),
            "editor base theme must not retain local control geometry `{raw_metric}`"
        );
    }
}

#[test]
fn primary_editor_panes_use_strict_workbench_theme_and_shared_tokens() {
    for (name, template) in PRIMARY_EDITOR_PANE_TEMPLATES {
        assert!(
            template.contains(STRICT_WORKBENCH_THEME_ZUI),
            "{name} pane must import the strict Workbench theme"
        );
        assert!(
            !template.contains("res://ui/theme/editor_base.zui"),
            "{name} pane must not retain the legacy Editor base theme"
        );
        assert!(
            template.contains("classes = [\"workbench-shell-root\"]"),
            "{name} pane root must use the strict Workbench shell role"
        );
        for token in [
            "$editor.control.border_width",
            "$editor.control.radius.small",
        ] {
            assert!(
                template.contains(token),
                "{name} pane must consume the shared token `{token}`"
            );
        }
        for raw_metric in ["border_width = 1.0", "radius = 3.0", "radius = 4.0"] {
            assert!(
                !template.contains(raw_metric),
                "{name} pane must not retain local control geometry `{raw_metric}`"
            );
        }
    }

    for (name, template) in PRIMARY_EDITOR_PANE_TEMPLATES {
        for token in [
            "$editor.density.row_height",
            "$editor.typography.strong.weight",
        ] {
            assert!(
                template.contains(token),
                "{name} pane must consume the shared token `{token}`"
            );
        }
        assert!(
            !template.contains("font_weight = 600"),
            "{name} pane must not retain a local strong font weight"
        );
        assert!(
            !template.contains("height = { min = 28.0"),
            "{name} pane must not retain a local standard row height"
        );
    }
}

#[test]
fn global_editor_design_system_assets_follow_responsive_contracts() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let assets_root = repo.join("assets/ui");
    let files = collect_ui_files(&repo);
    assert_eq!(
        files.len(),
        EXPECTED_VIEW_SURFACES.len(),
        "Current .zui view surface inventory changed; update the acceptance inventory and this conformance test together"
    );
    let actual_view_surfaces: Vec<_> = files
        .iter()
        .map(|path| asset_relative_path(path, &assets_root))
        .collect();
    assert_eq!(
        actual_view_surfaces,
        EXPECTED_VIEW_SURFACES.map(str::to_string).to_vec(),
        "Current .zui view surface paths changed; update the acceptance inventory and this conformance test together"
    );

    let documents = load_documents(&files);
    let import_documents = load_documents(&collect_import_graph_files(&repo, &files));
    let import_graph = import_graph(&import_documents, &assets_root);
    let mut failures = Vec::new();

    for path in &files {
        let document = documents.get(path).expect("document loaded");
        let relative = asset_relative_path(path, &assets_root);
        if is_component_library(&relative, document) {
            continue;
        }
        if !imports_editor_design_system_theme(&relative, &import_graph)
            && !is_material_import_pending_surface(&relative)
        {
            failures.push(format!(
                "{} must import the Material or strict Workbench theme directly or through another imported asset",
                relative
            ));
        }

        let Some(root) = document.get("root") else {
            failures.push(format!("{} must define [root]", relative));
            continue;
        };
        let Some(effective_root) = effective_root(document, root) else {
            failures.push(format!(
                "{} root must resolve to an authored node",
                relative
            ));
            continue;
        };
        if !root_has_responsive_contract(&relative, effective_root) {
            failures.push(format!(
                "{} root must stretch in width and height unless it is a bounded popup/menu/dialog/window chrome surface",
                relative
            ));
        }

        visit_nodes(
            document,
            &relative,
            "root",
            effective_root,
            &mut |location, node| {
                check_interactive_material_contract(location, node, &mut failures);
                check_fixed_axis_contract(location, node, &mut failures);
            },
        );

        if is_collection_heavy(&relative, document)
            && !has_scrollable_or_bounded_viewport(document, effective_root)
            && !is_bounded_collection_exception(&relative)
        {
            failures.push(format!(
                "{} is list/table/grid heavy and must expose ScrollableBox or an explicit bounded viewport",
                relative
            ));
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn editor_design_system_import_graph_uses_normalized_res_paths() {
    let mut graph = BTreeMap::new();
    graph.insert(
        "editor/welcome.zui".to_string(),
        vec!["res://ui/theme/editor_base.zui".to_string()],
    );
    graph.insert(
        "theme/editor_base.zui".to_string(),
        vec![MATERIAL_THEME_ZUI.to_string()],
    );

    assert!(imports_editor_design_system_theme(
        "editor/welcome.zui",
        &graph
    ));

    graph.insert(
        "editor/hierarchy.zui".to_string(),
        vec![STRICT_WORKBENCH_THEME_ZUI.to_string()],
    );
    assert!(imports_editor_design_system_theme(
        "editor/hierarchy.zui",
        &graph
    ));
}

#[test]
fn runtime_v2_fixture_assets_parse_from_runtime_crate_assets() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let runtime_fixture_root = repo
        .parent()
        .expect("zircon_editor lives directly under workspace root")
        .join("zircon_runtime/assets/ui/runtime/fixtures");

    for (file_name, asset_id, root_node) in [
        (
            "hud_overlay.zui",
            "res://ui/runtime/fixtures/hud_overlay.zui",
            "hud_root",
        ),
        (
            "pause_menu.zui",
            "res://ui/runtime/fixtures/pause_menu.zui",
            "pause_root",
        ),
        (
            "settings_dialog.zui",
            "res://ui/runtime/fixtures/settings_dialog.zui",
            "settings_root",
        ),
        (
            "inventory_list.zui",
            "res://ui/runtime/fixtures/inventory_list.zui",
            "inventory_root",
        ),
        (
            "quest_log_dialog.zui",
            "res://ui/runtime/fixtures/quest_log_dialog.zui",
            "quest_root",
        ),
    ] {
        let path = runtime_fixture_root.join(file_name);
        let document = UiZuiAssetLoader::load_zui_file(&path)
            .unwrap_or_else(|error| panic!("{} parses as runtime .zui: {error}", path.display()));

        assert_eq!(document.asset.kind, UiV2AssetKind::View);
        assert_eq!(document.asset.version, 2);
        assert_eq!(document.asset.id, asset_id);
        assert_eq!(document.root_node_id(), Some(root_node));
        assert!(
            document.nodes.contains_key(root_node),
            "{file_name} should contain its declared root node"
        );
    }
}
