use crate::ui::retained_host::FrameRect;

use super::*;

#[test]
#[ignore = "requires a preregistered physical Editor fixture root"]
fn r21_boundary_same_root_new_activation_exits_through_native_close() {
    let f = Fixture::cold();
    let a1 = f.host.ready(&f.output.join("ProjectA"));
    let first_generation = persisted_generation(&a1);
    f.host.close_project_to_welcome(&a1); // Includes the actual first lock release.
    f.open_from_welcome("ProjectA"); // Reuse the existing directory, never create it again.
    let a2 = f.host.ready(a1.project_root());
    assert_eq!(a1.project_root(), a2.project_root());
    assert_ne!(a1.operation_id(), a2.operation_id());
    assert_ne!(first_generation, persisted_generation(&a2));
    eprintln!("R21_BOUNDARY_SAME_ROOT_BEFORE_X old={a1:?} new={a2:?}");
    f.request_root_x();
    assert!(f.host.retained.borrow().pending_close_prompt.is_none());
    assert!(!f.host.ui.get_host_presentation().close_prompt.visible);
    f.await_owned_exit(&a1, &a2);
}

#[test]
#[ignore = "requires a preregistered physical Editor fixture root"]
fn r21_boundary_dirty_second_project_cancel_preserves_ready_scene_and_history() {
    let f = Fixture::cold();
    let a = f.host.ready(&f.output.join("ProjectA"));
    let b = ProjectAuthority::default()
        .create_project(
            &NewProjectDraft {
                project_name: "ProjectB".into(),
                location: f.location(),
                template: ProjectTemplateId::RenderableEmpty,
            },
            &test_project_creation_provenance(),
        )
        .expect("fixture: prepare B on disk without opening a second host");
    f.host.close_project_to_welcome(&a);
    f.open_from_welcome("ProjectB");
    let b = f.host.ready(&b.root);
    assert_ne!(a.project_root(), b.project_root());
    assert_ne!(a.operation_id(), b.operation_id());
    assert_released(&a);
    let b_generation = persisted_generation(&b);
    let scene_path = b.project_root().join("assets/scenes/main.scene.toml");
    let saved = fs::read(&scene_path).expect("fixture: template startup Scene source");
    let (history, dirty, token, nodes) = {
        let mut host = f.host.retained.borrow_mut();
        let history = host.runtime.active_scene_history_context().unwrap();
        let baseline = host
            .runtime
            .context()
            .transactions()
            .history_status(history)
            .unwrap();
        assert!(!baseline.dirty);
        let node_count = host
            .runtime
            .project_scene_snapshot()
            .unwrap()
            .unwrap()
            .entity_count();
        // Use the registered Workbench Add > Cube operation and commit the real Scene history.
        let effects = callback_dispatch::dispatch_menu_action(
            &host.runtime,
            "workbench.scene.node.create.cube",
        )
        .expect("fixture: genuine Create Cube menu action in B");
        host.apply_dispatch_effects(effects);
        host.refresh_ui();
        let dirty = host
            .runtime
            .context()
            .transactions()
            .history_status(history)
            .unwrap();
        assert!(dirty.dirty && dirty.len == baseline.len + 1 && dirty.top != baseline.top);
        let token = host
            .dirty_project_scene_token()
            .unwrap()
            .expect("real dirty Scene decision");
        let scene = host.runtime.project_scene_snapshot().unwrap().unwrap();
        assert_eq!(scene.entity_count(), node_count + 1);
        (history, dirty, token, scene.node_records())
    };
    eprintln!("R21_BOUNDARY_DIRTY_BEFORE_X target={b:?} history={dirty:?}");
    let prompt = f.dirty_root_x();
    f.click(&prompt.cancel_button_frame);
    f.host.retained.borrow_mut().tick();
    {
        let host = f.host.retained.borrow();
        assert!(host.pending_close_prompt.is_none());
        assert!(!host.main_window_close_requested());
        assert!(!host.main_window_close_blocks_session_ui());
        assert_eq!(host.runtime.active_scene_history_context(), Some(history));
        assert_eq!(
            host.runtime
                .context()
                .transactions()
                .history_status(history)
                .unwrap(),
            dirty
        );
        assert_eq!(host.dirty_project_scene_token().unwrap(), Some(token));
        assert_eq!(
            host.runtime
                .project_scene_snapshot()
                .unwrap()
                .unwrap()
                .node_records(),
            nodes
        );
        let manager = host.editor_manager.enter().unwrap();
        assert_eq!(
            manager.project_close_activation_identity().as_ref(),
            Some(&b)
        );
        assert!(manager.project_session_heartbeat_deadline().is_some());
    }
    assert!(!f.host.ui.get_host_presentation().close_prompt.visible);
    assert!(!f.host.ui.exit_requested_for_test());
    assert_persisted_ready(&b);
    assert_eq!(persisted_generation(&b), b_generation);
    assert_released(&a);
    assert_eq!(
        fs::read(&scene_path).unwrap(),
        saved,
        "Cancel must not save B's Scene"
    );
    eprintln!("R21_BOUNDARY_DIRTY_CANCEL_READY target={b:?}");

    // Now make a new, ordinary close decision to retire the disposable B cleanly.
    let prompt = f.dirty_root_x();
    f.click(&prompt.discard_button_frame);
    assert!(f.host.retained.borrow().pending_close_prompt.is_none());
    f.await_owned_exit(&a, &b);
}

struct Fixture {
    host: ColdHost,
    output: PathBuf,
    _restore: [RestoreEnv; 3],
    _lock: std::sync::MutexGuard<'static, ()>,
}

impl Fixture {
    fn cold() -> Self {
        let lock = lock_env();
        let output = fixture_root();
        let restore =
            ["ZIRCON_CONFIG_PATH", "USERPROFILE", SETTINGS_USER_ROOT_ENV].map(RestoreEnv::capture);
        std::env::set_var("ZIRCON_CONFIG_PATH", output.join("config.json"));
        std::env::set_var("USERPROFILE", output.join("home"));
        std::env::set_var(SETTINGS_USER_ROOT_ENV, output.join("settings"));
        let host = ColdHost::new(&output);
        Self {
            host,
            output,
            _restore: restore,
            _lock: lock,
        }
    }

    fn location(&self) -> String {
        ProjectPaths::display_path(&self.output)
            .to_string_lossy()
            .into_owned()
    }

    fn open_from_welcome(&self, name: &str) {
        let location = self.location();
        let pane = pane_surface_host(&self.host.ui);
        pane.invoke_welcome_control_changed(
            "welcome.project.location.edit".into(),
            location.as_str().into(),
        );
        pane.invoke_welcome_control_changed("welcome.project.name.edit".into(), name.into());
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            self.host.retained.borrow_mut().tick();
            let welcome = pane.get_welcome_pane();
            if welcome.form.can_open_existing
                && welcome.form.project_name.as_str() == name
                && welcome.form.location.as_str() == location.as_str()
                && welcome.nodes.iter().any(|node| {
                    node.control_id.as_str() == "WelcomeOpenExistingButton"
                        && node.action_id.as_str() == "welcome.project.open_existing"
                        && !node.disabled
                })
            {
                pane.invoke_welcome_control_clicked("welcome.project.open_existing".into());
                self.host.retained.borrow_mut().tick();
                return;
            }
            assert!(
                Instant::now() < deadline,
                "fixture: Welcome cannot open {name}: {}",
                self.host
                    .retained
                    .borrow()
                    .startup_session
                    .creation_validation
            );
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    fn request_root_x(&self) {
        assert_eq!(
            self.host.ui.dispatch_native_close_request_for_test(),
            CloseRequestResponse::KeepWindowShown
        );
    }

    fn dirty_root_x(&self) -> crate::ui::retained_host::HostClosePromptData {
        self.request_root_x();
        let prompt = self.host.ui.get_host_presentation().close_prompt;
        assert!(prompt.visible && prompt.can_save);
        assert_eq!(prompt.target_window_id.as_str(), "main");
        assert_eq!(prompt.title.as_str(), "Save changes before closing Zircon?");
        assert_eq!(prompt.details.as_str(), "Active Scene");
        for button in [
            &prompt.save_button_frame,
            &prompt.discard_button_frame,
            &prompt.cancel_button_frame,
        ] {
            assert!(
                button.x.is_finite()
                    && button.y.is_finite()
                    && button.width > 0.0
                    && button.height > 0.0
            );
        }
        assert!(!self.host.ui.exit_requested_for_test());
        prompt
    }

    fn click(&self, frame: &FrameRect) {
        let result = self.host.ui.dispatch_native_primary_press_for_test(
            frame.x + frame.width * 0.5,
            frame.y + frame.height * 0.5,
        );
        assert!(result.request_redraw());
    }

    fn await_owned_exit(&self, retired: &ProjectCloseOperation, target: &ProjectCloseOperation) {
        assert!(self.host.retained.borrow().main_window_close_requested());
        let deadline = Instant::now() + Duration::from_secs(6);
        while !self.host.ui.exit_requested_for_test() {
            self.host.retained.borrow_mut().tick();
            let host = self.host.retained.borrow();
            let status = host.runtime.status_line();
            if status == STALE_OWNER {
                let manager = host.editor_manager.enter().unwrap();
                assert_eq!(host.project_close_coordinator.phase(), Phase::Closed);
                assert_eq!(host.project_close_coordinator.operation(), Some(retired));
                assert_eq!(
                    manager.project_close_activation_identity().as_ref(),
                    Some(target)
                );
                assert!(host.main_window_close_blocks_session_ui());
                assert!(!self.host.ui.exit_requested_for_test());
                assert_persisted_ready(target);
                // On a same-root replacement, the on-disk filename now belongs to the new operation.
                if retired.project_root() != target.project_root() {
                    assert_released(retired);
                }
                panic!("R21_BOUNDARY_STALE_OWNER: {status} retired={retired:?} target={target:?}");
            }
            assert!(
                Instant::now() < deadline,
                "close did not complete (not boundary stale-owner RED): {status}"
            );
            drop(host);
            std::thread::sleep(Duration::from_millis(5));
        }
        let host = self.host.retained.borrow();
        assert!(host.main_window_close_complete());
        assert!(host
            .editor_manager
            .enter()
            .unwrap()
            .project_close_activation_identity()
            .is_none());
        assert_released(target);
        assert_released(retired);
    }
}

fn persisted_generation(operation: &ProjectCloseOperation) -> u64 {
    let payload = fs::read_to_string(project_session_lock_path(operation.project_root())).unwrap();
    let record = decode_project_session_admission_record(&payload).unwrap();
    assert_eq!(record.operation_id(), operation.operation_id());
    assert_eq!(record.lifecycle(), Lifecycle::Ready);
    record.session_generation().unwrap().get()
}
