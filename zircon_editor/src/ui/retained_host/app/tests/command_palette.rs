use super::support::*;

#[test]
fn native_unhandled_ctrl_shift_p_opens_workbench_command_palette() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_native_keymap_palette");

    {
        let host = harness.host.borrow();
        assert!(!workbench_control_bool(
            &host,
            "WorkbenchCommandPalette",
            "popup_open"
        ));
    }

    let dispatch = harness.root_ui.dispatch_native_key_for_test(
        key_event(
            Key::Character("P".into()),
            PhysicalKey::Code(KeyCode::KeyP),
            Some("P"),
            ElementState::Pressed,
        ),
        ModifiersState::CONTROL | ModifiersState::SHIFT,
    );

    assert!(
        !dispatch.request_redraw(),
        "keymap dispatch should request redraw through retained host invalidation, not native text damage"
    );

    let mut host = harness.host.borrow_mut();
    host.recompute_if_dirty();
    assert!(workbench_control_bool(
        &host,
        "WorkbenchCommandPalette",
        "popup_open"
    ));
    assert_eq!(
        host.runtime
            .journal()
            .records()
            .last()
            .expect("keymap dispatch should record the command palette event")
            .event,
        EditorEvent::Transient(EditorEventTransient::OpenCommandPalette)
    );
}

#[test]
fn native_command_palette_enter_commits_focused_workbench_command() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_native_palette_enter");

    harness.root_ui.dispatch_native_key_for_test(
        key_event(
            Key::Character("P".into()),
            PhysicalKey::Code(KeyCode::KeyP),
            Some("P"),
            ElementState::Pressed,
        ),
        ModifiersState::CONTROL | ModifiersState::SHIFT,
    );
    {
        let mut host = harness.host.borrow_mut();
        host.recompute_if_dirty();
        assert!(workbench_control_bool(
            &host,
            "WorkbenchCommandPalette",
            "popup_open"
        ));
    }
    let baseline = harness.journal_len();
    let dispatch = harness.root_ui.dispatch_native_key_for_test(
        key_event(
            Key::Named(NamedKey::Enter),
            PhysicalKey::Code(KeyCode::Enter),
            None,
            ElementState::Pressed,
        ),
        ModifiersState::empty(),
    );

    assert!(
        dispatch.request_redraw(),
        "native popup acceptance should request repaint for the committed row frame"
    );
    assert_eq!(
        harness.delta_events_since(baseline),
        vec![EditorEvent::WorkbenchMenu(MenuAction::OpenProject)]
    );
}

#[test]
fn native_ctrl_n_opens_scene_picker_and_commit_creates_an_empty_project_scene() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_native_new_scene_commit");
    let location = unique_temp_path("zircon_retained_native_new_scene_project").with_extension("");
    fs::create_dir_all(&location).unwrap();
    let manager = harness
        ._core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let admitted = manager
        .create_project_and_open(crate::core::project::NewProjectDraft {
            project_name: "NewSceneCommit".to_string(),
            location: location.to_string_lossy().into_owned(),
            template: crate::core::project::ProjectTemplateId::RenderableEmpty,
        })
        .expect("real project admission should succeed");
    let project_root = admitted.project.as_ref().unwrap().root_path.clone();
    harness
        .host
        .borrow_mut()
        .apply_startup_session(admitted)
        .unwrap();
    harness.root_ui.dispatch_native_key_for_test(
        key_event(
            Key::Character("N".into()),
            PhysicalKey::Code(KeyCode::KeyN),
            Some("N"),
            ElementState::Pressed,
        ),
        ModifiersState::CONTROL,
    );
    let uri = "res://scenes/workbench-review-empty.scene.toml";
    {
        let mut host = harness.host.borrow_mut();
        host.recompute_if_dirty();
        assert!(workbench_control_bool(
            &host,
            "WorkbenchCommandPalette",
            "popup_open"
        ));
        assert_eq!(
            host.runtime.journal().records().last().unwrap().event,
            EditorEvent::WorkbenchMenu(MenuAction::CreateScene)
        );
        host.dispatch_workbench_scene_picker_query_edited(
            "WorkbenchCommandPalette",
            "CommandPalette/QueryChanged",
            uri,
        )
        .expect("real picker query binding should be recognized")
        .expect("project URI should update the picker");
        assert!(
            host.dispatch_workbench_scene_picker_committed(
                "WorkbenchCommandPalette",
                "CommandPalette/Commit",
                "file.scene.create"
            )
            .expect("real Commit binding should be recognized")
            .is_err(),
            "stale command ID must not write a scene"
        );
        assert!(!project_root
            .join("assets/scenes/workbench-review-empty.scene.toml")
            .exists());
        host.dispatch_workbench_scene_picker_committed(
            "WorkbenchCommandPalette",
            "CommandPalette/Commit",
            "scene-picker-create-confirm",
        )
        .expect("real Commit binding should be recognized")
        .expect("normal runtime scene creation should succeed");
        assert!(!workbench_control_bool(
            &host,
            "WorkbenchCommandPalette",
            "popup_open"
        ));
        assert!(
            host.runtime.chrome_snapshot().scene_entries.is_empty(),
            "created scene must have no authored nodes"
        );
    }
    assert!(project_root
        .join("assets/scenes/workbench-review-empty.scene.toml")
        .is_file());
    let close = manager
        .begin_project_close()
        .unwrap()
        .expect("created project should have a close operation");
    manager.commit_project_close(&close).unwrap();
    drop(harness);
    drop(manager);
    fs::remove_dir_all(&location).unwrap();
}
