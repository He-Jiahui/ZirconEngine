use super::support::collect_rust_files;

fn slint_host_import_blocks(source: &str) -> Vec<String> {
    let normalized = source.split_whitespace().collect::<String>();
    let mut blocks = Vec::new();
    let mut rest = normalized.as_str();

    while let Some(start) = rest.find("usecrate::ui::slint_host::{") {
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
fn view_presentations_keep_asset_and_welcome_slint_dtos_at_ui_boundary_only() {
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
            .join("slint_host")
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
        let import_blocks = slint_host_import_blocks(&source);

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
                    "expected {:?} to stop importing generated Slint view DTO `{forbidden}` into layouts::views",
                    path.file_name().expect("file name")
                );
            }
            assert!(
                !source.contains(&format!("crate::ui::slint_host::{forbidden}")),
                "expected {:?} to stop importing generated Slint view DTO `{forbidden}` into layouts::views",
                path.file_name().expect("file name")
            );
        }
    }

    for required in [
        "fn to_slint_welcome_pane(",
        "fn to_slint_recent_projects(",
        "fn to_slint_asset_folders(",
        "fn to_slint_asset_items(",
        "fn to_slint_asset_references(",
        "fn to_slint_asset_selection(",
        "fn to_slint_scene_viewport_chrome(",
        "fn to_slint_pane(",
    ] {
        assert!(
            apply_presentation.contains(required),
            "expected apply_presentation.rs to own Slint boundary conversion `{required}`"
        );
    }
}
