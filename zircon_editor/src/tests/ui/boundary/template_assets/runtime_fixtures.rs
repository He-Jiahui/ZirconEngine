use super::support::source;

#[test]
fn runtime_ui_golden_is_hard_cut_to_zui_fixtures() {
    let runtime_golden = source("src/tests/ui/boundary/runtime_ui_golden.rs");
    for required in [
        "UiV2PrototypeStoreFileCache",
        "UiV2SurfaceBuilder",
        "hud_overlay.zui",
        "pause_menu.zui",
        "settings_dialog.zui",
        "inventory_list.zui",
        "quest_log_dialog.zui",
    ] {
        assert!(
            runtime_golden.contains(required),
            "runtime UI golden should cover .zui fixture marker `{required}`"
        );
    }

    for forbidden in [
        "UiAssetLoader",
        "UiDocumentCompiler",
        "UiTemplateSurfaceBuilder",
        "build_legacy_surface",
        "runtime_hud.ui.toml",
        "pause_dialog.ui.toml",
        "settings_dialog.ui.toml",
        "inventory_dialog.ui.toml",
        "quest_log_dialog.ui.toml",
    ] {
        assert!(
            !runtime_golden.contains(forbidden),
            "runtime UI golden should not keep old runtime schema fallback `{forbidden}`"
        );
    }
}

#[test]
fn runtime_fixture_host_tests_are_hard_cut_to_zui_paths() {
    let pane_body_documents_mod =
        source("src/tests/host/template_runtime/pane_body_documents/mod.rs");
    let pane_body_documents = format!(
        "{}\n{}\n{}",
        source("src/tests/host/template_runtime/pane_body_documents/support.rs"),
        source("src/tests/host/template_runtime/pane_body_documents/runtime_v2_fixtures.rs"),
        source("src/tests/host/template_runtime/pane_body_documents/asset_contracts.rs")
    );
    let material_surface_assets =
        source("src/tests/ui/boundary/global_material_surface_assets/contracts.rs");
    let ui_asset_editor_preview = source("src/tests/ui/ui_asset_editor/runtime_previews.rs");
    let ui_asset_editor_support = source("src/tests/ui/ui_asset_editor/support.rs");

    for module in [
        "mod support;",
        "mod runtime_v2_fixtures;",
        "mod asset_contracts;",
    ] {
        assert!(pane_body_documents_mod.contains(module));
    }

    for required in [
        "runtime_v2_fixture_path",
        "register_document_file",
        "runtime_v2_fixture_buttons_project_interactive_metadata",
        "UiV2AssetLoader",
        "runtime_v2_fixture_assets_parse_from_runtime_crate_assets",
        "from_v2_source",
        "open_v2_preview_session",
    ] {
        assert!(
            pane_body_documents.contains(required)
                || material_surface_assets.contains(required)
                || ui_asset_editor_preview.contains(required)
                || ui_asset_editor_support.contains(required),
            "runtime fixture host/material tests should keep .zui runtime marker `{required}`"
        );
    }

    for forbidden in [
        "runtime_hud.ui.toml",
        "pause_dialog.ui.toml",
        "settings_dialog.ui.toml",
        "inventory_dialog.ui.toml",
        "quest_log_dialog.ui.toml",
    ] {
        assert!(
            !pane_body_documents.contains(forbidden),
            "host/template runtime tests should not keep old runtime schema asset `{forbidden}`"
        );
        assert!(
            !material_surface_assets.contains(forbidden),
            "global material surface tests should not keep old runtime schema asset `{forbidden}`"
        );
        assert!(
            !ui_asset_editor_preview.contains(forbidden),
            "ui asset editor runtime preview tests should not keep old runtime schema asset `{forbidden}`"
        );
        assert!(
            !ui_asset_editor_support.contains(forbidden),
            "ui asset editor support should not include old runtime schema asset `{forbidden}`"
        );
    }
}
