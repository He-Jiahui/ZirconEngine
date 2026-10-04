use super::*;

#[test]
fn retained_host_presents_reopened_serialized_startup_workspace_before_first_retained_frame() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_serialized_startup_presentation");
    let location =
        unique_temp_path("zircon_retained_serialized_startup_project").with_extension("");
    fs::create_dir_all(&location).expect("temporary project parent should exist");
    let manager = harness
        ._core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .expect("editor manager");

    let mut first_session = manager
        .create_project_and_open(crate::core::project::NewProjectDraft {
            project_name: "SerializedStartup".to_string(),
            location: location.to_string_lossy().into_owned(),
            template: crate::core::project::ProjectTemplateId::RenderableEmpty,
        })
        .expect("the normal project admission should produce a startup document");
    let project_root = first_session
        .project
        .as_ref()
        .expect("admitted startup session carries the prepared project")
        .root_path
        .clone();
    // Create a real split workspace before serialization; the reopened document must carry both
    // scene leaves through the ProjectManager/editor workspace decoder.
    manager
        .open_view(ViewDescriptorId::new("editor.scene"), None)
        .expect("the second Scene view should be admitted by the normal manager route");
    let workspace = manager.project_workspace();
    let project = first_session
        .project
        .as_ref()
        .expect("admitted startup session carries the prepared project");
    manager
        .save_active_scene_with_workspace(&project.root_path, &project.world, &workspace)
        .expect("the scene and opaque editor workspace should serialize together");
    let close = manager
        .begin_project_close()
        .expect("project close should begin")
        .expect("the admitted project has a close operation");
    manager
        .commit_project_close(&close)
        .expect("project close should commit runtime, documents, and workspace ownership");
    manager
        .finalize_project_close(&close)
        .expect("project close should release its Ready guard");

    let reopened = manager
        .open_project_and_remember(&project_root)
        .expect("the normal ProjectManager open should decode the serialized startup document");
    let reopened_project = reopened
        .project
        .as_ref()
        .expect("reopened session carries the decoded project");
    let reopened_workspace = reopened_project
        .editor_workspace
        .as_ref()
        .expect("serialized workspace should be present after reopen");
    let scene_views = reopened_workspace
        .open_view_instances
        .iter()
        .filter(|view| view.descriptor_id.0 == "editor.scene")
        .count();
    assert!(
        scene_views >= 2,
        "serialized reopen must retain both Scene leaves"
    );
    assert!(
        reopened_workspace.focused_view.is_some(),
        "serialized focus must survive reopen"
    );

    harness
        .host
        .borrow_mut()
        .apply_startup_session(reopened)
        .expect("retained host should commit the decoded startup session before presentation");
    let restored_scene_ids = manager
        .current_view_instances()
        .into_iter()
        .filter(|view| view.descriptor_id.0 == "editor.scene")
        .map(|view| crate::core::editor_event::ViewInstanceId::new(view.instance_id.0))
        .collect::<Vec<_>>();
    assert!(
        restored_scene_ids.len() >= 2,
        "normal startup reopen must retain both Scene view identities"
    );
    let restored_sessions = harness
        .host
        .borrow()
        .runtime
        .scene_viewport_workspace_sessions();
    assert!(
        restored_scene_ids
            .iter()
            .all(|view_id| restored_sessions.contains_key(view_id)),
        "normal startup reopen must seed every restored Scene viewport session before retained presentation"
    );
    harness.host.borrow_mut().refresh_ui();
    harness.host.borrow_mut().recompute_if_dirty();

    let presentation = harness.root_ui.get_host_presentation();
    let scene_leaves = presentation
        .host_scene_data
        .document_leaves
        .iter()
        .filter(|leaf| leaf.pane.kind.as_str() == "Scene")
        .collect::<Vec<_>>();
    assert!(
        scene_leaves.len() >= 2,
        "first retained presentation must publish both serialized Scene leaves"
    );
    let host = harness.host.borrow();
    let committed = host
        .committed_shell_state
        .as_ref()
        .expect("startup recompute must retain a committed shell");
    let retained = committed
        .retained_shell_presentation
        .as_ref()
        .expect("startup recompute must retain shell presentation");
    let retained_scene = retained
        .retained_scene_data
        .as_ref()
        .expect("startup recompute must retain HostWindowSceneData");
    assert_eq!(
        retained_scene.document_leaves.len(),
        presentation.host_scene_data.document_leaves.len(),
        "the first retained shell frame must expose the same serialized Scene leaf set"
    );
    assert!(
        retained_scene.document_leaves.len() >= 2,
        "retained Scene projection must preserve independent split leaves"
    );

    harness
        .host
        .borrow_mut()
        .commit_project_close()
        .expect("serialized startup fixture should release its admitted project before cleanup");
    let _ = fs::remove_file(&harness.config_path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn retained_host_rolls_back_after_admitted_scene_activation_failure_and_allows_retry() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_startup_admission_rollback");
    let location = unique_temp_path("zircon_retained_startup_admission_project").with_extension("");
    fs::create_dir_all(&location).expect("temporary project parent should exist");
    let manager = harness
        ._core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .expect("editor manager");
    let welcome = harness
        .host
        .borrow()
        .startup_session
        .welcome_pane_snapshot(false);
    harness
        .host
        .borrow_mut()
        .runtime
        .clear_project(welcome)
        .expect("the fixture should start from the Welcome runtime state");

    let mut admitted = manager
        .create_project_and_open(crate::core::project::NewProjectDraft {
            project_name: "AdmissionRollback".to_string(),
            location: location.to_string_lossy().into_owned(),
            template: crate::core::project::ProjectTemplateId::RenderableEmpty,
        })
        .expect("normal project admission should complete before retained projection");
    let project_root = admitted
        .project
        .as_ref()
        .expect("admitted project document")
        .root_path
        .clone();
    admitted
        .project
        .as_mut()
        .expect("admitted project document")
        .manifest
        .default_scene =
        zircon_runtime::asset::AssetUri::parse("res://missing-after-admission.scene.toml")
            .expect("malformed scene URI fixture");

    let error = harness
        .host
        .borrow_mut()
        .apply_startup_session(admitted)
        .expect_err("scene activation must fail after manager admission and world replacement");
    assert!(
        error.contains("startup session"),
        "rollback should preserve the startup error"
    );
    assert!(
        manager.active_project_session_focus_target().is_none(),
        "failed retained projection must close the admitted Ready session guard"
    );
    assert!(
        !harness.host.borrow().runtime.editor_snapshot().project_open,
        "failed retained projection must restore the previous Welcome runtime"
    );
    assert!(
        manager.active_scene_identity_for_session().is_none(),
        "failed retained projection must not leave a scene journal owner"
    );

    let retry = manager
        .open_project_and_remember(&project_root)
        .expect("the next normal admission must be possible after rollback");
    assert!(
        retry.project.is_some(),
        "retry should receive a prepared project document"
    );
    let close = manager
        .begin_project_close()
        .expect("retry close should begin")
        .expect("retry owns an admitted close operation");
    manager
        .commit_project_close(&close)
        .expect("retry close should commit");
    manager
        .finalize_project_close(&close)
        .expect("retry close should release the session guard");
    let _ = fs::remove_dir_all(project_root);
}
