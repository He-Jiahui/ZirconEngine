use super::support::*;
use crate::core::editing::engine::{HistoryContextId, HistoryStatus};
use crate::core::editor_event::{EditorEventSource, EditorHierarchyEvent};
use crate::core::project::{NewProjectDraft, ProjectAuthority, ProjectTemplateId};
use crate::ui::retained_host::app::hierarchy_rename::{
    HIERARCHY_INLINE_RENAME_COMMIT_ACTION_ID, HIERARCHY_INLINE_RENAME_CONTROL_ID,
    HIERARCHY_INLINE_RENAME_DISPATCH_KIND_PREFIX, HIERARCHY_INLINE_RENAME_EDIT_ACTION_ID,
};
use crate::ui::retained_host::HostTextInputFocusData;
use zircon_runtime_interface::project::{
    ProjectLaunchIntent, ProjectLaunchProfile, ProjectLaunchSource,
};

#[test]
fn failed_hierarchy_rename_commit_keeps_exact_focus_for_successful_retry() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_hierarchy_rename_failed_commit_draft");
    harness.activate_workbench_page();

    let node_id = harness
        .host
        .borrow()
        .hierarchy_scene_entries
        .iter()
        .next()
        .expect("default scene should expose a hierarchy rename target")
        .entity;
    let draft = "  Retry Rename Draft  ";
    let expected_focus = HostTextInputFocusData {
        control_id: HIERARCHY_INLINE_RENAME_CONTROL_ID.into(),
        dispatch_kind: format!("{HIERARCHY_INLINE_RENAME_DISPATCH_KIND_PREFIX}{node_id}").into(),
        edit_action_id: HIERARCHY_INLINE_RENAME_EDIT_ACTION_ID.into(),
        commit_action_id: HIERARCHY_INLINE_RENAME_COMMIT_ACTION_ID.into(),
        value_text: draft.into(),
        ..HostTextInputFocusData::default()
    };
    {
        let mut host = harness.host.borrow_mut();
        host.runtime
            .shell()
            .lock()
            .state
            .enter_play_mode()
            .expect("loaded test world should enter editor play mode");
        host.runtime.refresh_reflection();
        host.refresh_ui();
        host.recompute_if_dirty();
    }
    host_context(&harness.root_ui).set_text_input_focus(expected_focus.clone());
    assert!(host_context(&harness.root_ui).get_text_input_focus() == expected_focus);

    let failed_journal = harness.journal_len();
    let history_before_failure = history_status(&harness);
    harness.root_ui.dispatch_native_key_for_test(
        key_event(
            Key::Named(NamedKey::Enter),
            PhysicalKey::Code(KeyCode::Enter),
            None,
            ElementState::Pressed,
        ),
        ModifiersState::empty(),
    );

    assert!(
        host_context(&harness.root_ui).get_text_input_focus() == expected_focus,
        "a synchronous Editor dispatch failure must retain the exact draft and rename target"
    );
    assert_eq!(history_status(&harness), history_before_failure);
    let failure_status = harness
        .host
        .borrow()
        .runtime
        .shell()
        .lock()
        .state
        .status_line
        .clone();
    assert!(
        failure_status.contains("scene editing is disabled during play mode"),
        "the failed synchronous rename should surface the executor error: {failure_status}"
    );
    let failed_events = harness.delta_events_since(failed_journal);
    assert!(failed_events.len() <= 1);
    assert!(failed_events.iter().all(|event| matches!(
        event,
        EditorEvent::Hierarchy(EditorHierarchyEvent::RenameNode {
            node_id: event_node,
            name: event_name,
        }) if *event_node == node_id && event_name == draft.trim()
    )));

    {
        let mut host = harness.host.borrow_mut();
        host.runtime
            .shell()
            .lock()
            .state
            .exit_play_mode()
            .expect("the test should restore edit mode before retry");
        host.runtime.refresh_reflection();
    }
    assert!(
        host_context(&harness.root_ui).get_text_input_focus() == expected_focus,
        "leaving play mode must not consume the retained rename draft"
    );

    let retry_journal = harness.journal_len();
    let history_before_retry = history_status(&harness);
    harness.root_ui.dispatch_native_key_for_test(
        key_event(
            Key::Named(NamedKey::Enter),
            PhysicalKey::Code(KeyCode::Enter),
            None,
            ElementState::Pressed,
        ),
        ModifiersState::empty(),
    );

    assert!(!host_context(&harness.root_ui)
        .get_text_input_focus()
        .is_active());
    assert_eq!(
        harness.delta_events_since(retry_journal),
        vec![EditorEvent::Hierarchy(EditorHierarchyEvent::RenameNode {
            node_id,
            name: draft.trim().to_string(),
        })]
    );
    let history_after_retry = history_status(&harness);
    assert_eq!(history_after_retry.len, history_before_retry.len + 1);
    assert!(history_after_retry.can_undo);
    assert_eq!(
        harness
            .host
            .borrow()
            .hierarchy_scene_entries
            .iter()
            .find(|entry| entry.entity == node_id)
            .map(|entry| entry.display_name.as_str()),
        Some(draft.trim())
    );
}

// Existing RenderableEmpty entities, authored into the saved scene before startup.
const HIERARCHY_RENAME_LABELS: [(&str, &str); 2] =
    [("Camera", "Caf\u{00e9}"), ("Sun", "Cafe\u{0301}")];

#[test]
fn hierarchy_inline_rename_keyboard_removes_one_visible_character() {
    let _guard = lock_env();
    let location = unique_temp_path("zircon_hierarchy_rename_grapheme").with_extension("");
    fs::create_dir(&location).expect("create fresh temporary project parent");
    let (core, root_ui, host) = open_authored_rename_project(&location);
    let history = {
        let host = host.borrow();
        let manager = host.editor_manager.enter().expect("real EditorManager");
        let (root, _, _) = manager
            .active_project_session_focus_target()
            .expect("cold project startup should reach Ready");
        let scene = manager
            .active_scene_identity_for_session()
            .expect("startup should bind the real scene document");
        assert_eq!(
            root.canonicalize().unwrap(),
            location.join("RenameGrapheme").canonicalize().unwrap()
        );
        assert_eq!(scene.project_root(), root.as_path());
        assert_eq!(scene.scene_uri(), "res://scenes/main.scene.toml");
        assert!(scene.source_revision().is_some());
        let history = HistoryContextId::Document(scene.document());
        assert_eq!(host.runtime.active_scene_history_context(), Some(history));
        history
    };
    let observe_history = || {
        let host = host.borrow();
        let shell = host.runtime.shell().lock();
        assert_eq!(shell.state.active_scene_history_context(), Some(history));
        shell.state.transactions().history_status(history).unwrap()
    };
    let press = |named, code, modifiers| {
        root_ui.dispatch_native_key_for_test(
            key_event(
                Key::Named(named),
                PhysicalKey::Code(code),
                None,
                ElementState::Pressed,
            ),
            modifiers,
        );
    };
    let world_label = |node_id| {
        let host = host.borrow();
        let shell = host.runtime.shell().lock();
        shell
            .state
            .world
            .expect_with_world(|scene| scene.find_node(node_id).expect("same authored entity").name)
    };
    let mut labels = Vec::new();

    for (_, initial) in HIERARCHY_RENAME_LABELS {
        let node_id = {
            let mut host = host.borrow_mut();
            let snapshot = host
                .runtime
                .editor_snapshot()
                .expect("actual edit world snapshot");
            let mut rows = snapshot
                .scene_entries
                .iter()
                .filter(|row| row.display_name.as_bytes() == initial.as_bytes());
            let id = rows
                .next()
                .expect("startup should load the exact authored UTF-8 label")
                .entity;
            assert!(rows.next().is_none());
            let effects = callback_dispatch::dispatch_hierarchy_selection(&host.runtime, id)
                .expect("select the authored entity through the real Hierarchy action");
            host.apply_dispatch_effects(effects);
            host.sync_shell_size();
            host.refresh_ui();
            host.recompute_if_dirty();
            if let Some(error) = root_ui.take_fatal_failure() {
                panic!("retained workspace should present the admitted scene: {error}");
            }
            let selection = host.runtime.editor_snapshot().unwrap().scene_entries;
            assert!(selection.is_selected(id));
            assert_eq!(
                selection
                    .iter()
                    .filter(|row| selection.is_selected(row.entity))
                    .count(),
                1
            );
            id
        };
        assert_eq!(world_label(node_id).as_bytes(), initial.as_bytes());
        let before = host.borrow().runtime.journal().records().len();
        let history_before = observe_history();
        let plain = ModifiersState::empty();
        press(NamedKey::F2, KeyCode::F2, plain);
        let context = host_context(&root_ui);
        let focus = context.get_text_input_focus();
        assert_eq!(
            focus.control_id.as_str(),
            HIERARCHY_INLINE_RENAME_CONTROL_ID
        );
        assert_eq!(
            focus.dispatch_kind.as_str(),
            format!("{HIERARCHY_INLINE_RENAME_DISPATCH_KIND_PREFIX}{node_id}")
        );
        assert_eq!(focus.value_text.as_str().as_bytes(), initial.as_bytes());

        press(NamedKey::End, KeyCode::End, plain);
        press(
            NamedKey::ArrowLeft,
            KeyCode::ArrowLeft,
            ModifiersState::SHIFT,
        );
        press(NamedKey::Backspace, KeyCode::Backspace, plain);
        let draft = context.get_text_input_focus().value_text;
        assert_eq!(world_label(node_id).as_bytes(), initial.as_bytes());
        assert_eq!(host.borrow().runtime.journal().records().len(), before);
        assert_eq!(observe_history(), history_before);

        press(NamedKey::Enter, KeyCode::Enter, plain);
        assert!(!context.get_text_input_focus().is_active());
        let committed = world_label(node_id);
        assert_eq!(committed.as_bytes(), draft.as_str().as_bytes());
        let history_after = observe_history();
        assert_eq!(history_after.len, history_before.len + 1);
        let journal = host.borrow().runtime.journal();
        let [record] = &journal.records()[before..] else {
            panic!("Enter should append exactly one scene event");
        };
        assert_eq!(record.source, EditorEventSource::RetainedHost);
        assert!(record.result.error.is_none(), "{:?}", record.result);
        assert_eq!(
            record.event,
            EditorEvent::Hierarchy(EditorHierarchyEvent::RenameNode {
                node_id,
                name: committed.clone()
            })
        );
        eprintln!("HIERARCHY_RENAME_GRAPHEME_COMMIT initial={initial:?} initial_bytes={:?} node={node_id} draft={:?} scene={committed:?} history_delta=1 rename_events=1", initial.as_bytes(), draft.as_str());
        labels.push((initial, committed));
    }

    // Test the Unicode result only after both cases proved an authoritative commit.
    for (initial, committed) in labels {
        assert_eq!(Some(committed.as_str()), Some("Caf"), "HIERARCHY_RENAME_GRAPHEME_ORACLE: End, Shift+Left, Backspace and Enter remove one visible character from {initial:?}");
    }

    host.borrow_mut()
        .commit_project_close()
        .expect("close the temporary authored project");
    drop(host);
    drop(root_ui);
    drop(core);
    fs::remove_dir_all(location).expect("remove only the owned temporary fixture");
}

fn open_authored_rename_project(
    location: &std::path::Path,
) -> (CoreRuntime, UiHostWindow, Rc<RefCell<RetainedEditorHost>>) {
    let restore_config =
        crate::tests::editor_event::support::EnvironmentVariableRestoreGuard::capture(
            "ZIRCON_CONFIG_PATH",
        );
    std::env::set_var("ZIRCON_CONFIG_PATH", location.join("config.json"));
    let provenance = crate::tests::support::test_project_creation_provenance();
    let authority = ProjectAuthority::default();
    let created = authority
        .create_project(
            &NewProjectDraft {
                project_name: "RenameGrapheme".into(),
                location: location.to_string_lossy().into_owned(),
                template: ProjectTemplateId::RenderableEmpty,
            },
            &provenance,
        )
        .expect("create the ordinary RenderableEmpty project");
    let scene_path = created.root.join("assets/scenes/main.scene.toml");
    let mut source = fs::read_to_string(&scene_path).expect("read the template's actual scene");
    for (original, label) in HIERARCHY_RENAME_LABELS {
        let original = format!("name = \"{original}\"");
        assert_eq!(source.matches(original.as_str()).count(), 1);
        source = source.replacen(original.as_str(), &format!("name = \"{label}\""), 1);
    }
    fs::write(&scene_path, source.as_bytes())
        .expect("author the exact UTF-8 scene labels before Open");

    let core = CoreRuntime::new();
    for engine_module in zircon_runtime::builtin::runtime_core_modules() {
        core.register_module(engine_module.descriptor()).unwrap();
    }
    core.register_module(zircon_runtime::ui::module_descriptor())
        .unwrap();
    core.register_module(module::module_descriptor()).unwrap();
    core.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    core.activate_module(zircon_runtime::asset::ASSET_MODULE_NAME)
        .unwrap();
    core.activate_module(zircon_runtime::core::framework::scene::SCENE_MODULE_NAME)
        .unwrap();
    core.activate_module(module::EDITOR_MODULE_NAME).unwrap();

    let launch = crate::tests::support::test_project_creation_provenance();
    let intent = ProjectLaunchIntent::open_existing(
        launch.operation_id(),
        ProjectLaunchSource::Cli,
        ProjectLaunchProfile::Normal,
        &created.root,
    )
    .unwrap();
    let startup = EditorGuiStartupRequest::project(intent)
        .preflight_project(&authority)
        .expect("preflight ordinary Open");
    let ui = UiHostWindow::new().expect("initialize retained window state");
    ui.window().set_size(PhysicalSize::new(1672, 941));
    let mut host = RetainedEditorHost::new(
        core.handle(),
        // The outer profiling transport does not own the project's authoring world.
        Arc::new(crate::core::gateway::DetachedEditorRuntimeGateway),
        ui.clone_strong(),
        Some(startup),
        Some(launch.build_set_id().clone()),
        None,
    )
    .expect("cold Open must admit the authored scene through the normal host startup");
    drop(restore_config);
    assert_eq!(host.startup_session.mode, EditorSessionMode::Project);
    let settings = host.runtime.context().settings().snapshot();
    apply_host_appearance_from_tokens(settings.design_tokens_handle(), Some(settings.generation()));
    apply_host_paint_scale_factor(
        ResolutionContext::from_physical_size_with_scale_mode(
            host.shell_size,
            host.shell_scale_factor,
            host.shell_scale_mode,
        )
        .effective_scale_factor(),
    );
    ui.sync_host_paint_theme();
    install_editor_v2_design_tokens(settings.as_ref());
    host.sync_plugin_template_documents_if_changed()
        .expect("real editor templates");
    let host = Rc::new(RefCell::new(host));
    wire_callbacks(&ui, &host, None);
    host.borrow_mut().self_handle = Some(Rc::downgrade(&host));
    ui.show()
        .expect("enable retained input without running a desktop event loop");
    (core, ui, host)
}

fn history_status(harness: &ChildWindowHostHarness) -> HistoryStatus {
    let host = harness.host.borrow();
    let shell = host.runtime.shell().lock();
    shell
        .state
        .transactions()
        .history_status(HistoryContextId::Document(
            shell.state.active_scene_document.unwrap(),
        ))
        .expect("the harness should have an active scene history")
}
