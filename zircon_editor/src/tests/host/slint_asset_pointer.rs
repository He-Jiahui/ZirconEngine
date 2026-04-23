use crate::core::editor_event::{EditorAssetEvent, EditorEvent};
use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogRecord, EditorAssetCatalogSnapshotRecord, EditorAssetFolderRecord,
};
use crate::ui::slint_host::asset_pointer::{
    AssetContentListPointerBridge, AssetContentListPointerLayout, AssetFolderTreePointerBridge,
    AssetFolderTreePointerLayout, AssetListPointerState, AssetListViewMode,
    AssetPointerContentRoute, AssetPointerReferenceRoute, AssetPointerTreeRoute,
    AssetReferenceListPointerBridge, AssetReferenceListPointerEntry,
    AssetReferenceListPointerLayout,
};
use crate::ui::slint_host::callback_dispatch::{
    dispatch_builtin_asset_surface_control, dispatch_shared_asset_content_pointer_click,
    dispatch_shared_asset_reference_pointer_click, dispatch_shared_asset_tree_pointer_click,
    BuiltinAssetSurfaceTemplateBridge,
};
use zircon_runtime::asset::project::PreviewState;
use zircon_runtime::core::resource::ResourceKind;
use zircon_runtime::ui::{
    binding::UiBindingValue, binding::UiEventKind, layout::UiPoint, layout::UiSize,
};

#[test]
fn shared_asset_tree_pointer_bridge_scrolls_and_dispatches_folder_selection() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_asset_tree_pointer");
    let bridge = BuiltinAssetSurfaceTemplateBridge::new()
        .expect("builtin asset surface bridge should build");
    harness.runtime.sync_asset_catalog(sample_catalog());
    let snapshot = harness.runtime.editor_snapshot();
    let base_folder_ids = snapshot
        .asset_browser
        .folder_tree
        .iter()
        .map(|folder| folder.folder_id.clone())
        .collect::<Vec<_>>();
    assert!(
        !base_folder_ids.is_empty(),
        "default fixture should expose asset folders"
    );

    let folder_ids = repeated_ids(&base_folder_ids, 12);
    let mut pointer_bridge = AssetFolderTreePointerBridge::new();
    pointer_bridge.sync(
        AssetFolderTreePointerLayout {
            pane_size: UiSize::new(240.0, 200.0),
            folder_ids: folder_ids.clone(),
        },
        AssetListPointerState::default(),
    );

    let scrolled = pointer_bridge
        .handle_scroll(UiPoint::new(120.0, 88.0), 44.0)
        .expect("asset tree should accept shared scroll input");
    assert!(scrolled.state.scroll_offset > 0.0);

    pointer_bridge.sync(
        AssetFolderTreePointerLayout {
            pane_size: UiSize::new(240.0, 200.0),
            folder_ids: folder_ids.clone(),
        },
        scrolled.state.clone(),
    );
    let row_index = 4usize;
    let click_y = 49.0 + 8.0 + row_index as f32 * 32.0 - scrolled.state.scroll_offset + 14.0;
    let dispatched = dispatch_shared_asset_tree_pointer_click(
        &harness.runtime,
        &bridge,
        &mut pointer_bridge,
        UiPoint::new(72.0, click_y),
    )
    .expect("shared asset tree route should dispatch folder selection");
    assert_eq!(
        dispatched.pointer.route,
        Some(AssetPointerTreeRoute::Folder {
            row_index,
            folder_id: folder_ids[row_index].clone(),
        })
    );
    let effects = dispatched
        .effects
        .expect("folder selection should dispatch through runtime");
    assert!(effects.presentation_dirty);
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Asset(EditorAssetEvent::SelectFolder {
            folder_id: folder_ids[row_index].clone(),
        })
    );
}

#[test]
fn shared_asset_content_pointer_bridge_scrolls_and_dispatches_item_selection() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_asset_content_pointer");
    let bridge = BuiltinAssetSurfaceTemplateBridge::new()
        .expect("builtin asset surface bridge should build");
    harness.runtime.sync_asset_catalog(sample_catalog());
    dispatch_builtin_asset_surface_control(
        &harness.runtime,
        &bridge,
        "SelectFolder",
        UiEventKind::Change,
        vec![UiBindingValue::string("res://materials")],
    )
    .expect("asset folder control should resolve")
    .expect("asset folder selection should dispatch");
    let snapshot = harness.runtime.editor_snapshot();
    let base_asset_ids = snapshot
        .asset_browser
        .visible_assets
        .iter()
        .map(|asset| asset.uuid.clone())
        .collect::<Vec<_>>();
    assert!(
        !base_asset_ids.is_empty(),
        "default fixture should expose visible asset items"
    );

    let asset_ids = repeated_ids(&base_asset_ids, 12);
    let mut pointer_bridge = AssetContentListPointerBridge::new();
    pointer_bridge.sync(
        AssetContentListPointerLayout {
            pane_size: UiSize::new(420.0, 220.0),
            view_mode: AssetListViewMode::List,
            folder_ids: Vec::new(),
            item_ids: asset_ids.clone(),
        },
        AssetListPointerState::default(),
    );

    let scrolled = pointer_bridge
        .handle_scroll(UiPoint::new(180.0, 100.0), 64.0)
        .expect("asset content list should accept shared scroll input");
    assert!(scrolled.state.scroll_offset > 0.0);

    pointer_bridge.sync(
        AssetContentListPointerLayout {
            pane_size: UiSize::new(420.0, 220.0),
            view_mode: AssetListViewMode::List,
            folder_ids: Vec::new(),
            item_ids: asset_ids.clone(),
        },
        scrolled.state.clone(),
    );
    let row_index = 3usize;
    let click_y = 53.0 + 8.0 + row_index as f32 * 46.0 - scrolled.state.scroll_offset + 18.0;
    let dispatched = dispatch_shared_asset_content_pointer_click(
        &harness.runtime,
        &bridge,
        &mut pointer_bridge,
        UiPoint::new(148.0, click_y),
    )
    .expect("shared asset content route should dispatch item selection");
    assert_eq!(
        dispatched.pointer.route,
        Some(AssetPointerContentRoute::Item {
            row_index,
            item_index: row_index,
            asset_uuid: asset_ids[row_index].clone(),
        })
    );
    let effects = dispatched
        .effects
        .expect("item selection should dispatch through runtime");
    assert!(effects.presentation_dirty);
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Asset(EditorAssetEvent::SelectItem {
            asset_uuid: asset_ids[row_index].clone(),
        })
    );
}

#[test]
fn shared_asset_reference_pointer_bridge_scrolls_and_dispatches_reference_activation() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_asset_reference_pointer");
    let bridge = BuiltinAssetSurfaceTemplateBridge::new()
        .expect("builtin asset surface bridge should build");
    harness.runtime.sync_asset_catalog(sample_catalog());

    let asset_ids = vec![
        "11111111-1111-1111-1111-111111111111".to_string(),
        "22222222-2222-2222-2222-222222222222".to_string(),
    ];
    let reference_entries = repeated_reference_entries(&asset_ids, 12);
    let mut pointer_bridge = AssetReferenceListPointerBridge::new();
    pointer_bridge.sync(
        AssetReferenceListPointerLayout {
            pane_size: UiSize::new(280.0, 180.0),
            entries: reference_entries.clone(),
        },
        AssetListPointerState::default(),
    );

    let scrolled = pointer_bridge
        .handle_scroll(UiPoint::new(120.0, 84.0), 44.0)
        .expect("asset reference list should accept shared scroll input");
    assert!(scrolled.state.scroll_offset > 0.0);

    pointer_bridge.sync(
        AssetReferenceListPointerLayout {
            pane_size: UiSize::new(280.0, 180.0),
            entries: reference_entries.clone(),
        },
        scrolled.state.clone(),
    );
    let row_index = 4usize;
    let click_y = 20.0 + row_index as f32 * 38.0 - scrolled.state.scroll_offset + 17.0;
    let dispatched = dispatch_shared_asset_reference_pointer_click(
        &harness.runtime,
        &bridge,
        &mut pointer_bridge,
        UiPoint::new(96.0, click_y),
    )
    .expect("shared asset reference route should dispatch reference activation");
    assert_eq!(
        dispatched.pointer.route,
        Some(AssetPointerReferenceRoute::Item {
            row_index,
            asset_uuid: reference_entries[row_index].asset_uuid.clone(),
        })
    );
    let effects = dispatched
        .effects
        .expect("reference activation should dispatch through runtime");
    assert!(effects.presentation_dirty);
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Asset(EditorAssetEvent::ActivateReference {
            asset_uuid: reference_entries[row_index].asset_uuid.clone(),
        })
    );
}

#[test]
fn asset_surface_controls_use_generic_template_callbacks_instead_of_legacy_business_abi() {
    let workbench = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/workbench.slint"));
    let pane_surface = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/pane_surface.slint"
    ));
    let pane_content = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/pane_content.slint"
    ));
    let template_pane = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/template_pane.slint"
    ));
    let pane_surface_host_context = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/pane_surface_host_context.slint"
    ));
    let assets = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/assets.slint"
    ));
    let wiring = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/callback_wiring.rs"
    ));
    let app_assets = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/assets.rs"
    ));
    let pane_content_normalized = pane_content.split_whitespace().collect::<String>();
    let template_pane_normalized = template_pane.split_whitespace().collect::<String>();

    for needle in [
        "callback asset_search_edited(",
        "callback asset_kind_filter_changed(",
        "callback asset_view_mode_changed(",
        "callback asset_utility_tab_changed(",
        "callback open_asset_browser(",
        "callback locate_selected_asset(",
        "callback import_model(",
        "asset_search_edited(value) =>",
        "asset_kind_filter_changed(kind) =>",
        "asset_view_mode_changed(",
        "asset_utility_tab_changed(",
        "open_asset_browser() =>",
        "locate_selected_asset() =>",
        "import_model() =>",
    ] {
        assert!(
            !workbench.contains(needle),
            "workbench shell still exposes legacy asset business callback `{needle}`"
        );
    }

    for needle in [
        "callback asset_control_changed(source: string, control_id: string, value: string);",
        "callback asset_control_clicked(source: string, control_id: string);",
    ] {
        assert!(
            !pane_surface.contains(needle),
            "pane surface should not keep dead asset callback bridge `{needle}`"
        );
    }

    for needle in [
        "callback search_edited(value: string);",
        "callback kind_filter_changed(kind: string);",
        "callback view_mode_changed(mode: string);",
        "callback utility_tab_changed(tab: string);",
        "callback open_asset_browser();",
        "callback locate_selected();",
        "callback import_model();",
        "edited(value) => { root.search_edited(value); }",
        "clicked => { root.kind_filter_changed(\"Texture\"); }",
        "clicked => { root.view_mode_changed(\"list\"); }",
        "clicked => { root.utility_tab_changed(\"preview\"); }",
        "clicked => { root.open_asset_browser(); }",
        "clicked => { root.locate_selected(); }",
        "clicked => { root.import_model(); }",
    ] {
        assert!(
            !assets.contains(needle),
            "asset leaf surfaces still expose legacy direct control callback `{needle}`"
        );
    }

    for needle in [
        "ui.on_asset_search_edited(",
        "ui.on_asset_kind_filter_changed(",
        "ui.on_asset_view_mode_changed(",
        "ui.on_asset_utility_tab_changed(",
        "ui.on_open_asset_browser(",
        "ui.on_locate_selected_asset(",
        "ui.on_import_model(",
    ] {
        assert!(
            !wiring.contains(needle),
            "slint host wiring still registers legacy asset control callback `{needle}`"
        );
    }

    for needle in [
        "fn update_asset_search(",
        "fn update_asset_kind_filter(",
        "fn update_asset_view_mode(",
        "fn update_asset_utility_tab(",
        "fn open_asset_browser(",
        "fn locate_selected_asset(",
    ] {
        assert!(
            !app_assets.contains(needle),
            "asset host helper file still carries legacy business helper `{needle}`"
        );
    }

    for needle in [
        "callback asset_control_changed(source: string, control_id: string, value: string);",
        "callback asset_control_clicked(source: string, control_id: string);",
    ] {
        assert!(
            pane_surface_host_context.contains(needle),
            "pane surface host context is missing generic asset control callback `{needle}`"
        );
    }

    for needle in [
        "control_changed(control_id, value) => { PaneSurfaceHostContext.asset_control_changed(\"activity\", control_id, value); }",
        "control_clicked(control_id) => { PaneSurfaceHostContext.asset_control_clicked(\"activity\", control_id); }",
        "control_changed(control_id, value) => { PaneSurfaceHostContext.asset_control_changed(\"browser\", control_id, value); }",
        "control_clicked(control_id) => { PaneSurfaceHostContext.asset_control_clicked(\"browser\", control_id); }",
    ] {
        assert!(
            pane_content.contains(needle),
            "pane content is missing generic asset control route `{needle}`"
        );
    }
    for needle in [
        "exportcomponentTemplatePaneinheritsRectangle{",
        "callbacknode_dispatched(control_id:string,dispatch_kind:string,action_id:string);",
        "clicked=>{root.node_dispatched(root.node.control_id,root.node.dispatch_kind,root.node.action_id",
    ] {
        assert!(
            template_pane_normalized.contains(needle),
            "template_pane.slint is missing generic template dispatch route `{needle}`"
        );
    }
    for needle in [
        "if!root.pane.show_empty&&root.pane.kind==\"Project\":TemplatePane{",
        "nodes:root.pane.project_overview.nodes;",
        "node_dispatched(control_id,dispatch_kind,action_id)=>{",
        "PaneSurfaceHostContext.asset_control_clicked(\"project\",control_id);",
    ] {
        assert!(
            pane_content_normalized.contains(needle),
            "pane content is missing projected project template route `{needle}`"
        );
    }

    for needle in [
        "callback control_changed(control_id: string, value: string);",
        "callback control_clicked(control_id: string);",
        "edited(value) => { root.control_changed(\"SearchEdited\", value); }",
        "clicked => { root.control_changed(\"SetKindFilter\", \"Texture\"); }",
        "clicked => { root.control_changed(\"SetViewMode\", \"list\"); }",
        "clicked => { root.control_changed(\"SetUtilityTab\", \"preview\"); }",
        "clicked => { root.control_clicked(\"OpenAssetBrowser\"); }",
        "clicked => { root.control_clicked(\"LocateSelectedAsset\"); }",
        "clicked => { root.control_clicked(\"ImportModel\"); }",
    ] {
        assert!(
            pane_content.contains(needle),
            "pane_content is missing generic asset control route `{needle}`"
        );
    }

    for needle in [
        "pane_surface_host.on_asset_control_changed(",
        "pane_surface_host.on_asset_control_clicked(",
    ] {
        assert!(
            wiring.contains(needle),
            "slint host wiring is missing generic asset control callback `{needle}`"
        );
    }
}

#[test]
fn asset_surface_templates_expose_physics_and_animation_kind_filters() {
    let pane_content = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/pane_content.slint"
    ));

    for kind in [
        "PhysicsMaterial",
        "AnimationSkeleton",
        "AnimationClip",
        "AnimationSequence",
        "AnimationGraph",
        "AnimationStateMachine",
    ] {
        let needle = format!("root.control_changed(\"SetKindFilter\", \"{kind}\")");
        assert_eq!(
            pane_content.matches(&needle).count(),
            2,
            "asset activity/browser surfaces should both expose {kind} kind filters"
        );
    }
}

#[test]
fn asset_surface_templates_map_no_preview_physics_and_animation_assets_to_specific_icons() {
    let assets = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/assets.slint"
    ));
    let chrome = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/chrome.slint"
    ));

    for (kind, icon_key) in [
        ("PhysicsMaterial", "physics-material"),
        ("AnimationSkeleton", "hierarchy"),
        ("AnimationClip", "tool-play"),
        ("AnimationSequence", "animation-sequence"),
        ("AnimationGraph", "animation-graph"),
        ("AnimationStateMachine", "animation-graph"),
    ] {
        let item_needle = format!("item.kind == \"{kind}\" ? \"{icon_key}\"");
        assert_eq!(
            assets.matches(&item_needle).count(),
            2,
            "asset item thumbnail/list fallbacks should classify {kind} with {icon_key}"
        );

        let selection_needle = format!("root.selection.kind == \"{kind}\" ? \"{icon_key}\"");
        assert_eq!(
            assets.matches(&selection_needle).count(),
            2,
            "selection preview/details fallbacks should classify {kind} with {icon_key}"
        );
    }

    for (icon_key, icon_svg) in [
        ("physics-material", "construct-outline.svg"),
        ("animation-sequence", "play-outline.svg"),
        ("animation-graph", "grid-outline.svg"),
    ] {
        let needle = format!(
            "icon_key == \"{icon_key}\" ? @image-url(\"../../assets/icons/ionicons/{icon_svg}\")"
        );
        assert!(
            chrome.contains(&needle),
            "ShellIcon should resolve the `{icon_key}` key to `{icon_svg}`"
        );
    }
}

fn repeated_ids(ids: &[String], len: usize) -> Vec<String> {
    (0..len)
        .map(|index| ids[index % ids.len()].clone())
        .collect()
}

fn repeated_reference_entries(ids: &[String], len: usize) -> Vec<AssetReferenceListPointerEntry> {
    (0..len)
        .map(|index| AssetReferenceListPointerEntry {
            asset_uuid: ids[index % ids.len()].clone(),
            known_project_asset: true,
        })
        .collect()
}

fn sample_catalog() -> EditorAssetCatalogSnapshotRecord {
    EditorAssetCatalogSnapshotRecord {
        project_name: "Sandbox".to_string(),
        project_root: "E:/Sandbox".to_string(),
        assets_root: "E:/Sandbox/assets".to_string(),
        library_root: "E:/Sandbox/library".to_string(),
        default_scene_uri: "res://scenes/main.scene.toml".to_string(),
        catalog_revision: 3,
        folders: vec![
            EditorAssetFolderRecord {
                folder_id: "res://".to_string(),
                parent_folder_id: None,
                locator_prefix: "res://".to_string(),
                display_name: "Assets".to_string(),
                child_folder_ids: vec!["res://materials".to_string(), "res://scenes".to_string()],
                direct_asset_uuids: Vec::new(),
                recursive_asset_count: 2,
            },
            EditorAssetFolderRecord {
                folder_id: "res://materials".to_string(),
                parent_folder_id: Some("res://".to_string()),
                locator_prefix: "res://materials".to_string(),
                display_name: "materials".to_string(),
                child_folder_ids: Vec::new(),
                direct_asset_uuids: vec!["11111111-1111-1111-1111-111111111111".to_string()],
                recursive_asset_count: 1,
            },
            EditorAssetFolderRecord {
                folder_id: "res://scenes".to_string(),
                parent_folder_id: Some("res://".to_string()),
                locator_prefix: "res://scenes".to_string(),
                display_name: "scenes".to_string(),
                child_folder_ids: Vec::new(),
                direct_asset_uuids: vec!["22222222-2222-2222-2222-222222222222".to_string()],
                recursive_asset_count: 1,
            },
        ],
        assets: vec![
            EditorAssetCatalogRecord {
                uuid: "11111111-1111-1111-1111-111111111111".to_string(),
                id: "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa".to_string(),
                locator: "res://materials/grid.material.toml".to_string(),
                kind: ResourceKind::Material,
                display_name: "grid.material".to_string(),
                file_name: "grid.material.toml".to_string(),
                extension: "toml".to_string(),
                preview_state: PreviewState::Ready,
                meta_path: "E:/Sandbox/assets/materials/grid.material.toml.meta.toml".to_string(),
                preview_artifact_path: "E:/Sandbox/library/editor-previews/grid.png".to_string(),
                source_mtime_unix_ms: 10,
                source_hash: "mat".to_string(),
                dirty: false,
                diagnostics: Vec::new(),
                direct_reference_uuids: Vec::new(),
            },
            EditorAssetCatalogRecord {
                uuid: "22222222-2222-2222-2222-222222222222".to_string(),
                id: "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb".to_string(),
                locator: "res://scenes/main.scene.toml".to_string(),
                kind: ResourceKind::Scene,
                display_name: "main.scene".to_string(),
                file_name: "main.scene.toml".to_string(),
                extension: "toml".to_string(),
                preview_state: PreviewState::Dirty,
                meta_path: "E:/Sandbox/assets/scenes/main.scene.toml.meta.toml".to_string(),
                preview_artifact_path: "E:/Sandbox/library/editor-previews/main.png".to_string(),
                source_mtime_unix_ms: 20,
                source_hash: "scene".to_string(),
                dirty: true,
                diagnostics: vec!["preview dirty".to_string()],
                direct_reference_uuids: Vec::new(),
            },
        ],
    }
}
