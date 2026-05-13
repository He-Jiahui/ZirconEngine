use super::support::collect_rust_files;

fn retained_host_import_blocks(source: &str) -> Vec<String> {
    let normalized = source.split_whitespace().collect::<String>();
    let mut blocks = Vec::new();
    let mut rest = normalized.as_str();

    while let Some(start) = rest.find("usecrate::ui::retained_host::{") {
        let after_start = &rest[start..];
        let Some(end) = after_start.find("};") else {
            break;
        };
        blocks.push(after_start[..end + 2].to_string());
        rest = &after_start[end + 2..];
    }

    blocks
}

fn block_imports_name(block: &str, name: &str) -> bool {
    [
        format!("{{{name},"),
        format!(",{name},"),
        format!(",{name}}}"),
        format!("{{{name}}}"),
    ]
    .into_iter()
    .any(|pattern| block.contains(&pattern))
}

#[test]
fn view_presentations_keep_asset_and_welcome_host_contract_dtos_at_ui_boundary_only() {
    let views_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("layouts")
        .join("views");
    let mod_source = std::fs::read_to_string(views_root.join("mod.rs")).expect("views mod");
    let apply_presentation = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("ui")
            .join("retained_host")
            .join("ui")
            .join("apply_presentation.rs"),
    )
    .expect("apply presentation");

    assert!(
        mod_source.contains("mod view_data;"),
        "expected layouts::views mod wiring to own Rust view DTOs under view_data.rs"
    );
    assert!(
        views_root.join("view_data.rs").exists(),
        "expected Rust-owned view DTO declaration file under {:?}",
        views_root
    );

    for path in collect_rust_files(&views_root) {
        let source = std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("{path:?}"));
        let import_blocks = retained_host_import_blocks(&source);

        for forbidden in [
            "AssetFolderData",
            "AssetItemData",
            "AssetReferenceData",
            "AssetSelectionData",
            "NewProjectFormData",
            "RecentProjectData",
            "SceneViewportChromeData",
            "TemplateNodeFrameData",
            "TemplatePaneNodeData",
            "WelcomePaneData",
        ] {
            for block in &import_blocks {
                assert!(
                    !block_imports_name(block, forbidden),
                    "expected {:?} to stop importing host-contract DTO `{forbidden}` into layouts::views",
                    path.file_name().expect("file name")
                );
            }
            assert!(
                !source.contains(&format!("crate::ui::retained_host::{forbidden}")),
                "expected {:?} to stop importing host-contract DTO `{forbidden}` into layouts::views",
                path.file_name().expect("file name")
            );
        }
    }

    for required in [
        "fn to_host_contract_welcome_pane(",
        "fn to_host_contract_recent_projects(",
        "fn to_host_contract_asset_folders(",
        "fn to_host_contract_asset_items(",
        "fn to_host_contract_asset_references(",
        "fn to_host_contract_asset_selection(",
        "fn to_host_contract_scene_viewport_chrome(",
        "fn to_host_contract_pane(",
    ] {
        assert!(
            apply_presentation.contains(required),
            "expected apply_presentation.rs to own host-contract boundary conversion `{required}`"
        );
    }
}

#[test]
fn v2_view_projection_uses_runtime_v2_file_cache_without_editor_local_loader() {
    let view_projection = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("ui")
            .join("layouts")
            .join("views")
            .join("view_projection.rs"),
    )
    .expect("view projection source");

    for required in [
        "UiV2PrototypeStoreFileCache",
        "view_v2_store_file_cache()",
        ".load_store(",
        "build_surface_from_compiled_document(",
    ] {
        assert!(
            view_projection.contains(required),
            "view_projection.rs should route v2 views through runtime file cache marker `{required}`"
        );
    }
    for forbidden in [
        "UiV2AssetLoader",
        "load_toml_file",
        "document.tokens.extend",
        "document.stylesheets.extend",
        "UiV2SurfaceBuilder::build_surface(",
    ] {
        assert!(
            !view_projection.contains(forbidden),
            "view_projection.rs should not keep editor-local v2 loader/cache bypass `{forbidden}`"
        );
    }

    let asset_browser = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("ui")
            .join("editor")
            .join("asset_browser.v2.ui.toml"),
    )
    .expect("asset browser v2 asset");
    assert!(asset_browser.contains("res://ui/theme/editor_base.v2.ui.toml"));
    assert!(
        !asset_browser.contains("res://ui/theme/editor_base.ui.toml"),
        "v2 asset browser must not import the old schema theme"
    );
}
