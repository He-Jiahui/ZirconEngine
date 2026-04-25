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
fn workbench_host_window_keeps_generated_slint_shell_dtos_at_ui_boundary_only() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("layouts")
        .join("windows")
        .join("workbench_host_window");
    let mod_source = std::fs::read_to_string(root.join("mod.rs")).expect("workbench host mod");
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
        mod_source.contains("mod host_data;"),
        "expected workbench_host_window mod wiring to own Rust host DTOs under host_data.rs"
    );
    assert!(
        root.join("host_data.rs").exists(),
        "expected Rust-owned host DTO declaration file under {:?}",
        root
    );

    for path in collect_rust_files(&root) {
        let source = std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("{path:?}"));
        let import_blocks = slint_host_import_blocks(&source);
        if import_blocks.is_empty() && !source.contains("crate::ui::slint_host::") {
            continue;
        }

        for forbidden in [
            "FrameRect",
            "PaneData",
            "TabData",
            "FloatingWindowData",
            "AssetsActivityShellFrameData",
            "AssetsActivityShellLayoutData",
            "AssetsActivityPaneData",
            "HierarchyShellFrameData",
            "HierarchyShellLayoutData",
            "HierarchyPaneData",
            "InspectorShellFrameData",
            "InspectorShellLayoutData",
            "InspectorPaneData",
            "ConsoleShellFrameData",
            "ConsoleShellLayoutData",
            "ConsolePaneData",
            "ProjectOverviewShellFrameData",
            "ProjectOverviewShellLayoutData",
            "ProjectOverviewPaneData",
            "AnimationEditorShellFrameData",
            "AnimationEditorShellLayoutData",
            "AnimationEditorPaneData",
            "UiAssetActionStateData",
            "UiAssetCanvasNodeData",
            "UiAssetCanvasSlotTargetData",
            "UiAssetCollectionPanelData",
            "UiAssetEditorPaneData",
            "UiAssetInspectorBindingData",
            "UiAssetInspectorLayoutData",
            "UiAssetInspectorPanelData",
            "UiAssetInspectorSemanticData",
            "UiAssetInspectorSlotData",
            "UiAssetInspectorWidgetData",
            "UiAssetMatchedStyleRuleData",
            "UiAssetPaletteDragData",
            "UiAssetPaneHeaderData",
            "UiAssetPreviewCanvasData",
            "UiAssetPreviewMockData",
            "UiAssetPreviewPanelData",
            "UiAssetSourceDetailData",
            "UiAssetSourcePanelData",
            "UiAssetStringSelectionData",
            "UiAssetStylePanelData",
            "UiAssetStyleRuleData",
            "UiAssetStyleRuleDeclarationData",
            "UiAssetStyleStateData",
            "UiAssetStyleTokenData",
            "UiAssetThemeSourceData",
            "HostWindowShellData",
            "HostWindowSurfaceData",
            "HostWindowLayoutData",
            "HostWindowSurfaceMetricsData",
            "HostWindowSurfaceOrchestrationData",
            "HostWindowSceneData",
            "HostNativeFloatingWindowSurfaceData",
            "ProjectOverviewData",
            "SceneNodeData",
        ] {
            for block in &import_blocks {
                assert!(
                    !block_imports_name(block, forbidden),
                    "expected {:?} to stop importing generated Slint host DTO `{forbidden}` into workbench_host_window internals",
                    path.file_name().expect("file name")
                );
            }
            assert!(
                !source.contains(&format!("crate::ui::slint_host::{forbidden}")),
                "expected {:?} to stop importing generated Slint host DTO `{forbidden}` into workbench_host_window internals",
                path.file_name().expect("file name")
            );
        }
    }

    assert!(
        apply_presentation.contains("fn to_slint_hierarchy_pane("),
        "expected apply_presentation.rs to own hierarchy pane conversion at the Slint boundary"
    );
    assert!(
        apply_presentation.contains("fn to_slint_inspector_pane("),
        "expected apply_presentation.rs to own inspector pane conversion at the Slint boundary"
    );
    assert!(
        apply_presentation.contains("fn to_slint_console_pane("),
        "expected apply_presentation.rs to own console pane conversion at the Slint boundary"
    );
    assert!(
        apply_presentation.contains("fn to_slint_assets_activity_pane("),
        "expected apply_presentation.rs to own assets-activity pane conversion at the Slint boundary"
    );
    assert!(
        apply_presentation.contains("fn to_slint_project_overview_pane("),
        "expected apply_presentation.rs to own project overview pane conversion at the Slint boundary"
    );
    assert!(
        apply_presentation.contains("fn to_slint_animation_editor_pane("),
        "expected apply_presentation.rs to own animation pane conversion at the Slint boundary"
    );
    assert!(
        apply_presentation.contains("fn to_slint_ui_asset_pane("),
        "expected apply_presentation.rs to own ui asset pane conversion at the Slint boundary"
    );
}
