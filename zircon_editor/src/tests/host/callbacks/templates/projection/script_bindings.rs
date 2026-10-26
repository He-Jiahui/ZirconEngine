//! Actual Details controls exercise the typed operation and its shared history.
use super::*;
use crate::core::editing::engine::HistoryContextId;
use crate::core::editor_event::{EditorEventSource, SelectionHostEvent};
use crate::core::play::WorldDomain;
use crate::tests::editor_event::support::EnvironmentVariableRestoreGuard;
use crate::ui::host::{module::EDITOR_MANAGER_NAME, EditorManager};
use crate::ui::retained_host::callback_dispatch::{
    dispatch_componentized_workbench_option_selected,
    dispatch_componentized_workbench_surface_control_edited,
};
use crate::ui::workbench::snapshot::InspectorScriptBindingSnapshot;
use serde_json::{json, Value};
use zircon_runtime::asset::SceneScriptBindingAsset;
use zircon_runtime::asset::{asset_manager_handle, project::ProjectManager};
use zircon_runtime::core::manager::resolve_manager_service;
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::{NodeId, World};

fn fixture() -> (
    EventRuntimeHarness,
    BuiltinWorkbenchWindowTemplateSurfaceBridge,
    NodeId,
    std::path::PathBuf,
) {
    // Fixture files belong to the caller's evidence root; Cargo keeps its own TMPDIR.
    let _restore_temp = EnvironmentVariableRestoreGuard::capture("TMPDIR");
    if let Some(root) = std::env::var_os("ZIRCON_G02_UI_ARTIFACT_ROOT") {
        std::fs::create_dir_all(&root).unwrap();
        std::env::set_var("TMPDIR", root);
    }
    let mut harness = EventRuntimeHarness::new("zircon_script_binding_controls");
    let mut root = std::path::PathBuf::new();
    harness.open_project_with_assets("zircon_script_binding_project", |assets| {
        let source = assets.source_path("res://probe.txt");
        root = source.parent().unwrap().parent().unwrap().to_owned();
        let manifest = root.join("zircon-project.toml");
        let mut text = std::fs::read_to_string(&manifest).unwrap();
        text.push_str("\n[scripts]\npackage_roots = [\"scripts\"]\nstartup_packages = []\n");
        std::fs::write(manifest, text).unwrap();
        let package = root.join("scripts/orb_run");
        std::fs::create_dir_all(&package).unwrap();
        std::fs::write(
            package.join("plugin.toml"),
            "name = \"orb_run\"\nversion = \"0.1.0\"\nentry = \"main\"\nbackend = \"test\"\n",
        )
        .unwrap();
    });
    let project = project(&harness);
    let uri = project.manifest().default_scene.clone();
    let (world, revision) = World::load_scene_document(&project, &uri)
        .unwrap()
        .into_parts();
    let target = world
        .nodes()
        .iter()
        .find(|node| node.kind == NodeKind::Cube)
        .unwrap()
        .id;
    install(&harness, world, revision, &project, target);
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    sync(&harness, &mut bridge);
    (harness, bridge, target, root)
}

fn sync(harness: &EventRuntimeHarness, bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge) {
    bridge
        .sync_from_chrome(&harness.runtime.chrome_snapshot().unwrap())
        .unwrap();
}

fn click(
    harness: &EventRuntimeHarness,
    bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge,
    id: &str,
) {
    dispatch_componentized_workbench_control(&harness.runtime, bridge, id, UiEventKind::Click)
        .expect("authored script control must have a registered binding")
        .unwrap();
}

fn edit(
    bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge,
    id: &str,
    binding: &str,
    value: &str,
) {
    dispatch_componentized_workbench_surface_control_edited(bridge, id, binding, value)
        .expect("authored script editor callback must be consumed")
        .unwrap();
}

fn bindings(harness: &EventRuntimeHarness, target: NodeId) -> Option<Value> {
    harness
        .runtime
        .shell()
        .lock()
        .state
        .world
        .expect_with_world(|world| {
            world
                .dynamic_component(target, SceneScriptBindingAsset::COMPONENT_TYPE_PATH)
                .cloned()
        })
}

fn history(harness: &EventRuntimeHarness) -> HistoryContextId {
    harness
        .runtime
        .shell()
        .lock()
        .state
        .active_scene_history_context()
        .unwrap()
}

fn project(harness: &EventRuntimeHarness) -> ProjectManager {
    let core = harness.core.handle();
    let project = resolve_manager_service(&core, asset_manager_handle(&core).unwrap())
        .unwrap()
        .current_project_snapshot()
        .unwrap();
    project
}

fn install(
    harness: &EventRuntimeHarness,
    world: World,
    revision: zircon_runtime::scene::world::SceneSourceRevision,
    project: &ProjectManager,
    target: NodeId,
) {
    {
        let manager = harness
            .core
            .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
            .unwrap();
        let seed = manager.prepare_authoring_world(world).unwrap();
        harness
            .runtime
            .replace_world(seed, project.paths().root().to_string_lossy())
            .unwrap();
        let document = manager
            .activate_startup_scene_document(
                project.paths().root(),
                &project.manifest().default_scene,
                Some(revision),
            )
            .unwrap();
        harness
            .runtime
            .shell()
            .lock()
            .state
            .bind_scene_document(document);
    }
    harness
        .runtime
        .dispatch_event(
            EditorEventSource::RetainedHost,
            EditorEvent::Selection(SelectionHostEvent::SelectSceneNode {
                world_domain: WorldDomain::Edit,
                node_id: target,
            }),
        )
        .unwrap();
}

fn save(harness: &EventRuntimeHarness) {
    let record = harness
        .runtime
        .dispatch_event(
            EditorEventSource::RetainedHost,
            EditorEvent::WorkbenchMenu(MenuAction::SaveProject),
        )
        .unwrap();
    assert!(record.save_generation.is_some());
    assert!(!harness
        .runtime
        .context()
        .transactions()
        .is_dirty(history(harness))
        .unwrap());
}

#[test]
fn script_binding_controls_save_reopen_source_and_persist_detach() {
    let _guard = env_lock().lock().unwrap();
    let (harness, mut bridge, target, root) = fixture();
    click(&harness, &mut bridge, "WorkbenchAddComponent");
    edit(
        &mut bridge,
        "WorkbenchComponentAddValues",
        "Inspector/ComponentAddValuesEdit",
        r#"{"role":"pickup","home_x":4.25,"moving":true}"#,
    );
    click(&harness, &mut bridge, "WorkbenchComponentAddConfirm");
    let attached = bindings(&harness, target).unwrap();
    save(&harness);

    // Reopen the saved source in a fresh project/World, then project it into Details.
    let mut reopened_project = ProjectManager::open(&root).unwrap();
    reopened_project.scan_and_import().unwrap();
    let uri = reopened_project.manifest().default_scene.clone();
    let (loaded, revision) = World::load_scene_document(&reopened_project, &uri)
        .unwrap()
        .into_parts();
    assert_eq!(
        loaded.dynamic_component(target, SceneScriptBindingAsset::COMPONENT_TYPE_PATH),
        Some(&attached)
    );
    install(&harness, loaded, revision, &reopened_project, target);
    sync(&harness, &mut bridge);
    assert_eq!(
        control_string(&bridge, "WorkbenchScriptBindingSelect", "value_text").as_deref(),
        Some("orb_run · main")
    );
    click(&harness, &mut bridge, "WorkbenchScriptBindingRemove");
    assert_eq!(bindings(&harness, target), None);
    let transactions = harness.runtime.context().transactions();
    assert!(transactions.undo(history(&harness)).unwrap());
    assert_eq!(bindings(&harness, target), Some(attached));
    assert!(transactions.redo(history(&harness)).unwrap());
    save(&harness);
    let (loaded, _) = World::load_scene_document(&reopened_project, &uri)
        .unwrap()
        .into_parts();
    assert_eq!(
        loaded.dynamic_component(target, SceneScriptBindingAsset::COMPONENT_TYPE_PATH),
        None
    );
}

#[test]
fn script_binding_controls_attach_edit_toggle_remove_with_exact_undo_redo() {
    let _guard = env_lock().lock().unwrap();
    let (harness, mut bridge, target, _) = fixture();
    click(&harness, &mut bridge, "WorkbenchAddComponent");
    assert_eq!(
        control_visibility(&bridge, "WorkbenchComponentAddScriptFields"),
        Some(UiVisibility::Visible)
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchComponentAddPackage", "value_text").as_deref(),
        Some("orb_run")
    );
    edit(
        &mut bridge,
        "WorkbenchComponentAddValues",
        "Inspector/ComponentAddValuesEdit",
        r#"{"role":"pickup","home_x":4,"moving":true}"#,
    );
    click(&harness, &mut bridge, "WorkbenchComponentAddFixedUpdate");
    click(&harness, &mut bridge, "WorkbenchComponentAddConfirm");
    let attached = bindings(&harness, target).unwrap();
    assert_eq!(attached[0]["properties"]["role"], "pickup");
    assert_eq!(attached[0]["fixed_update"], false);
    let history = history(&harness);
    let transactions = harness.runtime.context().transactions();
    assert!(transactions.undo(history).unwrap());
    assert_eq!(bindings(&harness, target), None);
    assert!(transactions.redo(history).unwrap());
    assert_eq!(bindings(&harness, target), Some(attached));

    sync(&harness, &mut bridge);
    click(&harness, &mut bridge, "WorkbenchScriptBindingEdit");
    edit(
        &mut bridge,
        "WorkbenchComponentAddModule",
        "Inspector/ScriptBindingModuleEdit",
        "pickup_module",
    );
    click(&harness, &mut bridge, "WorkbenchComponentAddUpdate");
    click(&harness, &mut bridge, "WorkbenchComponentAddConfirm");
    let edited = bindings(&harness, target).unwrap();
    assert_eq!(edited[0]["module"], "pickup_module");
    assert_eq!(edited[0]["update"], false);
    sync(&harness, &mut bridge);
    click(&harness, &mut bridge, "WorkbenchScriptBindingEnable");
    let disabled = bindings(&harness, target).unwrap();
    assert_eq!(disabled[0]["enabled"], false);
    sync(&harness, &mut bridge);
    click(&harness, &mut bridge, "WorkbenchScriptBindingRemove");
    assert_eq!(bindings(&harness, target), None);
    assert!(transactions.undo(history).unwrap());
    assert_eq!(bindings(&harness, target), Some(disabled.clone()));

    assert!(transactions.redo(history).unwrap());
    assert_eq!(bindings(&harness, target), None);
    assert!(transactions.undo(history).unwrap());
    assert_eq!(bindings(&harness, target), Some(disabled));
}

#[test]
fn script_binding_controls_preserve_invalid_stale_and_legacy_drafts() {
    let _guard = env_lock().lock().unwrap();
    let (harness, mut bridge, target, _) = fixture();
    click(&harness, &mut bridge, "WorkbenchAddComponent");
    edit(
        &mut bridge,
        "WorkbenchComponentAddValues",
        "Inspector/ComponentAddValuesEdit",
        r#"{"nested":{}}"#,
    );
    click(&harness, &mut bridge, "WorkbenchComponentAddConfirm");
    assert_eq!(bindings(&harness, target), None);
    assert_eq!(
        control_visibility(&bridge, "WorkbenchComponentAddPanel"),
        Some(UiVisibility::Visible)
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchComponentAddValues", "query").as_deref(),
        Some(r#"{"nested":{}}"#)
    );
    edit(
        &mut bridge,
        "WorkbenchComponentAddValues",
        "Inspector/ComponentAddValuesEdit",
        r#"{"role":"pickup"}"#,
    );
    harness
        .runtime
        .shell()
        .lock()
        .state
        .world
        .expect_with_world_mut(|world| {
            world
                .spawn((zircon_runtime::scene::components::Name("unrelated".into()),))
                .unwrap();
        });
    click(&harness, &mut bridge, "WorkbenchComponentAddConfirm");
    assert_eq!(bindings(&harness, target), None);
    click(&harness, &mut bridge, "WorkbenchComponentAddCancel");
    assert!(control_component_focused(&bridge, "WorkbenchAddComponent"));

    let legacy = json!([{"package":"orb_run","module":"main","enabled":true,"update":true,"fixed_update":false,"properties":{"nested":{"keep":[1,2]}}}]);
    harness
        .runtime
        .shell()
        .lock()
        .state
        .world
        .expect_with_world_mut(|world| {
            world
                .set_dynamic_component(target, "script.bindings", legacy.clone())
                .unwrap();
        });
    sync(&harness, &mut bridge);
    click(&harness, &mut bridge, "WorkbenchScriptBindingEdit");
    click(&harness, &mut bridge, "WorkbenchComponentAddEnabled");
    click(&harness, &mut bridge, "WorkbenchComponentAddConfirm");
    let value = bindings(&harness, target).unwrap();
    assert_eq!(value[0]["enabled"], false);
    assert_eq!(value[0]["properties"], legacy[0]["properties"]);
}

#[test]
fn script_binding_controls_page_long_names_and_project_empty_and_error_states() {
    let _guard = env_lock().lock().unwrap();
    let (harness, mut bridge, _, _) = fixture();
    bridge.recompute_layout(UiSize::new(680.0, 720.0)).unwrap();
    let mut chrome = harness.runtime.chrome_snapshot().unwrap();
    let inspector = chrome.inspector.as_mut().unwrap();
    inspector.script_bindings = Ok((0..35)
        .map(|i| InspectorScriptBindingSnapshot {
            package: format!("长名称 package | with, delimiters {i}"),
            module: "main".into(),
            enabled: true,
            update: true,
            fixed_update: false,
            properties: Default::default(),
        })
        .collect());
    bridge.sync_from_chrome(&chrome).unwrap();
    assert_eq!(
        control_attribute(&bridge, "WorkbenchScriptBindingSelect", "options")
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        16
    );
    assert!(!control_bool(
        &bridge,
        "WorkbenchScriptBindingNext",
        "disabled"
    ));
    // Select a page entry through the actual retained dropdown callback.
    click(&harness, &mut bridge, "WorkbenchScriptBindingNext");
    dispatch_componentized_workbench_option_selected(
        &harness.runtime,
        &mut bridge,
        "WorkbenchScriptBindingSelect",
        "17",
    )
    .unwrap();
    assert!(
        control_string(&bridge, "WorkbenchScriptBindingSelect", "value_text")
            .unwrap()
            .ends_with("17 · main")
    );
    chrome.inspector.as_mut().unwrap().script_bindings = Ok(Vec::new());
    bridge.sync_from_chrome(&chrome).unwrap();
    assert!(control_bool(
        &bridge,
        "WorkbenchScriptBindingEdit",
        "disabled"
    ));
    chrome.inspector.as_mut().unwrap().script_bindings = Err("invalid stored binding".into());
    bridge.sync_from_chrome(&chrome).unwrap();
    assert!(
        control_string(&bridge, "WorkbenchScriptBindingStatus", "text")
            .unwrap()
            .contains("invalid stored binding")
    );
    chrome.inspector = None;
    bridge.sync_from_chrome(&chrome).unwrap();
    assert_eq!(
        control_visibility(&bridge, "WorkbenchInspectorScripts"),
        Some(UiVisibility::Collapsed)
    );
}

#[test]
fn script_binding_controls_share_picker_validate_module_and_undo_selected_detach() {
    let _guard = env_lock().lock().unwrap();
    let (harness, mut bridge, target, _) = fixture();
    harness
        .runtime
        .shell()
        .lock()
        .state
        .world
        .expect_with_world_mut(|world| {
            world
                .register_component_type(
                    zircon_runtime::core::framework::scene::ComponentTypeDescriptor::new(
                        "test.G02",
                        "test",
                        "Other component",
                    )
                    .with_property("label", "String", false),
                )
                .unwrap();
        });
    click(&harness, &mut bridge, "WorkbenchAddComponent");
    dispatch_componentized_workbench_option_selected(
        &harness.runtime,
        &mut bridge,
        "WorkbenchComponentAddType",
        "test.G02",
    )
    .unwrap();
    assert_eq!(
        control_visibility(&bridge, "WorkbenchComponentAddScriptFields"),
        Some(UiVisibility::Collapsed)
    );
    dispatch_componentized_workbench_option_selected(
        &harness.runtime,
        &mut bridge,
        "WorkbenchComponentAddType",
        "script.bindings",
    )
    .unwrap();
    assert_eq!(
        control_visibility(&bridge, "WorkbenchComponentAddScriptFields"),
        Some(UiVisibility::Visible)
    );
    edit(
        &mut bridge,
        "WorkbenchComponentAddModule",
        "Inspector/ScriptBindingModuleEdit",
        "  ",
    );
    assert!(control_bool(
        &bridge,
        "WorkbenchComponentAddConfirm",
        "disabled"
    ));
    assert_eq!(
        harness
            .runtime
            .context()
            .transactions()
            .history_status(history(&harness))
            .unwrap()
            .len,
        0
    );
    edit(
        &mut bridge,
        "WorkbenchComponentAddModule",
        "Inspector/ScriptBindingModuleEdit",
        "main",
    );
    assert!(!control_bool(
        &bridge,
        "WorkbenchComponentAddConfirm",
        "disabled"
    ));
    click(&harness, &mut bridge, "WorkbenchComponentAddPackage");
    assert!(control_bool(
        &bridge,
        "WorkbenchComponentAddPackage",
        "popup_open"
    ));
    dispatch_componentized_workbench_option_selected(
        &harness.runtime,
        &mut bridge,
        "WorkbenchComponentAddPackage",
        "0",
    )
    .unwrap();
    click(&harness, &mut bridge, "WorkbenchComponentAddConfirm");
    let first = bindings(&harness, target).unwrap();
    assert_eq!(first[0]["package"], "orb_run");
    let chrome = harness.runtime.chrome_snapshot().unwrap();
    assert!(!chrome
        .inspector
        .as_ref()
        .unwrap()
        .plugin_components
        .iter()
        .any(|component| component.component_id == SceneScriptBindingAsset::COMPONENT_TYPE_PATH));
    bridge.sync_from_chrome(&chrome).unwrap();
    click(&harness, &mut bridge, "WorkbenchScriptBindingSelect");
    assert!(control_bool(
        &bridge,
        "WorkbenchScriptBindingSelect",
        "popup_open"
    ));
    dispatch_componentized_workbench_option_selected(
        &harness.runtime,
        &mut bridge,
        "WorkbenchScriptBindingSelect",
        "0",
    )
    .unwrap();
    assert!(!control_bool(
        &bridge,
        "WorkbenchScriptBindingSelect",
        "popup_open"
    ));

    click(&harness, &mut bridge, "WorkbenchAddComponent");
    edit(
        &mut bridge,
        "WorkbenchComponentAddModule",
        "Inspector/ScriptBindingModuleEdit",
        "pickup",
    );
    click(&harness, &mut bridge, "WorkbenchComponentAddConfirm");
    let both = bindings(&harness, target).unwrap();
    assert_eq!(both.as_array().unwrap().len(), 2);
    sync(&harness, &mut bridge);
    dispatch_componentized_workbench_option_selected(
        &harness.runtime,
        &mut bridge,
        "WorkbenchScriptBindingSelect",
        "1",
    )
    .unwrap();
    click(&harness, &mut bridge, "WorkbenchScriptBindingRemove");
    assert_eq!(bindings(&harness, target), Some(first.clone()));
    let transactions = harness.runtime.context().transactions();
    assert!(transactions.undo(history(&harness)).unwrap());
    assert_eq!(bindings(&harness, target), Some(both));
    assert!(transactions.redo(history(&harness)).unwrap());
    assert_eq!(bindings(&harness, target), Some(first));
}
