use super::*;

#[test]
fn runtime_ui_entry_assets_do_not_live_under_src() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_root = manifest_dir.join("src");
    let offending = collect_ui_toml_files(&src_root);

    assert!(
        offending.is_empty(),
        "production runtime ui entry assets must not live under `src/`: {}",
        format_paths(&offending, manifest_dir)
    );
}

#[test]
fn legacy_runtime_fixture_source_directory_is_removed() {
    let legacy_fixture_dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/runtime_ui/fixtures");
    assert!(
        !legacy_fixture_dir.exists(),
        "runtime fixture source directory must stay removed after the assets/ cutover: {}",
        legacy_fixture_dir.display()
    );
}

#[test]
fn runtime_fixture_assets_live_under_crate_assets() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture_root = manifest_dir.join("assets/ui/runtime/fixtures");
    let actual_files = collect_ui_toml_files(&fixture_root);

    let expected_files = vec![
        "assets/ui/runtime/fixtures/hud_overlay.v2.ui.toml".to_string(),
        "assets/ui/runtime/fixtures/inventory_list.v2.ui.toml".to_string(),
        "assets/ui/runtime/fixtures/pause_menu.v2.ui.toml".to_string(),
        "assets/ui/runtime/fixtures/quest_log_dialog.v2.ui.toml".to_string(),
        "assets/ui/runtime/fixtures/settings_dialog.v2.ui.toml".to_string(),
    ];

    assert_eq!(
        rel_paths(&actual_files, manifest_dir),
        expected_files,
        "runtime fixtures should live exclusively under crate assets/"
    );
}

#[test]
fn runtime_fixture_loader_stays_on_asset_paths() {
    let fixture_source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/ui/tests/runtime_ui_support/runtime_ui_fixture.rs"),
    )
    .expect("runtime_ui_fixture.rs should be readable");

    for required in [
        "fn relative_asset_path",
        "fn asset_path",
        "use crate::asset::runtime_asset_path",
        "runtime_asset_path(self.relative_asset_path())",
    ] {
        assert!(
            fixture_source.contains(required),
            "runtime fixture loader should keep asset-path helper `{required}`"
        );
    }

    for forbidden in ["fn source(", "include_str!"] {
        assert!(
            !fixture_source.contains(forbidden),
            "runtime fixture loader should not keep source-embedded entry helper `{forbidden}`"
        );
    }
}

#[test]
fn runtime_ui_manager_loads_fixture_documents_from_asset_files() {
    let manager_source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/ui/tests/runtime_ui_support/runtime_ui_manager.rs"),
    )
    .expect("runtime_ui_manager.rs should be readable");

    assert!(
        manager_source.contains("UiV2PrototypeStoreFileCache")
            && manager_source.contains(".load_store(std::iter::once(fixture.asset_path()))")
            && manager_source.contains("UiV2SurfaceBuilder::build_surface_from_compiled_document")
            && manager_source.contains("apply_pointer_dispatch_dirty(&result)")
            && manager_source.contains("rebuild_dirty(self.root_size())"),
        "runtime ui manager should load fixture documents through the v2 heap-resident file cache and refresh the persistent surface by dirty domain"
    );

    for forbidden in [
        "fixture.source()",
        "include_str!",
        "UiAssetLoader::load_toml_file",
        "UiDocumentCompiler",
        "UiTemplateSurfaceBuilder::build_surface_from_compiled_document",
    ] {
        assert!(
            !manager_source.contains(forbidden),
            "runtime ui manager should not regress to embedded fixture source `{forbidden}`"
        );
    }
}

#[test]
fn ui_v2_surface_projection_does_not_call_template_tree_builder() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let surface_builder_source =
        fs::read_to_string(manifest_dir.join("src/ui/v2/surface_builder.rs"))
            .expect("ui v2 surface_builder.rs should be readable");
    let mut combined_source = surface_builder_source;
    for relative in [
        "src/ui/v2/surface_tree/mod.rs",
        "src/ui/v2/surface_tree/node.rs",
        "src/ui/v2/surface_tree/layout.rs",
        "src/ui/v2/surface_tree/slot.rs",
        "src/ui/v2/surface_tree/interaction.rs",
        "src/ui/v2/surface_tree/parse.rs",
    ] {
        combined_source.push('\n');
        combined_source.push_str(
            &fs::read_to_string(manifest_dir.join(relative)).unwrap_or_else(|error| {
                panic!("ui v2 surface tree file {relative} should be readable: {error}")
            }),
        );
    }

    for required in ["UiV2ArenaNode", "UiTreeNode", "UiTemplateNodeMetadata"] {
        assert!(
            combined_source.contains(required),
            "ui v2 surface projection should build runtime tree data directly with `{required}`"
        );
    }

    for forbidden in [
        "UiTemplateTreeBuilder",
        "UiTemplateSurfaceBuilder",
        "crate::ui::template",
        "ui::template::UiTemplateNode",
    ] {
        assert!(
            !combined_source.contains(forbidden),
            "ui v2 surface projection should not depend on old template tree path `{forbidden}`"
        );
    }
}
