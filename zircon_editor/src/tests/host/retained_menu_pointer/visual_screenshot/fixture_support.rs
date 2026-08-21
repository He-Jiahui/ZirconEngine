use super::*;

fn visual_layout_output_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("editor crate should live under the repository root")
        .join("docs")
        .join("tests")
        .join("editor")
}

pub(super) fn visual_layout_output_path(filename: &str) -> PathBuf {
    let output_dir = visual_layout_output_dir();
    std::fs::create_dir_all(&output_dir).expect("visual-layout output directory should exist");
    output_dir.join(filename)
}

pub(super) fn host_window_layout_for_visual_artifact(
    width: f32,
    height: f32,
) -> HostWindowLayoutData {
    HostWindowLayoutData {
        center_band_frame: frame(0.0, 38.0, width, height - 62.0),
        status_bar_frame: frame(0.0, height - 24.0, width, 24.0),
        left_region_frame: frame(0.0, 38.0, 198.0, height - 62.0),
        document_region_frame: frame(198.0, 38.0, width - 198.0, height - 62.0),
        viewport_content_frame: frame(214.0, 66.0, width - 230.0, height - 118.0),
        ..HostWindowLayoutData::default()
    }
}

pub(super) fn nested_menu_chrome_for_visual_artifact() -> HostMenuChromeData {
    HostMenuChromeData {
        top_bar_height_px: 25.0,
        menu_frames: crate::ui::layouts::common::model_rc(vec![HostChromeControlFrameData {
            control_id: "MenuSlotTools".into(),
            frame: frame(72.0, 2.0, 64.0, 22.0),
        }]),
        menus: crate::ui::layouts::common::model_rc(vec![HostMenuChromeMenuData {
            label: "Tools".into(),
            popup_width_px: 184.0,
            popup_height_px: 92.0,
            items: crate::ui::layouts::common::model_rc(vec![
                HostMenuChromeItemData {
                    label: "Weather".into(),
                    shortcut: ">".into(),
                    enabled: true,
                    children: crate::ui::layouts::common::model_rc(vec![
                        HostMenuChromeItemData {
                            label: "Refresh Clouds".into(),
                            action_id: "weather.cloud_layer.refresh".into(),
                            shortcut: "Ctrl+Alt+R".into(),
                            enabled: true,
                            ..HostMenuChromeItemData::default()
                        },
                        HostMenuChromeItemData {
                            label: "Bake Probe Preview".into(),
                            action_id: "weather.probe.bake_preview".into(),
                            enabled: true,
                            ..HostMenuChromeItemData::default()
                        },
                    ]),
                    ..HostMenuChromeItemData::default()
                },
                HostMenuChromeItemData {
                    label: "Diagnostics".into(),
                    action_id: "tools.diagnostics.open".into(),
                    shortcut: "Ctrl+Shift+D".into(),
                    enabled: true,
                    ..HostMenuChromeItemData::default()
                },
            ]),
            popup_nodes: crate::ui::layouts::common::model_rc(vec![
                template_node("NestedMenuPopupPanel", "Panel", "", 0.0, 0.0, 184.0, 92.0),
                template_node(
                    "NestedMenuPopupItem0",
                    "Panel",
                    "Weather",
                    6.0,
                    6.0,
                    172.0,
                    26.0,
                ),
                template_node(
                    "NestedMenuPopupItem1",
                    "Panel",
                    "Diagnostics",
                    6.0,
                    36.0,
                    172.0,
                    26.0,
                ),
            ]),
        }]),
        ..HostMenuChromeData::default()
    }
}

fn template_node(
    control_id: &str,
    role: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: format!("{control_id}.node").into(),
        control_id: control_id.into(),
        role: role.into(),
        text: text.into(),
        surface_variant: "panel".into(),
        border_width: 1.0,
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

pub(super) fn window_menu_preset_names() -> Vec<String> {
    (0..24).map(|index| format!("Preset {index:02}")).collect()
}

pub(super) fn m3_asset_workspace() -> AssetWorkspaceSnapshot {
    AssetWorkspaceSnapshot {
        project_name: "Zircon M3 Visual".to_string(),
        project_root: "E:/Git/ZirconEngine".to_string(),
        assets_root: "zircon_editor/assets".to_string(),
        cache_root: "zircon_runtime/.zircon/cache".to_string(),
        default_scene_uri: "res://scenes/editor_preview.zscene".to_string(),
        catalog_revision: 42,
        view_mode: AssetViewMode::Thumbnail,
        utility_tab: AssetUtilityTab::Preview,
        search_query: "workbench".to_string(),
        folder_tree: m3_asset_folders(),
        visible_folders: m3_asset_folders(),
        visible_assets: vec![
            asset_item(
                "asset-ui-layout",
                "res://ui/editor/workbench_page_chrome.zui",
                "workbench_page_chrome.zui",
                "zui",
                ResourceKind::UiLayout,
                true,
            ),
            asset_item(
                "asset-theme-base",
                "res://ui/theme/editor_base.zui",
                "editor_base.zui",
                "zui",
                ResourceKind::UiStyle,
                false,
            ),
            asset_item(
                "asset-folder-open-svg",
                "res://icons/ionicons/folder-open-outline.svg",
                "folder-open-outline.svg",
                "svg",
                ResourceKind::Texture,
                false,
            ),
            asset_item(
                "asset-accessibility-audit",
                "res://ui/editor/components/workbench/modules/extensions/ui/workbench_extension_accessibility_workspace.zui",
                "workbench_extension_accessibility_workspace.zui",
                "zui",
                ResourceKind::UiWidget,
                false,
            ),
            asset_item(
                "asset-material-workspace",
                "res://ui/editor/components/workbench/modules/core/rendering/workbench_material_workspace.zui",
                "workbench_material_workspace.zui",
                "zui",
                ResourceKind::MaterialGraph,
                false,
            ),
            asset_item(
                "asset-scene-preview",
                "res://scenes/editor_preview.zscene",
                "editor_preview.zscene",
                "zscene",
                ResourceKind::Scene,
                false,
            ),
            asset_item(
                "asset-shader-unlit",
                "res://shaders/ui/unlit.zshader",
                "unlit.zshader",
                "zshader",
                ResourceKind::Shader,
                false,
            ),
            asset_item(
                "asset-player-prefab",
                "res://prefabs/player_start.prefab",
                "player_start.prefab",
                "prefab",
                ResourceKind::Prefab,
                false,
            ),
        ],
        selected_folder_id: Some("folder-ui".to_string()),
        selected_asset_uuid: Some("asset-ui-layout".to_string()),
        selection: AssetSelectionSnapshot {
            uuid: Some("asset-ui-layout".to_string()),
            display_name: "workbench_page_chrome.zui".to_string(),
            locator: "res://ui/editor/workbench_page_chrome.zui".to_string(),
            kind: Some(ResourceKind::UiLayout),
            asset_type:
                crate::ui::workbench::snapshot::AssetTypeProjectionSnapshot::from_resource_kind(
                    ResourceKind::UiLayout,
                ),
            preview_artifact_path: "docs/tests/editor/editor-window-m3-workbench-900x620.png"
                .to_string(),
            meta_path: "zircon_editor/assets/ui/editor/workbench_page_chrome.zui".to_string(),
            toolkit_view_id: "editor.ui_asset".to_string(),
            toolkit_open_operation: "view.editor.ui_asset.open".to_string(),
            context_commands: Vec::new(),
            package_id: Some("zircon.editor.ui".to_string()),
            asset_unit: "single".to_string(),
            included_files: vec![
                "zircon_editor/assets/ui/editor/workbench_page_chrome.zui".to_string(),
                "zircon_editor/assets/ui/editor/asset_browser.zui".to_string(),
                "zircon_editor/assets/ui/editor/theme/editor_tokens.zui".to_string(),
            ],
            subassets: vec![
                asset_subasset(
                    "subasset-content-table",
                    "res://ui/editor/asset_browser.zui#AssetBrowserAssetTablePanel",
                    ResourceKind::UiWidget,
                ),
                asset_subasset(
                    "subasset-preview-card",
                    "res://ui/editor/asset_browser.zui#AssetBrowserContentPreviewCard",
                    ResourceKind::UiWidget,
                ),
            ],
            diagnostics: vec![
                "SVG icons resolve through scalable template metadata.".to_string(),
                "Retained-host content table uses workbench table row painter.".to_string(),
            ],
            resource_state: Some(ResourceState::Ready),
            resource_revision: Some(42),
            references: vec![
                asset_reference(
                    "ref-editor-base",
                    "res://ui/theme/editor_base.zui",
                    "editor_base.zui",
                    ResourceKind::UiStyle,
                ),
                asset_reference(
                    "ref-editor-material",
                    "res://ui/theme/editor_material.zui",
                    "editor_material.zui",
                    ResourceKind::UiStyle,
                ),
            ],
            used_by: vec![
                asset_reference(
                    "used-asset-browser",
                    "res://ui/editor/asset_browser.zui",
                    "Asset Browser",
                    ResourceKind::UiLayout,
                ),
                asset_reference(
                    "used-workbench-shell",
                    "res://ui/editor/host/workbench_shell.zui",
                    "Workbench Shell",
                    ResourceKind::UiLayout,
                ),
            ],
        },
        ..AssetWorkspaceSnapshot::default()
    }
}

fn m3_asset_folders() -> Vec<AssetFolderSnapshot> {
    vec![
        AssetFolderSnapshot {
            folder_id: "folder-assets".to_string(),
            parent_folder_id: None,
            display_name: "Assets".to_string(),
            recursive_asset_count: 6,
            depth: 0,
            selected: false,
        },
        AssetFolderSnapshot {
            folder_id: "folder-ui".to_string(),
            parent_folder_id: Some("folder-assets".to_string()),
            display_name: "ui".to_string(),
            recursive_asset_count: 4,
            depth: 1,
            selected: true,
        },
        AssetFolderSnapshot {
            folder_id: "folder-icons".to_string(),
            parent_folder_id: Some("folder-assets".to_string()),
            display_name: "icons".to_string(),
            recursive_asset_count: 1,
            depth: 1,
            selected: false,
        },
        AssetFolderSnapshot {
            folder_id: "folder-workbench".to_string(),
            parent_folder_id: Some("folder-ui".to_string()),
            display_name: "workbench".to_string(),
            recursive_asset_count: 4,
            depth: 2,
            selected: false,
        },
    ]
}

fn asset_item(
    uuid: &str,
    locator: &str,
    file_name: &str,
    extension: &str,
    kind: ResourceKind,
    selected: bool,
) -> AssetItemSnapshot {
    AssetItemSnapshot {
        uuid: uuid.to_string(),
        locator: locator.to_string(),
        display_name: file_name.to_string(),
        file_name: file_name.to_string(),
        extension: extension.to_string(),
        kind,
        asset_type: crate::ui::workbench::snapshot::AssetTypeProjectionSnapshot::from_resource_kind(
            kind,
        ),
        preview_artifact_path: String::new(),
        dirty: false,
        diagnostics: Vec::new(),
        selected,
        resource_state: Some(ResourceState::Ready),
        resource_revision: Some(42),
    }
}

fn asset_reference(
    uuid: &str,
    locator: &str,
    display_name: &str,
    kind: ResourceKind,
) -> AssetReferenceSnapshot {
    AssetReferenceSnapshot {
        uuid: uuid.to_string(),
        locator: locator.to_string(),
        display_name: display_name.to_string(),
        kind: Some(kind),
        asset_type: Some(AssetTypeProjectionSnapshot::from_resource_kind(kind)),
        known_project_asset: true,
    }
}

fn asset_subasset(uuid: &str, locator: &str, kind: ResourceKind) -> AssetSubassetSnapshot {
    AssetSubassetSnapshot {
        uuid: uuid.to_string(),
        locator: locator.to_string(),
        kind,
        asset_type: AssetTypeProjectionSnapshot::from_resource_kind(kind),
        artifact_locator: Some(locator.to_string()),
        dependency_locators: Vec::new(),
    }
}

pub(super) fn frame(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}
