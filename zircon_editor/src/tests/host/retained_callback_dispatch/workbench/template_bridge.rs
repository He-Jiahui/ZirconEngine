use super::super::support::*;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::editing::engine::HistoryContextId;
use crate::core::project::{NewProjectDraft, NewProjectTemplate, ProjectAuthority};
use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorManager;

#[test]
fn builtin_host_window_template_bridge_dispatches_reset_layout_from_shared_control_projection() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_retained_template_bridge_reset_layout");
    let bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    let reset_layout_binding_path = bridge
        .binding_for_control("ResetLayout", UiEventKind::Click)
        .expect("reset layout control should expose a native binding")
        .path()
        .native_prefix();

    let reset_layout = bridge
        .host_projection()
        .node_by_control_id("ResetLayout")
        .expect("reset layout control should exist in builtin template projection");
    assert_eq!(reset_layout.frame, UiFrame::new(164.0, 0.0, 96.0, 24.0));

    let effects =
        dispatch_builtin_host_control(&harness.runtime, &bridge, "ResetLayout", UiEventKind::Click)
            .expect("templated control should resolve")
            .unwrap();

    let journal = harness.runtime.journal();
    let record = journal.records().last().unwrap();
    assert_eq!(
        record.event,
        EditorEvent::WorkbenchMenu(MenuAction::ResetLayout)
    );
    assert_eq!(
        record.binding_path.as_deref(),
        Some(reset_layout_binding_path.as_str()),
        "retained-host menu dispatch must preserve the originating UI binding path"
    );
    assert_eq!(record.operation_id.as_deref(), Some("window.layout.reset"));
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert!(!effects.render_dirty);
}

#[test]
fn builtin_host_save_project_preserves_binding_provenance_and_save_generation() {
    let _guard = env_lock().lock().unwrap();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after the Unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("zircon_retained_template_save_{unique}"));
    let location = root
        .parent()
        .expect("temporary project root should have a parent");
    ProjectAuthority::default()
        .create_project(&NewProjectDraft {
            project_name: root
                .file_name()
                .expect("temporary project root should have a name")
                .to_string_lossy()
                .into_owned(),
            location: location.to_string_lossy().into_owned(),
            template: NewProjectTemplate::RenderableEmpty,
        })
        .expect("renderable template project should be created");

    {
        let harness = EventRuntimeHarness::new("zircon_retained_template_save_project");
        let manager = harness
            .core
            .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
            .expect("editor manager should resolve");
        let document = manager.open_project(&root).expect("project should open");
        let level = manager
            .prepare_authoring_world(document.world)
            .expect("opened project scene should create an authoring world");
        harness
            .runtime
            .replace_world(level, root.to_string_lossy())
            .expect("runtime should adopt the opened project level");

        dispatch_menu_action(&harness.runtime, "workbench.scene.node.create.cube")
            .expect("menu mutation should create a transaction before save");
        assert!(harness
            .runtime
            .context()
            .transactions()
            .is_dirty(HistoryContextId::Global)
            .expect("transaction dirty state should be queryable"));

        let bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
        let save_binding_path = bridge
            .binding_for_control("SaveProject", UiEventKind::Click)
            .expect("save project control should expose a native binding")
            .path()
            .native_prefix();
        let effects = dispatch_builtin_host_menu_action(&harness.runtime, &bridge, "SaveProject")
            .expect("templated save project action should resolve")
            .expect("templated save project action should dispatch");

        let journal = harness.runtime.journal();
        let record = journal
            .records()
            .last()
            .expect("save project control must append an event record");
        assert_eq!(
            record.event,
            EditorEvent::WorkbenchMenu(MenuAction::SaveProject)
        );
        assert_eq!(
            record.binding_path.as_deref(),
            Some(save_binding_path.as_str())
        );
        assert_eq!(record.operation_id.as_deref(), Some("file.project.save"));
        assert_eq!(record.transaction_id, None);
        assert!(
            record.save_generation.is_some(),
            "a retained-host save must expose the persisted history generation"
        );
        assert!(effects.presentation_dirty);
        assert!(!harness
            .runtime
            .context()
            .transactions()
            .is_dirty(HistoryContextId::Global)
            .expect("successful save should mark the current history clean"));
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn builtin_host_open_project_requests_present_welcome_from_template_binding() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_retained_template_bridge_open_project");
    let bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();

    let effects = dispatch_builtin_host_menu_action(&harness.runtime, &bridge, "OpenProject")
        .expect("templated open project action should resolve")
        .unwrap();

    let journal = harness.runtime.journal();
    assert_eq!(
        journal.records().last().unwrap().event,
        EditorEvent::WorkbenchMenu(MenuAction::OpenProject)
    );
    assert!(effects.present_welcome_surface);
    assert!(effects.presentation_dirty);
    assert!(!effects.layout_dirty);
    assert!(!effects.render_dirty);
    assert!(!effects.sync_asset_workspace);
}

#[test]
fn builtin_host_reset_layout_matches_menu_action_dispatch() {
    let _guard = env_lock().lock().unwrap();

    let menu_harness = EventRuntimeHarness::new("zircon_retained_parity_reset_layout_menu");
    let menu_effects =
        dispatch_menu_action(&menu_harness.runtime, "workbench.layout.reset").unwrap();
    let menu_record = menu_harness
        .runtime
        .journal()
        .records()
        .last()
        .unwrap()
        .clone();

    let builtin_harness = EventRuntimeHarness::new("zircon_retained_parity_reset_layout_builtin");
    let bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    let builtin_effects =
        dispatch_builtin_host_menu_action(&builtin_harness.runtime, &bridge, "ResetLayout")
            .expect("templated reset layout action should resolve")
            .unwrap();
    let builtin_record = builtin_harness
        .runtime
        .journal()
        .records()
        .last()
        .unwrap()
        .clone();

    assert_eq!(builtin_effects, menu_effects);
    assert_eq!(builtin_record.event, menu_record.event);
    assert_eq!(builtin_record.operation_id, menu_record.operation_id);
    assert_eq!(menu_record.binding_path, None);
    assert!(
        builtin_record.binding_path.is_some(),
        "a retained host control must retain its binding provenance"
    );
}
