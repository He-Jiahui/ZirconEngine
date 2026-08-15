use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;
use zircon_editor::core::editor_event::{
    EditorEvent, EditorEventSequence, EditorEventSource, EditorViewportEvent, ViewInstanceId,
};
use zircon_editor::core::editor_message::{
    EditorUiDeltaBarrierKind, EditorUiDeltaEntry, EditorViewInvalidationMask,
    SharedEditorMessageBus,
};
use zircon_editor::core::sync::WorldWatchMap;
use zircon_editor::ui::host::{EditorHostEventController, EditorManager};
use zircon_editor::ui::workbench::state::EditorState;
use zircon_editor::{
    module_descriptor as editor_module_descriptor, EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY,
    EDITOR_MANAGER_NAME, EDITOR_MODULE_NAME,
};
use zircon_runtime::core::framework::scene::SCENE_MODULE_NAME;
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::foundation::{
    module_descriptor as foundation_module_descriptor, FOUNDATION_MODULE_NAME,
};
use zircon_runtime::scene::DefaultLevelManager;
use zircon_runtime_interface::math::UVec2;
use zircon_runtime_interface::ui::event_ui::{
    UiControlRequest, UiControlResponse, UiNodePath, UiReflectionNodePatch,
};
use zircon_runtime_interface::world_sync::{
    InvalidationBatch, WatchKey, WatchRegistration, WatchToken,
};

struct TestConfigEnvironment {
    previous_path: Option<OsString>,
    path: PathBuf,
}

fn config_environment_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl TestConfigEnvironment {
    fn install(prefix: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be later than the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("{prefix}_{unique}.json"));
        let previous_path = std::env::var_os("ZIRCON_CONFIG_PATH");
        std::env::set_var("ZIRCON_CONFIG_PATH", &path);
        Self {
            previous_path,
            path,
        }
    }
}

impl Drop for TestConfigEnvironment {
    fn drop(&mut self) {
        if let Some(previous_path) = self.previous_path.take() {
            std::env::set_var("ZIRCON_CONFIG_PATH", previous_path);
        } else {
            std::env::remove_var("ZIRCON_CONFIG_PATH");
        }
        let _ = std::fs::remove_file(&self.path);
    }
}

#[test]
fn public_watch_map_projects_runtime_tokens_into_view_dirty_state() {
    let mut map = WorldWatchMap::default();
    let hierarchy = ViewInstanceId::new("hierarchy");
    map.bind(
        WatchToken::new(4),
        WatchRegistration::new(WatchKey::WorldStructure),
        hierarchy.clone(),
        EditorViewInvalidationMask::TREE_STRUCTURE,
    )
    .unwrap();
    map.bind(
        WatchToken::new(8),
        WatchRegistration::new(WatchKey::Subtree { root: 4 }),
        hierarchy.clone(),
        EditorViewInvalidationMask::PRESENTATION_DATA,
    )
    .unwrap();

    let projection = map.project(&InvalidationBatch {
        generation: 17,
        dirty: vec![WatchToken::new(8), WatchToken::new(99), WatchToken::new(4)],
        facts: Vec::new(),
    });

    assert_eq!(projection.generation(), 17);
    assert_eq!(projection.matched_tokens(), 2);
    assert_eq!(projection.unknown_tokens(), &[WatchToken::new(99)]);
    assert_eq!(
        projection.dirty().mask_for(&hierarchy),
        Some(
            EditorViewInvalidationMask::TREE_STRUCTURE
                .union(EditorViewInvalidationMask::PRESENTATION_DATA)
        )
    );
}

#[test]
fn public_watch_map_view_and_session_cleanup_return_sorted_tokens() {
    let mut map = WorldWatchMap::default();
    let hierarchy = ViewInstanceId::new("hierarchy");
    let inspector = ViewInstanceId::new("inspector");
    for token in [WatchToken::new(8), WatchToken::new(3)] {
        map.bind(
            token,
            WatchRegistration::new(WatchKey::WorldStructure),
            hierarchy.clone(),
            EditorViewInvalidationMask::TREE_STRUCTURE,
        )
        .unwrap();
    }
    map.bind(
        WatchToken::new(5),
        WatchRegistration::new(WatchKey::ComponentType {
            type_name: "test.Component".to_string(),
        }),
        inspector,
        EditorViewInvalidationMask::PRESENTATION_DATA,
    )
    .unwrap();

    assert_eq!(
        map.unbind_view(&hierarchy),
        vec![WatchToken::new(3), WatchToken::new(8)]
    );
    assert_eq!(map.drain_tokens(), vec![WatchToken::new(5)]);
    assert!(map.is_empty());
}

#[test]
fn public_editor_ui_delta_queue_coalesces_properties_and_drains_with_dirty_state() {
    let bus = SharedEditorMessageBus::default();
    let view = ViewInstanceId::new("workbench.root");
    let node_path = UiNodePath::new("editor/workbench/scene");

    bus.push_editor_ui_patch(
        view.clone(),
        UiReflectionNodePatch::new(node_path.clone())
            .with_property("transient.hovered", json!(true)),
    );
    bus.push_editor_ui_patch(
        view.clone(),
        UiReflectionNodePatch::new(node_path).with_property("transient.hovered", json!(false)),
    );
    bus.mark_view_dirty(view.clone(), EditorViewInvalidationMask::TREE_STRUCTURE);
    bus.mark_view_dirty(view.clone(), EditorViewInvalidationMask::HIT_TEST);

    let (dirty, deltas) = bus.drain_view_updates();

    assert_eq!(deltas.node_delta_count(), 1);
    assert_eq!(deltas.barrier_count(), 0);
    assert_eq!(
        deltas.reflection_patches()[0].properties["transient.hovered"],
        json!(false)
    );
    assert_eq!(
        dirty.mask_for(&view),
        Some(
            EditorViewInvalidationMask::TREE_STRUCTURE.union(EditorViewInvalidationMask::HIT_TEST)
        )
    );
    let (dirty, deltas) = bus.drain_view_updates();
    assert!(dirty.is_empty());
    assert!(deltas.is_empty());
}

#[test]
fn public_editor_ui_delta_queue_uses_runtime_node_identity_across_views() {
    let bus = SharedEditorMessageBus::default();
    let node_path = UiNodePath::new("editor/workbench/shared-node");

    bus.push_editor_ui_patch(
        ViewInstanceId::new("z-view"),
        UiReflectionNodePatch::new(node_path.clone())
            .with_property("transient.hovered", json!(true)),
    );
    bus.push_editor_ui_patch(
        ViewInstanceId::new("a-view"),
        UiReflectionNodePatch::new(node_path).with_property("transient.hovered", json!(false)),
    );

    let (_, deltas) = bus.drain_view_updates();

    assert_eq!(deltas.node_delta_count(), 1);
    let EditorUiDeltaEntry::Nodes(node_deltas) = &deltas.entries()[0] else {
        panic!("expected one coalesced node delta");
    };
    assert_eq!(node_deltas[0].view(), &ViewInstanceId::new("a-view"));
    assert_eq!(
        deltas.reflection_patches()[0].properties["transient.hovered"],
        json!(false)
    );
}

#[test]
fn public_editor_ui_delta_queue_preserves_press_release_barriers() {
    let bus = SharedEditorMessageBus::default();
    let view = ViewInstanceId::new("workbench.root");
    let node_path = UiNodePath::new("editor/workbench/scene");

    bus.push_editor_ui_patch(
        view.clone(),
        UiReflectionNodePatch::new(node_path.clone()).with_pressed(true),
    );
    bus.push_editor_ui_barrier(EditorUiDeltaBarrierKind::Press, EditorEventSequence::new(7));
    bus.push_editor_ui_patch(
        view,
        UiReflectionNodePatch::new(node_path).with_pressed(false),
    );
    bus.push_editor_ui_barrier(
        EditorUiDeltaBarrierKind::Release,
        EditorEventSequence::new(9),
    );

    let (_, deltas) = bus.drain_view_updates();

    assert_eq!(deltas.node_delta_count(), 2);
    assert_eq!(deltas.barrier_count(), 2);
    assert!(matches!(
        deltas.entries(),
        [
            EditorUiDeltaEntry::Nodes(_),
            EditorUiDeltaEntry::Barrier {
                kind: EditorUiDeltaBarrierKind::Press,
                sequence: EditorEventSequence(7),
            },
            EditorUiDeltaEntry::Nodes(_),
            EditorUiDeltaEntry::Barrier {
                kind: EditorUiDeltaBarrierKind::Release,
                sequence: EditorEventSequence(9),
            },
        ]
    ));
    assert_eq!(
        deltas
            .reflection_patches()
            .iter()
            .map(|patch| patch.pressed)
            .collect::<Vec<_>>(),
        vec![Some(true), Some(false)]
    );
}

#[test]
fn public_full_refresh_replays_other_view_delta_after_snapshot() {
    let _config_lock = config_environment_lock();
    let _config = TestConfigEnvironment::install("zircon_editor_public_full_refresh_delta");
    let core = CoreRuntime::new();
    core.register_module(foundation_module_descriptor())
        .unwrap();
    core.register_module(zircon_runtime::asset::module_descriptor())
        .unwrap();
    core.register_module(zircon_runtime::scene::module_descriptor())
        .unwrap();
    core.register_module(editor_module_descriptor()).unwrap();
    core.store_config_value(EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY, json!([]));
    core.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    core.activate_module(zircon_runtime::asset::ASSET_MODULE_NAME)
        .unwrap();
    core.activate_module(SCENE_MODULE_NAME).unwrap();
    core.activate_module(EDITOR_MODULE_NAME).unwrap();

    let manager = core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let mut state = EditorState::with_default_selection_with_context(
        DefaultLevelManager::default().create_default_level(),
        UVec2::new(1280, 720),
        manager.context().clone(),
    );
    state.mark_project_open();
    let controller = EditorHostEventController::new(state, manager);
    let scene_path = UiNodePath::new("editor/workbench/pages/workbench/editor.scene#1");

    controller.context().bus().push_editor_ui_patch(
        ViewInstanceId::new("outliner"),
        UiReflectionNodePatch::new(scene_path.clone())
            .with_property("transient.hovered", json!(true)),
    );
    controller.context().bus().mark_view_dirty(
        ViewInstanceId::new("inspector"),
        EditorViewInvalidationMask::PRESENTATION_DATA,
    );

    let report = controller.drain_pending_view_refreshes();

    assert!(report.used_full_snapshot_fallback());
    assert_eq!(report.deltas().node_delta_count(), 1);
    let response = controller.handle_control_request(UiControlRequest::QueryNode {
        node_path: scene_path,
    });
    let UiControlResponse::Node(Some(scene_node)) = response else {
        panic!("expected scene reflection node after full refresh");
    };
    assert_eq!(
        scene_node.properties["transient.hovered"].reflected_value,
        json!(true)
    );
}

#[test]
fn public_retained_pointer_move_burst_does_not_schedule_reflection_work() {
    let _config_lock = config_environment_lock();
    let _config = TestConfigEnvironment::install("zircon_editor_public_pointer_move_burst");
    let core = CoreRuntime::new();
    core.register_module(foundation_module_descriptor())
        .unwrap();
    core.register_module(zircon_runtime::asset::module_descriptor())
        .unwrap();
    core.register_module(zircon_runtime::scene::module_descriptor())
        .unwrap();
    core.register_module(editor_module_descriptor()).unwrap();
    core.store_config_value(EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY, json!([]));
    core.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    core.activate_module(zircon_runtime::asset::ASSET_MODULE_NAME)
        .unwrap();
    core.activate_module(SCENE_MODULE_NAME).unwrap();
    core.activate_module(EDITOR_MODULE_NAME).unwrap();

    let manager = core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let mut state = EditorState::with_default_selection_with_context(
        DefaultLevelManager::default().create_default_level(),
        UVec2::new(1280, 720),
        manager.context().clone(),
    );
    state.mark_project_open();
    let controller = EditorHostEventController::new(state, manager);

    for index in 0..1_000 {
        controller
            .dispatch_event(
                EditorEventSource::RetainedHost,
                EditorEvent::Viewport(EditorViewportEvent::PointerMoved {
                    x: index as f32,
                    y: (index % 100) as f32,
                }),
            )
            .unwrap();
    }

    assert!(controller.context().bus().dirty_set().is_empty());
    let report = controller.drain_pending_view_refreshes();
    assert!(report.dirty().is_empty());
    assert!(report.deltas().is_empty());
    assert!(!report.used_full_snapshot_fallback());
    let journal = controller.journal();
    assert_eq!(journal.records().len(), 1);
    assert_eq!(journal.retention_diagnostics().coalesced_records(), 999);
    assert_eq!(journal.retention_diagnostics().dropped_records(), 0);
    assert!(matches!(
        &journal.records()[0].event,
        EditorEvent::Viewport(EditorViewportEvent::PointerMoved { x, y })
            if *x == 999.0 && *y == 99.0
    ));
}
