use super::support::*;
use crate::ui::binding::{AssetCommand, EditorUiBindingPayload};
use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogGeneration, EditorAssetCatalogRecord, EditorAssetCatalogSnapshotRecord,
    EditorAssetFolderRecord,
};
use std::sync::Arc;

#[test]
fn u09_browser_new_folder_uses_the_published_foreground_name_prompt() {
    use crate::ui::retained_host::{
        hit_test_workbench_window_template_node_with_index, HostWorkbenchHitIndex,
    };
    use crate::ui::template_runtime::builtin::{
        ASSET_WINDOW_DOCUMENT_ID, UNREAL_EDITOR_WINDOW_DOCUMENT_ID,
    };

    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_editor_u09_browser_name_prompt");
    harness.activate_scene_for_editor09_close_fixture();
    crate::tests::support::configure_editor_test_runtime_build_set(&harness._core);
    let location = unique_temp_path("zircon_editor_u09_browser_name_project").with_extension("");
    fs::create_dir_all(&location).unwrap();
    let manager = harness
        ._core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let admitted = manager
        .create_project_and_open(crate::core::project::NewProjectDraft {
            project_name: "BrowserNames".to_owned(),
            location: location.to_string_lossy().into_owned(),
            template: crate::core::project::ProjectTemplateId::RenderableEmpty,
        })
        .expect("the name prompt uses a real admitted project generation");
    let project_root = admitted.project.as_ref().unwrap().root_path.clone();
    {
        let mut host = harness.host.borrow_mut();
        // Native test-host startup replaces the gateway after the harness's default scene
        // was constructed. Enter the admitted project from a current Welcome state.
        let state = EditorState::welcome_with_context(
            UVec2::new(1280, 720),
            host.startup_session.welcome_pane_snapshot(false),
            host.runtime.context().clone(),
        );
        host.runtime = Arc::new(
            crate::ui::host::EditorHostEventController::new(
                state,
                host.runtime.editor_manager_handle(),
            )
            .unwrap(),
        );
        host.apply_startup_session(admitted).unwrap();
    }
    harness.open_view("editor.asset_browser");
    {
        let mut host = harness.host.borrow_mut();
        assert!(host
            .active_activity_window_template_document_is(ASSET_WINDOW_DOCUMENT_ID)
            .unwrap());
        assert_eq!(
            host.workbench_window_bridge.host_projection().document_id,
            UNREAL_EDITOR_WINDOW_DOCUMENT_ID,
            "the host publishes one shared foreground popup surface on the Browser page"
        );
        assert!(host
            .runtime
            .editor_snapshot()
            .unwrap()
            .asset_browser
            .folder_tree
            .iter()
            .any(|folder| folder.folder_id == "res://"));
        host.open_asset_folder_context_menu("browser", "res://", false, 128.0, 256.0);
        host.recompute_if_dirty();
    }
    let menu = harness
        .root_ui
        .get_host_presentation()
        .workbench_window_nodes
        .iter()
        .find(|node| node.control_id.as_str() == "WorkbenchContextMenu")
        .cloned()
        .expect("the Browser context menu must be in the final native presentation");
    assert!(menu.popup_open);
    assert!(menu.source_surface_frame.is_some());
    pane_surface_host(&harness.root_ui).invoke_surface_control_clicked(
        "WorkbenchContextMenu".into(),
        "menu.item.asset.folder.create".into(),
    );
    {
        let mut host = harness.host.borrow_mut();
        assert!(
            host.asset_name_prompt.is_some(),
            "Browser New Folder must open its prompt: {}",
            host.runtime.editor_snapshot().unwrap().status_line
        );
        assert!(host
            .dispatch_componentized_workbench_surface_control("WorkbenchToolScale", "Tool/Scale")
            .is_none());
        host.recompute_if_dirty();
    }
    let name = "关卡 Names";
    pane_surface_host(&harness.root_ui).invoke_surface_control_edited(
        "WorkbenchCommandPalette".into(),
        "CommandPalette/QueryChanged".into(),
        name.into(),
    );
    {
        let mut host = harness.host.borrow_mut();
        host.recompute_if_dirty();
        assert_eq!(host.workbench_window_bridge.command_palette_query(), name);
    }
    let presentation = harness.root_ui.get_host_presentation();
    let palette = presentation
        .workbench_window_nodes
        .iter()
        .find(|node| node.control_id.as_str() == "WorkbenchCommandPalette")
        .expect("the name prompt must be in the final native Browser presentation");
    assert!(palette.popup_open);
    assert_eq!(palette.search_query.as_str(), name);
    assert!(palette.source_surface_frame.is_some());
    let index = HostWorkbenchHitIndex::from_presentation(&presentation);
    let option = (palette.frame.y.floor() as i32
        ..(palette.frame.y + palette.frame.height).ceil() as i32)
        .find_map(|y| {
            hit_test_workbench_window_template_node_with_index(
                &presentation,
                &index,
                palette.frame.x + palette.frame.width * 0.5,
                y as f32,
            )
            .filter(|hit| {
                hit.control_id.as_str() == "WorkbenchCommandPalette"
                    && hit.value_text.as_str() == "asset-name-prompt-confirm"
            })
        })
        .expect("the published name candidate must have a live native option hit");
    assert_eq!(option.commit_action_id.as_str(), "CommandPalette/Commit");
    assert!(!project_root.join("assets").join(name).exists());
    drop(presentation);
    pane_surface_host(&harness.root_ui).invoke_surface_control_clicked(
        "WorkbenchCommandPalette".into(),
        "workbench.popup.cancel".into(),
    );
    {
        let host = harness.host.borrow();
        assert!(host.asset_name_prompt.is_none());
        assert!(!host.workbench_window_bridge.command_palette_open());
    }
    harness
        .host
        .borrow_mut()
        .commit_project_close()
        .expect("host commit_project_close after U-09 browser fixture");
    drop(harness);
    drop(manager);
    fs::remove_dir_all(location).unwrap();
}

#[test]
fn u09_asset_popups_remain_refused_on_unrelated_activity_pages() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_editor_u09_popup_page_ownership");
    harness.open_view("editor.ui_component_showcase");
    let mut host = harness.host.borrow_mut();
    host.dispatch_workbench_context_menu_requested(WorkbenchContextMenuRequestData {
        target_path: "workbench://asset-folder/7265733a2f2f".into(),
        menu_items: vec!["New Folder|action=menu.item.asset.folder.create".into()],
        ..Default::default()
    });
    assert!(!workbench_control_bool(
        &host,
        "WorkbenchContextMenu",
        "popup_open"
    ));
    assert!(host
        .dispatch_componentized_workbench_surface_control(
            "WorkbenchContextMenu",
            "menu.item.asset.folder.create",
        )
        .is_none());
    assert!(host
        .dispatch_componentized_workbench_surface_control_edited(
            "WorkbenchCommandPalette",
            "CommandPalette/QueryChanged",
            "unused",
        )
        .is_none());
}

#[test]
fn u09_folder_background_and_tree_menus_retain_targets_without_changing_selection() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_editor_u09_folder_context_menus");
    harness.activate_workbench_page();
    let _browser = harness.open_view("editor.asset_browser");
    {
        let mut host = harness.host.borrow_mut();
        assert!(host
            .active_activity_window_template_document_is(
                crate::ui::template_runtime::builtin::ASSET_WINDOW_DOCUMENT_ID,
            )
            .unwrap());
        host.runtime.sync_asset_catalog(Arc::new(
            EditorAssetCatalogGeneration::from_snapshot_record(
                EditorAssetCatalogSnapshotRecord {
                    project_root: "E:/FolderMenus".to_owned(),
                    catalog_revision: 1,
                    folders: vec![
                        EditorAssetFolderRecord {
                            folder_id: "res://".to_owned(),
                            parent_folder_id: None,
                            locator_prefix: "res://".to_owned(),
                            display_name: "Assets".to_owned(),
                            child_folder_ids: vec!["res://empty".to_owned()],
                            direct_asset_uuids: Vec::new(),
                            recursive_asset_count: 0,
                        },
                        EditorAssetFolderRecord {
                            folder_id: "res://empty".to_owned(),
                            parent_folder_id: Some("res://".to_owned()),
                            locator_prefix: "res://empty".to_owned(),
                            display_name: "empty".to_owned(),
                            child_folder_ids: Vec::new(),
                            direct_asset_uuids: Vec::new(),
                            recursive_asset_count: 0,
                        },
                    ],
                    ..EditorAssetCatalogSnapshotRecord::default()
                },
                1,
            ),
        ));
        host.mark_layout_dirty();
        host.refresh_ui();
    }
    let pane = pane_surface_host(&harness.root_ui);
    pane.invoke_asset_content_pointer_event(
        "browser".into(),
        0,
        2,
        40.0,
        100.0,
        600.0,
        400.0,
        10.0,
        20.0,
    );
    {
        let host = harness.host.borrow();
        assert_eq!(
            host.workbench_window_bridge
                .context_menu_item_binding("WorkbenchContextMenu", "menu.item.asset.folder.create")
                .unwrap()
                .payload(),
            &EditorUiBindingPayload::AssetCommand(AssetCommand::CreateFolder {
                parent_folder_id: "res://".to_owned()
            })
        );
        assert_eq!(
            host.runtime
                .editor_snapshot()
                .unwrap()
                .asset_browser
                .selected_folder_id
                .as_deref(),
            Some("res://")
        );
    }
    pane.invoke_asset_tree_pointer_event(
        "browser".into(),
        0,
        2,
        40.0,
        100.0,
        240.0,
        400.0,
        500.0,
        100.0,
    );
    let host = harness.host.borrow();
    for (action, expected) in [
        (
            "menu.item.asset.folder.create",
            AssetCommand::CreateFolder {
                parent_folder_id: "res://empty".to_owned(),
            },
        ),
        (
            "menu.item.asset.folder.rename",
            AssetCommand::RenameFolder {
                folder_id: "res://empty".to_owned(),
            },
        ),
        (
            "menu.item.asset.folder.delete",
            AssetCommand::DeleteFolder {
                folder_id: "res://empty".to_owned(),
            },
        ),
    ] {
        assert_eq!(
            host.workbench_window_bridge
                .context_menu_item_binding("WorkbenchContextMenu", action)
                .unwrap()
                .payload(),
            &EditorUiBindingPayload::AssetCommand(expected)
        );
    }
    assert_eq!(
        host.runtime
            .editor_snapshot()
            .unwrap()
            .asset_browser
            .selected_folder_id
            .as_deref(),
        Some("res://")
    );
    assert!(host.active_asset_drag_payload.is_none());
    let menu_node = host
        .workbench_window_bridge
        .surface()
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata.as_ref().is_some_and(|metadata| {
                metadata.control_id.as_deref() == Some("WorkbenchContextMenu")
            })
        })
        .unwrap();
    let menu_items = menu_node
        .template_metadata
        .as_ref()
        .unwrap()
        .attributes
        .get("menu_items")
        .and_then(toml::Value::as_array)
        .unwrap();
    let delete_label = host
        .runtime
        .context()
        .i18n()
        .translate("menu.assets.delete_empty_folder.label");
    assert!(menu_items.iter().any(|value| value
        .as_str()
        .is_some_and(|item| item.starts_with(delete_label.as_ref())
            && item.contains("menu.item.asset.folder.delete"))));
    assert_eq!(
        host.workbench_window_bridge
            .surface()
            .input
            .popup_anchor_points
            .get(&menu_node.node_id)
            .copied(),
        Some(UiPoint::new(540.0, 200.0))
    );
    assert_eq!(
        menu_node
            .template_metadata
            .as_ref()
            .unwrap()
            .attributes
            .get("popup_owner")
            .and_then(toml::Value::as_str),
        Some("WorkbenchWindowRoot")
    );
    let root_node_id = host
        .workbench_window_bridge
        .surface()
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata.as_ref().is_some_and(|metadata| {
                metadata.control_id.as_deref() == Some("WorkbenchWindowRoot")
            })
        })
        .unwrap()
        .node_id;
    let popup = host
        .workbench_window_bridge
        .surface()
        .input
        .popup_stack
        .iter()
        .find(|popup| popup.popup_node == Some(menu_node.node_id))
        .expect("the native asset folder menu must retain its runtime popup stack entry");
    assert_eq!(popup.owner, Some(root_node_id));
    assert_eq!(popup.anchor, Some(UiPoint::new(540.0, 200.0)));
    assert!(
        host.workbench_window_bridge
            .host_projection()
            .node_by_control_id("WorkbenchContextMenu")
            .unwrap()
            .popup_open
    );
}

#[test]
fn u09_project_subasset_context_menu_reimports_owning_source_without_renaming_subasset() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_editor_u09_subasset_reimport_menu");
    harness.activate_workbench_page();
    let _browser = harness.open_view("editor.asset_browser");
    let asset_uuid = "00112233-4455-6677-8899-aabbccddeeff";
    {
        let mut host = harness.host.borrow_mut();
        assert!(host
            .active_activity_window_template_document_is(
                crate::ui::template_runtime::builtin::ASSET_WINDOW_DOCUMENT_ID,
            )
            .unwrap());
        host.runtime.sync_asset_catalog(Arc::new(
            EditorAssetCatalogGeneration::from_snapshot_record(
                EditorAssetCatalogSnapshotRecord {
                    project_root: "E:/SubassetMenu".to_owned(),
                    catalog_revision: 1,
                    folders: vec![EditorAssetFolderRecord {
                        folder_id: "res://".to_owned(),
                        parent_folder_id: None,
                        locator_prefix: "res://".to_owned(),
                        display_name: "Assets".to_owned(),
                        child_folder_ids: Vec::new(),
                        direct_asset_uuids: vec![asset_uuid.to_owned()],
                        recursive_asset_count: 1,
                    }],
                    assets: vec![EditorAssetCatalogRecord {
                        uuid: asset_uuid.to_owned(),
                        id: "mesh-1".to_owned(),
                        locator: "res://ship.glb#mesh=1".to_owned(),
                        kind: zircon_runtime_interface::resource::ResourceKind::Mesh,
                        display_name: "Ship mesh".to_owned(),
                        file_name: "ship.glb".to_owned(),
                        extension: "glb".to_owned(),
                        preview_state: zircon_runtime::asset::project::PreviewState::Dirty,
                        meta_path: String::new(),
                        preview_artifact_path: String::new(),
                        source_mtime_unix_ms: 0,
                        source_hash: String::new(),
                        dirty: false,
                        diagnostics: Vec::new(),
                        direct_reference_uuids: Vec::new(),
                    }],
                    ..EditorAssetCatalogSnapshotRecord::default()
                },
                1,
            ),
        ));
        host.mark_layout_dirty();
        host.refresh_ui();
    }
    pane_surface_host(&harness.root_ui).invoke_asset_content_pointer_event(
        "browser".into(),
        0,
        2,
        96.0,
        96.0,
        600.0,
        400.0,
        0.0,
        0.0,
    );
    let host = harness.host.borrow();
    let menu_items = host
        .workbench_window_bridge
        .surface()
        .tree
        .nodes
        .values()
        .find_map(|node| {
            let metadata = node.template_metadata.as_ref()?;
            (metadata.control_id.as_deref() == Some("WorkbenchContextMenu"))
                .then(|| metadata.attributes.get("menu_items"))
                .flatten()
                .and_then(toml::Value::as_array)
        })
        .unwrap();
    assert!(menu_items.iter().any(|value| value
        .as_str()
        .is_some_and(|item| item.contains("action=menu.item.asset.reimport,"))));
    assert!(!menu_items.iter().any(|value| value
        .as_str()
        .is_some_and(|item| item.contains("action=menu.item.asset.rename,"))));
    assert_eq!(
        host.workbench_window_bridge
            .context_menu_item_binding("WorkbenchContextMenu", "menu.item.asset.reimport")
            .unwrap()
            .payload(),
        &EditorUiBindingPayload::AssetCommand(AssetCommand::ReimportAsset {
            asset_uuid: asset_uuid.to_owned(),
        })
    );
}

struct U09CancelledAssetMutation;

impl crate::core::jobs::EditorJob for U09CancelledAssetMutation {
    type Output = crate::core::asset::EditorAssetDeletionResult;

    fn run(
        self,
        _context: crate::core::jobs::JobContext,
    ) -> Result<Self::Output, crate::core::jobs::JobError> {
        Err(crate::core::jobs::JobError::Cancelled)
    }
}

#[test]
fn u09_pending_asset_mutation_blocks_scene_open_and_existing_picker_commit() {
    use super::super::scene_picker_session::{ScenePickerMode, ScenePickerSession};

    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_editor_u09_scene_mutation_guard");
    let lifecycle = crate::core::document::DocumentLifecycleAuthority::default();
    lifecycle.begin_project_session(&harness.config_path);
    let ticket = lifecycle
        .issue_scene_picker_ticket(&harness.config_path)
        .unwrap();
    let session = ScenePickerSession::new(
        ticket,
        ScenePickerMode::Create,
        &EditorAssetCatalogGeneration::default(),
    );
    let state = session.command_palette_state("res://new.scene.toml", 0, false);
    let mut host = harness.host.borrow_mut();
    assert!(host
        .workbench_window_bridge
        .open_command_palette_with_chrome(
            state,
            ScenePickerMode::Create.command_source(),
            ScenePickerMode::Create.placeholder(),
            ScenePickerMode::Create.empty_text(),
            ScenePickerMode::Create.accessibility_label(),
            ScenePickerMode::Create.empty_text(),
        )
        .unwrap());
    host.scene_picker_session = Some(session);
    let mutation = host
        .runtime
        .context()
        .jobs()
        .submit(
            crate::core::jobs::EditorJobSpec::new(
                "U09 pending asset mutation scene guard",
                crate::core::jobs::JobCategory::Misc,
            ),
            U09CancelledAssetMutation,
        )
        .unwrap();
    host.install_pending_asset_deletion_for_editor09_close_test(
        crate::core::asset::EditorAssetDeletionTicket::new(mutation, Default::default()),
    );

    let blocked = host
        .open_startup_scene(
            zircon_runtime::asset::AssetUri::parse("res://main.scene.toml").unwrap(),
        )
        .unwrap_err();
    assert!(blocked.contains("asset operation is already running"));
    host.open_workbench_scene_picker(ScenePickerMode::Open);
    assert_eq!(
        host.scene_picker_session.as_ref().unwrap().mode(),
        ScenePickerMode::Create
    );
    let result = host
        .dispatch_workbench_scene_picker_committed(
            "WorkbenchCommandPalette",
            "CommandPalette/Commit",
            "scene-picker-create-confirm",
        )
        .expect("the existing scene picker still owns the binding");
    assert_eq!(result.unwrap_err(), blocked);
    assert!(host.workbench_window_bridge.command_palette_open());
    assert_eq!(
        host.workbench_window_bridge.command_palette_query(),
        "res://new.scene.toml"
    );
}
