#![cfg(windows)]

use std::time::{Duration, Instant};

use super::*;
use crate::core::editing::command::EditorCommand;
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::{HistoryContextId, HistoryStatus};
use crate::core::editing::intent::EditorIntent;
use crate::core::editor_event::{EditorEvent, EditorEventSource, MenuAction};
use crate::core::gateway::InProcessGateway;
use crate::core::notifications::{DecisionOptionId, DecisionTicket};
use crate::core::play::{PlayKind, WorldDomain};
use crate::core::project::{NewProjectDraft, ProjectTemplateId, SceneCreateRequest};
use crate::scene::modes::SceneModeActivation;
use crate::scene::selection::SelectionMutation;
use crate::scene::viewport::{
    HandleElementExtract, OverlayAxis, ProjectionMode, TransformHandleKind,
};
use crate::ui::binding::ViewportCommand;
use crate::ui::host::module;
use crate::ui::host::EditorHostEventController;
use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::foundation::{
    module_descriptor as foundation_module_descriptor, FOUNDATION_MODULE_NAME,
};
use zircon_runtime::scene::{components::NodeKind, DefaultLevelManager};
use zircon_runtime_interface::math::{Transform, Vec2};

struct ReloadHarness {
    host: RetainedEditorHost,
    _core: CoreRuntime,
    _files: FixtureFiles,
}

struct FixtureFiles {
    root: std::path::PathBuf,
    config: std::path::PathBuf,
}

impl Drop for FixtureFiles {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
        let _ = std::fs::remove_file(&self.config);
    }
}

impl ReloadHarness {
    fn new(prefix: &str) -> Self {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("{prefix}_{unique}"));
        let config = root.with_extension("json");
        std::env::set_var("ZIRCON_CONFIG_PATH", &config);
        let core = CoreRuntime::new();
        core.register_module(foundation_module_descriptor())
            .unwrap();
        core.register_module(zircon_runtime::asset::module_descriptor())
            .unwrap();
        core.register_module(zircon_runtime::scene::module_descriptor())
            .unwrap();
        core.register_module(module::module_descriptor()).unwrap();
        core.activate_module(FOUNDATION_MODULE_NAME).unwrap();
        core.activate_module(zircon_runtime::asset::ASSET_MODULE_NAME)
            .unwrap();
        core.activate_module(zircon_runtime::core::framework::scene::SCENE_MODULE_NAME)
            .unwrap();
        core.activate_module(module::EDITOR_MODULE_NAME).unwrap();
        crate::tests::support::configure_editor_test_runtime_build_set(&core);
        std::env::remove_var("ZIRCON_CONFIG_PATH");
        let ui = UiHostWindow::new().unwrap();
        let mut host = RetainedEditorHost::new_for_test(core.handle(), ui).unwrap();
        let created = ProjectAuthority::default()
            .create_project(
                &NewProjectDraft {
                    project_name: "ReloadFixture".to_owned(),
                    location: root.to_string_lossy().into_owned(),
                    template: ProjectTemplateId::RenderableEmpty,
                },
                &crate::tests::support::test_project_creation_provenance(),
            )
            .unwrap();
        let mut project = ProjectManager::open(&created.root).unwrap();
        let secondary = ResourceLocator::parse("res://scenes/reload-other.scene.toml").unwrap();
        ProjectAuthority::default()
            .create_scene(&mut project, SceneCreateRequest::new(secondary))
            .unwrap();
        host.editor_manager.open_project(&created.root).unwrap();
        let (project, _) = host
            .asset_runtime_access
            .project_asset_manager()
            .unwrap()
            .current_project_generation_snapshot()
            .unwrap()
            .into_parts();
        let manager = Arc::clone(&host.editor_manager);
        let world = manager
            .prepare_authoring_world(
                DefaultLevelManager::default()
                    .create_default_level()
                    .with_world(Clone::clone),
            )
            .unwrap();
        let mut state = EditorState::project_with_context(
            world,
            UVec2::new(1280, 720),
            project.paths().root().to_string_lossy(),
            Arc::clone(manager.context()),
        );
        let document = manager
            .activate_startup_scene_document(
                project.paths().root(),
                &project.manifest().default_scene,
            )
            .unwrap();
        state.bind_scene_document(document);
        host.runtime = EditorHostEventController::new(state, manager);
        Self {
            host,
            _core: core,
            _files: FixtureFiles { root, config },
        }
    }

    fn rename(&self, name: &str) {
        let mut shell = self.host.runtime.shell().lock();
        let command = shell.state.world.expect_with_world(|scene| {
            let node = scene
                .nodes()
                .iter()
                .find(|node| matches!(node.kind, NodeKind::Cube))
                .unwrap()
                .id;
            EditorCommand::rename_node(scene, node, name.to_owned())
                .unwrap()
                .unwrap()
        });
        shell
            .state
            .execute_scene_command("reload regression edit", command)
            .unwrap();
    }

    fn name(&self) -> String {
        self.host
            .runtime
            .shell()
            .lock()
            .state
            .world
            .expect_with_world(|scene| {
                scene
                    .nodes()
                    .iter()
                    .find(|node| matches!(node.kind, NodeKind::Cube))
                    .unwrap()
                    .name
                    .clone()
            })
    }

    fn history(&self) -> HistoryStatus {
        let history = self.host.runtime.active_scene_history_context().unwrap();
        self.host
            .runtime
            .context()
            .transactions()
            .history_status(history)
            .unwrap()
    }

    fn decision(&self) -> DecisionTicket {
        self.host
            .runtime
            .context()
            .notifications()
            .decisions()
            .unwrap()
            .snapshot()
            .into_iter()
            .find(|snapshot| {
                snapshot.resolved().is_none()
                    && snapshot
                        .ticket()
                        .notification_id()
                        .as_str()
                        .starts_with("editor.scene.active_reload_conflict.")
            })
            .expect("reload conflict should publish a pending decision")
            .ticket()
            .clone()
    }

    fn prompt(&mut self) -> DecisionTicket {
        let identity = self
            .host
            .editor_manager
            .active_scene_identity_for_session()
            .unwrap();
        let (_, generation) = self
            .host
            .asset_runtime_access
            .project_asset_manager()
            .unwrap()
            .current_project_generation_snapshot()
            .unwrap()
            .into_parts();
        self.host
            .install_active_scene_reload_conflict(identity, generation);
        self.decision()
    }

    fn choose(&mut self, ticket: &DecisionTicket, option: &str) {
        self.host
            .runtime
            .context()
            .notifications()
            .decisions()
            .unwrap()
            .resolve(ticket, &DecisionOptionId::parse(option).unwrap())
            .unwrap();
        self.host.reconcile_active_scene_reload_conflict();
    }

    fn finish_reload(&mut self) {
        assert!(self.host.pending_active_scene_reload.is_some());
        let deadline = Instant::now() + Duration::from_secs(5);
        while self.host.pending_active_scene_reload.is_some() {
            assert!(Instant::now() < deadline, "scene load job should complete");
            self.host.poll_active_scene_reload();
            std::thread::yield_now();
        }
    }

    fn transform(&self, node: u64) -> Transform {
        self.host
            .runtime
            .shell()
            .lock()
            .state
            .world
            .expect_with_world(|scene| scene.find_node(node).unwrap().transform)
    }

    fn begin_preview(&self) -> (u64, Transform) {
        let mut shell = self.host.runtime.shell().lock();
        let state = &mut shell.state;
        let (node, before) = state.world.expect_with_world(|scene| {
            let node = scene
                .nodes()
                .iter()
                .find(|node| matches!(node.kind, NodeKind::Cube))
                .unwrap();
            (node.id, node.transform)
        });
        state
            .apply_viewport_command(&ViewportCommand::ActivateSceneMode(
                SceneModeActivation::Transform(TransformHandleKind::Move),
            ))
            .unwrap();
        let packet = state.render_snapshot().unwrap();
        let handle = packet
            .overlays
            .handles
            .iter()
            .find(|handle| handle.owner == node)
            .unwrap();
        let (start, end) = handle
            .elements
            .iter()
            .find_map(|element| match element {
                HandleElementExtract::AxisLine {
                    axis: OverlayAxis::X,
                    start,
                    end,
                    ..
                } => Some((*start, *end)),
                _ => None,
            })
            .unwrap();
        let camera = &packet.scene.camera;
        assert_eq!(camera.projection_mode, ProjectionMode::Perspective);
        let size = state.viewport_state().size;
        let view_projection = zircon_runtime_interface::math::perspective(
            camera.fov_y_radians,
            size.x as f32 / size.y.max(1) as f32,
            camera.z_near,
            camera.z_far,
        ) * zircon_runtime_interface::math::view_matrix(camera.transform);
        let project = |world: zircon_runtime_interface::math::Vec3| {
            let clip = view_projection * world.extend(1.0);
            assert!(clip.w > f32::EPSILON);
            let ndc = clip.truncate() / clip.w;
            Vec2::new(
                (ndc.x * 0.5 + 0.5) * size.x as f32,
                (-ndc.y * 0.5 + 0.5) * size.y as f32,
            )
        };
        let start = project(start);
        let direction = (project(end) - start).normalize_or_zero();
        let press = start + direction * 24.0;
        let moved = press + direction * 96.0;
        state
            .apply_viewport_command(&ViewportCommand::LeftPressed {
                x: press.x,
                y: press.y,
                selection_mutation: SelectionMutation::Replace,
            })
            .unwrap();
        state
            .apply_viewport_command(&ViewportCommand::PointerMoved {
                x: moved.x,
                y: moved.y,
            })
            .unwrap();
        assert!(state.has_active_gizmo_interaction());
        assert_ne!(
            state
                .world
                .expect_with_world(|scene| scene.find_node(node).unwrap().transform),
            before
        );
        (node, before)
    }

    fn await_deferred_reload(&mut self) {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let pending = self
                .host
                .pending_active_scene_reload
                .as_ref()
                .expect("a blocked installation must retain its pending reload");
            if pending.prepared.is_some() {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "scene load should reach deferred installation"
            );
            self.host.poll_active_scene_reload();
            std::thread::yield_now();
        }
    }

    fn end_preview(&self, command: ViewportCommand) {
        let mut shell = self.host.runtime.shell().lock();
        shell.state.apply_viewport_command(&command).unwrap();
        assert!(!shell.state.has_active_gizmo_interaction());
    }
}

#[test]
fn discard_after_a_new_edit_since_the_prompt_requires_a_new_decision() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let mut harness = ReloadHarness::new("reload_discard_prompt_generation");
    harness.rename("edit shown by prompt");
    let ticket = harness.prompt();
    harness.rename("new edit before decision");
    let before = harness.history();

    harness.choose(&ticket, "discard");

    assert!(harness.host.pending_active_scene_reload.is_none());
    assert_ne!(harness.decision(), ticket);
    assert_eq!(harness.name(), "new edit before decision");
    assert_eq!(harness.history(), before);
    let refreshed = harness.decision();
    harness.choose(&refreshed, "keep_editing");
    assert!(harness.host.pending_active_scene_reload.is_none());
    assert_eq!(harness.history(), before);
}

#[test]
fn an_edit_after_discard_submission_survives_real_scene_job_completion() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let mut harness = ReloadHarness::new("reload_discard_pending_generation");
    harness.rename("authorized edit");
    let ticket = harness.prompt();
    harness.choose(&ticket, "discard");
    assert!(harness.host.pending_active_scene_reload.is_some());
    harness.rename("edit while scene load is pending");
    let before = harness.history();
    let selection = harness
        .host
        .runtime
        .context()
        .transactions()
        .with_context::<CoreEditContext, _>(CoreEditContext::selection_snapshot)
        .unwrap()
        .unwrap();

    harness.finish_reload();

    assert_ne!(harness.decision(), ticket);
    assert_eq!(harness.name(), "edit while scene load is pending");
    assert_eq!(harness.history(), before);
    assert_eq!(
        harness
            .host
            .runtime
            .context()
            .transactions()
            .with_context::<CoreEditContext, _>(CoreEditContext::selection_snapshot)
            .unwrap()
            .unwrap(),
        selection
    );
    assert!(harness
        .host
        .runtime
        .shell()
        .lock()
        .state
        .apply_intent(EditorIntent::Undo)
        .unwrap());
    assert_eq!(harness.name(), "authorized edit");
}

#[test]
fn unchanged_discard_authorization_allows_the_real_scene_job_to_commit() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let mut harness = ReloadHarness::new("reload_discard_unchanged_generation");
    harness.rename("authorized edit");
    let identity = harness
        .host
        .editor_manager
        .active_scene_identity_for_session()
        .unwrap();
    let ticket = harness.prompt();
    harness.choose(&ticket, "discard");

    harness.finish_reload();

    assert!(harness.host.active_scene_reload_conflict.is_none());
    assert!(!harness.history().dirty);
    assert!(!harness.history().can_undo);
    assert_eq!(
        harness
            .host
            .editor_manager
            .active_scene_identity_for_session(),
        Some(identity)
    );
}

#[test]
fn a_scene_activation_supersedes_the_pending_discard_without_clearing_history() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let mut harness = ReloadHarness::new("reload_discard_scene_supersession");
    harness.rename("authorized edit");
    let ticket = harness.prompt();
    harness.choose(&ticket, "discard");
    let before = harness.history();
    let (node, _) = harness.begin_preview();
    let preview = harness.transform(node);
    harness.await_deferred_reload();
    let (project, _) = harness
        .host
        .asset_runtime_access
        .project_asset_manager()
        .unwrap()
        .current_project_generation_snapshot()
        .unwrap()
        .into_parts();
    let secondary = ResourceLocator::parse("res://scenes/reload-other.scene.toml").unwrap();
    let default = project.manifest().default_scene.clone();
    harness
        .host
        .editor_manager
        .activate_startup_scene_document(project.paths().root(), &secondary)
        .unwrap();
    let document = harness
        .host
        .editor_manager
        .activate_startup_scene_document(project.paths().root(), &default)
        .unwrap();
    harness.host.runtime.bind_scene_document(document);

    harness.finish_reload();

    assert_eq!(harness.name(), "authorized edit");
    assert_eq!(harness.transform(node), preview);
    assert_eq!(harness.history(), before);
    harness.end_preview(ViewportCommand::CancelInteraction);
}

#[test]
fn a_project_generation_supersedes_the_pending_discard_without_mutating_the_world() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let mut harness = ReloadHarness::new("reload_discard_project_supersession");
    harness.rename("authorized edit");
    let ticket = harness.prompt();
    harness.choose(&ticket, "discard");
    let history = harness.host.runtime.active_scene_history_context().unwrap();
    let before = harness.history();
    let (node, _) = harness.begin_preview();
    let preview = harness.transform(node);
    harness.await_deferred_reload();
    let close = harness
        .host
        .editor_manager
        .begin_project_close()
        .unwrap()
        .unwrap();
    harness
        .host
        .editor_manager
        .commit_project_close(&close)
        .unwrap();

    harness.finish_reload();

    assert_eq!(harness.name(), "authorized edit");
    assert_eq!(harness.transform(node), preview);
    assert_eq!(
        harness
            .host
            .runtime
            .context()
            .transactions()
            .history_status(history)
            .unwrap(),
        before
    );
    harness.end_preview(ViewportCommand::CancelInteraction);
    harness
        .host
        .editor_manager
        .finalize_project_close(&close)
        .unwrap();
}

#[test]
fn a_gizmo_preview_survives_completion_and_requires_a_new_decision_after_commit() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let mut harness = ReloadHarness::new("reload_discard_preview_commit");
    harness.rename("authorized edit");
    let ticket = harness.prompt();
    harness.choose(&ticket, "discard");
    let before_history = harness.history();
    let job = harness
        .host
        .pending_active_scene_reload
        .as_ref()
        .unwrap()
        .ticket
        .id();
    let (node, initial) = harness.begin_preview();
    let preview = harness.transform(node);
    assert_eq!(harness.history(), before_history);

    harness.await_deferred_reload();
    for _ in 0..3 {
        harness.host.poll_active_scene_reload();
        let pending = harness.host.pending_active_scene_reload.as_ref().unwrap();
        assert_eq!(pending.ticket.id(), job);
        assert!(pending.prepared.is_some());
        assert_eq!(harness.transform(node), preview);
        assert_eq!(harness.history(), before_history);
        assert!(harness
            .host
            .runtime
            .shell()
            .lock()
            .state
            .has_active_gizmo_interaction());
    }
    harness.end_preview(ViewportCommand::LeftReleased);
    let committed = harness.history();
    assert!(committed.generation > before_history.generation);
    harness.finish_reload();

    assert_ne!(harness.decision(), ticket);
    assert_eq!(harness.transform(node), preview);
    assert_eq!(harness.history(), committed);
    assert!(harness
        .host
        .runtime
        .shell()
        .lock()
        .state
        .apply_intent(EditorIntent::Undo)
        .unwrap());
    assert_eq!(harness.transform(node), initial);
}

#[test]
fn cancelling_a_preview_resumes_the_same_authorized_prepared_reload() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let mut harness = ReloadHarness::new("reload_discard_preview_cancel");
    harness.rename("authorized edit");
    let ticket = harness.prompt();
    harness.choose(&ticket, "discard");
    let job = harness
        .host
        .pending_active_scene_reload
        .as_ref()
        .unwrap()
        .ticket
        .id();
    let (node, initial) = harness.begin_preview();
    harness.await_deferred_reload();
    assert_eq!(
        harness
            .host
            .pending_active_scene_reload
            .as_ref()
            .unwrap()
            .ticket
            .id(),
        job
    );

    harness.end_preview(ViewportCommand::CancelInteraction);
    assert_eq!(harness.transform(node), initial);
    harness.finish_reload();

    assert!(harness.host.active_scene_reload_conflict.is_none());
    assert!(!harness.history().dirty);
    assert!(!harness.history().can_undo);
}

#[test]
fn a_clean_scene_preview_defers_the_same_job_until_the_interaction_ends() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let mut harness = ReloadHarness::new("reload_clean_preview_deferred");
    harness.host.request_active_scene_reload().unwrap();
    let job = harness
        .host
        .pending_active_scene_reload
        .as_ref()
        .unwrap()
        .ticket
        .id();
    let before = harness.history();
    assert!(!before.dirty);
    let (node, _) = harness.begin_preview();
    let preview = harness.transform(node);
    harness.await_deferred_reload();
    for _ in 0..3 {
        harness.host.poll_active_scene_reload();
        assert_eq!(
            harness
                .host
                .pending_active_scene_reload
                .as_ref()
                .unwrap()
                .ticket
                .id(),
            job
        );
        assert!(harness.host.active_scene_reload_conflict.is_none());
        assert_eq!(harness.transform(node), preview);
        assert_eq!(harness.history(), before);
    }

    harness.end_preview(ViewportCommand::CancelInteraction);
    harness.finish_reload();

    assert!(harness.host.active_scene_reload_conflict.is_none());
    assert!(!harness.history().dirty);
}

#[test]
fn play_and_simulate_keep_pending_authoring_reload_out_of_both_history_domains() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    for kind in [PlayKind::Play, PlayKind::Simulate] {
        let mut harness = ReloadHarness::new("reload_authoring_during_play");
        harness.rename("dirty authoring edit");
        let document = harness.host.runtime.active_scene_history_context().unwrap();
        harness.host.request_active_scene_reload().unwrap();
        let job = harness
            .host
            .pending_active_scene_reload
            .as_ref()
            .unwrap()
            .ticket
            .id();
        let play_level = DefaultLevelManager::default().create_default_level();
        let instance = harness
            .host
            .runtime
            .start_test_play_gateway(
                kind,
                Arc::new(InProcessGateway::for_authoring_level(play_level)),
            )
            .unwrap();
        harness
            .host
            .runtime
            .shell()
            .lock()
            .state
            .enter_play_mode()
            .unwrap();
        assert!(harness.host.runtime.sync_active_selection_world_domain());
        let play_history = HistoryContextId::PlaySession(instance);
        {
            let mut edit = harness
                .host
                .runtime
                .context()
                .transactions()
                .begin("real Play world edit", play_history)
                .unwrap();
            edit.push(EditorCommand::create_node(NodeKind::Cube))
                .unwrap();
            edit.commit().unwrap();
        }
        let authoring_before = harness
            .host
            .runtime
            .context()
            .transactions()
            .history_status(document)
            .unwrap();
        let play_before = harness
            .host
            .runtime
            .context()
            .transactions()
            .history_status(play_history)
            .unwrap();
        assert!(authoring_before.dirty);
        assert!(!play_before.dirty);
        assert!(play_before.can_undo);

        harness.await_deferred_reload();
        for domain in [WorldDomain::Play(instance), WorldDomain::Edit] {
            harness
                .host
                .runtime
                .shell()
                .lock()
                .state
                .sync_selection_world_domain(domain);
            harness.host.poll_active_scene_reload();
            let pending = harness.host.pending_active_scene_reload.as_ref().unwrap();
            assert_eq!(pending.ticket.id(), job);
            assert!(pending.prepared.is_some());
            assert!(harness.host.active_scene_reload_conflict.is_none());
            assert_eq!(harness.name(), "dirty authoring edit");
            assert_eq!(
                harness
                    .host
                    .runtime
                    .context()
                    .transactions()
                    .history_status(document)
                    .unwrap(),
                authoring_before
            );
            assert_eq!(
                harness
                    .host
                    .runtime
                    .context()
                    .transactions()
                    .history_status(play_history)
                    .unwrap(),
                play_before
            );
        }
        let stop = harness
            .host
            .runtime
            .dispatch_event(
                EditorEventSource::RetainedHost,
                EditorEvent::WorkbenchMenu(MenuAction::ExitPlayMode),
            )
            .unwrap();
        assert!(stop.result.error.is_none(), "{stop:?}");
        assert!(!harness.host.runtime.shell().lock().state.is_playing());
        harness.finish_reload();

        assert!(harness.host.active_scene_reload_conflict.is_some());
        let _decision = harness.decision();
        assert_eq!(harness.name(), "dirty authoring edit");
        assert_eq!(
            harness
                .host
                .runtime
                .context()
                .transactions()
                .history_status(document)
                .unwrap(),
            authoring_before
        );
        assert!(harness
            .host
            .runtime
            .shell()
            .lock()
            .state
            .apply_intent(EditorIntent::Undo)
            .unwrap());
        assert_ne!(harness.name(), "dirty authoring edit");
    }
}
