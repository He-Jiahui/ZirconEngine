#![cfg(windows)]

use super::super::support::*;
use crate::core::editing::engine::{HistoryContextId, HistoryStatus};
use crate::core::editor_event::EditorEventSource;
use crate::core::gateway::{DetachedEditorRuntimeGateway, GatewaySessionIdentity};
use crate::core::play::WorldDomain;
use crate::core::project::{ProjectAuthority, ProjectTemplateId};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime::core::framework::scene::Mobility;
use zircon_runtime::scene::{components::NodeKind, NodeId, Scene};
use zircon_runtime_interface::math::Transform;
use zircon_runtime_interface::project::{
    ProjectLaunchIntent, ProjectLaunchProfile, ProjectLaunchSource,
};
use zircon_runtime_interface::ui::binding::UiBindingValue;

const X: &str = "WorkbenchTransformPositionX";
const Y: &str = "WorkbenchTransformPositionY";
const UNDO: &str = "workbench.history.undo";
const REDO: &str = "workbench.history.redo";

#[test]
#[ignore = "requires an isolated physical Editor fixture and user profile"]
fn details_enter_static_cube_rejects_x_edit_without_history() {
    let _guard = lock_env();
    let f = DetailsFixture::static_cube();
    f.assert_static_axis_rejected(X, "12.3456");
    f.finish();
}

#[test]
#[ignore = "requires an isolated physical Editor fixture and user profile"]
fn details_enter_dynamic_camera_untouched_preserves_saved_precision_and_redo() {
    let _guard = lock_env();
    exercise_untouched(DetailsFixture::dynamic_camera());
}

fn exercise_untouched(f: DetailsFixture) {
    let (mut exact, _) = f.observe();
    exact.translation.x = 12.3456;
    f.type_axis(X, "12.3456");
    assert_eq!(f.observe().0, exact);
    f.harness.dispatch_menu_action("workbench.project.save");
    let (_, saved) = f.observe();
    assert!(saved.top.is_some() && saved.top == saved.saved_top && !saved.dirty);
    let (saved_x, saved_bytes) = f.read_saved_source();
    assert_eq!(saved_x.to_bits(), 12.3456_f32.to_bits());

    // A second genuine UI edit leaves a distinct, testable Redo target.
    f.type_axis(Y, "6.25");
    let mut moved = exact;
    moved.translation.y = 6.25;
    let (actual_moved, edited) = f.observe();
    assert_eq!(actual_moved, moved);
    assert!(edited.dirty && edited.top != saved.top && edited.len == saved.len + 1);
    f.harness.dispatch_menu_action(UNDO);
    let (after_undo, before) = f.observe();
    assert_eq!(after_undo, exact);
    assert!(before.can_redo && !before.dirty && before.top == saved.top);
    assert_eq!(f.axis(X).value_text.as_str(), "12.35");

    f.focus(X);
    f.focus(Y); // Clicking the other axis changes focus, not document state.
    assert_eq!(f.observe(), (exact, before));
    f.focus(X);
    let focus = host_context(&f.harness.root_ui).get_text_input_focus();
    assert_eq!(focus.value_text.as_str(), "12.35");
    assert_eq!(f.observe(), (exact, before));
    f.enter(); // No caret change or text input since pointer focus.
    assert_eq!(
        f.observe(),
        (exact, before),
        "DETAILS_UNTOUCHED_ENTER_AUTHORING: exact X and its saved history before replay"
    );
    assert_eq!(f.axis(X).value_text.as_str(), "12.35");
    assert_eq!(f.read_saved_source().1, saved_bytes);

    // Selection and caret keys alone must not qualify as text edits either.
    f.focus(X);
    f.key(
        Key::Character("a".into()),
        KeyCode::KeyA,
        None,
        ModifiersState::CONTROL,
    );
    f.key(
        Key::Named(NamedKey::ArrowRight),
        KeyCode::ArrowRight,
        None,
        ModifiersState::empty(),
    );
    f.enter();
    assert_eq!(
        f.observe(),
        (exact, before),
        "caret movement is not authoring"
    );
    f.harness.dispatch_menu_action(REDO);
    let (after_redo, history_after_redo) = f.observe();
    assert_eq!((after_redo, history_after_redo.top), (moved, edited.top));
    f.harness.dispatch_menu_action(UNDO);
    f.finish();
}

#[test]
#[ignore = "requires an isolated physical Editor fixture and user profile"]
fn details_enter_static_cube_rejects_y_and_x_edits_without_history() {
    let _guard = lock_env();
    let f = DetailsFixture::static_cube();
    f.assert_static_axis_rejected(Y, "2.5436");
    f.assert_static_axis_rejected(X, "12.3456");
    f.finish();
}

#[test]
#[ignore = "requires an isolated physical Editor fixture and user profile"]
fn details_enter_dynamic_camera_explicitly_retyping_displayed_value_is_a_real_edit() {
    let _guard = lock_env();
    exercise_explicit_retype(DetailsFixture::dynamic_camera());
}

fn exercise_explicit_retype(f: DetailsFixture) {
    f.type_axis(Y, "2.5436");
    f.type_axis(X, "12.3456");
    let (precise, before) = f.observe();
    assert_eq!(precise.translation.x.to_bits(), 12.3456_f32.to_bits());
    assert_eq!(precise.translation.y.to_bits(), 2.5436_f32.to_bits());
    assert_eq!(f.axis(X).value_text.as_str(), "12.35");
    f.type_axis(X, "12.35");
    let mut rounded = precise;
    rounded.translation.x = 12.35;
    let (actual, edited) = f.observe();
    assert_eq!(
        actual, rounded,
        "deliberate entry can round only its own axis"
    );
    assert!(edited.can_undo && edited.top != before.top && edited.len == before.len + 1);
    f.harness.dispatch_menu_action(UNDO);
    assert_eq!(f.observe().0, precise);
    assert!(f.observe().1.can_redo);
    f.harness.dispatch_menu_action(REDO);
    assert_eq!(f.observe().0, rounded);
    assert_eq!(f.observe().1.top, edited.top);
    f.harness.dispatch_menu_action("workbench.project.save");
    f.finish();
}

struct DetailsFixture {
    harness: ColdHostHarness,
    location: PathBuf,
    node: NodeId,
    history: HistoryContextId,
    world: GatewaySessionIdentity,
}

impl DetailsFixture {
    fn static_cube() -> Self {
        // RenderableEmpty's authored Cube is a project OBJ, not builtin://cube.
        Self::for_node("Cube", NodeKind::Mesh, Mobility::Static)
    }

    fn dynamic_camera() -> Self {
        // The normal editable control is another authored entity in the same template.
        Self::for_node("Camera", NodeKind::Camera, Mobility::Dynamic)
    }

    fn for_node(name: &str, kind: NodeKind, mobility: Mobility) -> Self {
        let base = fs::canonicalize(
            std::env::var_os("ZIRCON_TEST_OUTPUT_ROOT").expect("an owned fixture root"),
        )
        .unwrap();
        let approved = fs::canonicalize("E:/cargo-targets/zircon-local").unwrap();
        let profile = fs::canonicalize(std::env::var_os("USERPROFILE").unwrap()).unwrap();
        let settings =
            fs::canonicalize(std::env::var_os("ZIRCON_EDITOR_APPEARANCE_PREFERENCES").unwrap())
                .unwrap();
        assert!(
            base != approved
                && base.starts_with(approved)
                && base.join(".zr-editor-owner-test").is_file()
        );
        assert!(profile.starts_with(&base) && settings.starts_with(&base));
        assert_eq!(fs::canonicalize(std::env::temp_dir()).unwrap(), base);
        let location = unique_temp_path("ze_details_enter").with_extension("");
        assert_eq!(fs::canonicalize(location.parent().unwrap()).unwrap(), base);
        fs::create_dir(&location).expect("fresh, owned fixture directory");
        let harness = ColdHostHarness::new(&location);
        let (node, history, world) = {
            let mut host = harness.host.borrow_mut();
            let manager = host
                .editor_manager
                .enter()
                .expect("setup: real EditorManager");
            let (root, _, _) = manager
                .active_project_session_focus_target()
                .expect("setup: cold project session reached Ready");
            let scene = manager
                .active_scene_identity_for_session()
                .expect("setup: scene document lifecycle bind");
            drop(manager);
            assert_eq!(
                fs::canonicalize(&root).unwrap(),
                fs::canonicalize(location.join("DetailsEnter")).unwrap()
            );
            assert_eq!(scene.project_root(), root.as_path());
            assert_eq!(scene.scene_uri(), "res://scenes/main.scene.toml");
            assert!(
                scene.source_revision().is_some(),
                "setup: physical scene source identity"
            );
            let history = HistoryContextId::Document(scene.document());
            assert_eq!(host.runtime.active_scene_history_context(), Some(history));
            let node = {
                let shell = host.runtime.shell().lock();
                shell.state.world.expect_with_world(|scene| {
                    let nodes = scene.nodes();
                    let mut matches = nodes.iter().filter(|node| node.name == name);
                    let node = matches.next().expect("setup: authored template entity");
                    assert!(matches.next().is_none(), "setup: unique authored {name}");
                    assert_eq!(node.kind, kind);
                    assert_eq!(scene.mobility(node.id), Some(mobility));
                    node.id
                })
            };
            let effects = callback_dispatch::dispatch_hierarchy_selection(&host.runtime, node)
                .expect("select the authored template entity in the authoring world");
            host.apply_dispatch_effects(effects);
            host.sync_shell_size();
            host.refresh_ui();
            if let Some(error) = harness.root_ui.take_fatal_failure() {
                panic!("setup: retained project projection failed: {error}");
            }
            let unreal = crate::ui::template_runtime::builtin::UNREAL_EDITOR_WINDOW_DOCUMENT_ID;
            assert!(host
                .active_activity_window_template_document_is(unreal)
                .unwrap());
            let world = host.runtime.context().authoring_gateway().identity();
            (node, history, world)
        };
        Self {
            harness,
            location,
            node,
            history,
            world,
        }
    }

    fn observe(&self) -> (Transform, HistoryStatus) {
        let host = self.harness.host.borrow();
        let shell = host.runtime.shell().lock();
        let state = &shell.state;
        assert_eq!(state.active_scene_history_context(), Some(self.history));
        assert_eq!(state.context.authoring_gateway().identity(), self.world);
        let selection = state.viewport_controller.selection();
        assert_eq!(selection.active_domain(), WorldDomain::Edit);
        assert_eq!(selection.active_primary(), Some(self.node));
        assert_eq!(selection.active_items().len(), 1);
        let transform = state
            .world
            .expect_with_world(|scene| scene.find_node(self.node).unwrap().transform);
        let history = state.transactions().history_status(self.history).unwrap();
        (transform, history)
    }

    fn assert_static_axis_rejected(&self, control: &str, value: &str) {
        let (field, binding_path) = match control {
            X => (
                "transform.translation.x",
                "Inspector/TransformPositionXCommit:onSubmit",
            ),
            Y => (
                "transform.translation.y",
                "Inspector/TransformPositionYCommit:onSubmit",
            ),
            _ => unreachable!("unqualified axis: {control}"),
        };
        let before = self.observe();
        let before_journal = self.harness.host.borrow().runtime.journal();
        let since = before_journal
            .records()
            .last()
            .map_or(0, |record| record.sequence.0);
        self.type_axis(control, value);
        assert_eq!(
            self.observe(),
            before,
            "Static transform and history must stay unchanged"
        );

        let journal = self.harness.host.borrow().runtime.journal();
        let mut submits = journal.records().iter().filter(|record| {
            record.sequence.0 > since && matches!(&record.event, EditorEvent::Inspector(_))
        });
        let record = submits
            .next()
            .expect("real Inspector Submit reached the Static veto");
        assert!(submits.next().is_none());
        assert_eq!(record.source, EditorEventSource::RetainedHost);
        assert_eq!(record.binding_path.as_deref(), Some(binding_path));
        let EditorEvent::Inspector(event) = &record.event else {
            unreachable!("filtered Inspector record");
        };
        assert_eq!(event.subject_path, "entity://selected");
        let [change] = event.changes.as_slice() else {
            panic!("expected one submitted axis: {:?}", event.changes);
        };
        assert_eq!(change.field_id, field);
        let typed = value.parse::<f32>().unwrap();
        assert_eq!(change.value, UiBindingValue::Float(f64::from(typed)));
        assert!(record.transaction_id.is_none());
        assert!(
            record
                .result
                .error
                .as_deref()
                .is_some_and(|error| error.contains("reflection edit failed")),
            "expected the reflected Scene veto: {record:?}"
        );
    }

    fn read_saved_source(&self) -> (f32, Vec<u8>) {
        let host = self.harness.host.borrow();
        let projects = host.asset_runtime_access.project_asset_manager().unwrap();
        let snapshot = projects.current_project_generation_snapshot().unwrap();
        let (project, _) = snapshot.into_parts();
        let uri = &project.manifest().default_scene;
        let bytes = fs::read(project.source_path_for_uri(uri).unwrap()).unwrap();
        // Decode independently for observation; do not install or reopen another authoring world.
        let loaded = Scene::load_scene_document(&project, uri).unwrap();
        let (scene, _) = loaded.into_parts();
        let x = scene.find_node(self.node).unwrap().transform.translation.x;
        (x, bytes)
    }

    fn axis(&self, control: &str) -> TemplatePaneNodeData {
        let presentation = self.harness.root_ui.get_host_presentation();
        let nodes = presentation.workbench_window_nodes;
        let mut nodes = nodes
            .iter()
            .filter(|node| node.control_id.as_str() == control);
        let node = nodes.next().expect("actual projected Details axis").clone();
        assert!(nodes.next().is_none() && !node.disabled);
        node
    }

    fn focus(&self, control: &str) {
        let node = self.axis(control);
        let frame = node.frame;
        assert!(frame.width > 0.0 && frame.height > 0.0);
        let (x, y) = (frame.x + frame.width * 0.5, frame.y + frame.height * 0.5);
        let ui = &self.harness.root_ui;
        ui.dispatch_native_primary_press_for_test(x, y);
        ui.dispatch_native_primary_release_for_test(x, y);
        let focus = host_context(ui).get_text_input_focus();
        assert_eq!(focus.control_id.as_str(), control);
        assert!(!focus.commit_action_id.is_empty());
    }

    fn key(&self, logical: Key, code: KeyCode, text: Option<&str>, modifiers: ModifiersState) {
        let event = key_event(
            logical,
            PhysicalKey::Code(code),
            text,
            ElementState::Pressed,
        );
        self.harness
            .root_ui
            .dispatch_native_key_for_test(event, modifiers);
    }

    fn enter(&self) {
        let since = self.trace_enter("pre-key", None);
        let enter = Key::Named(NamedKey::Enter);
        self.key(enter, KeyCode::Enter, None, ModifiersState::empty());
        self.trace_enter("post-key", Some(since));
        self.harness.host.borrow_mut().refresh_ui();
    }

    // Read-only. Take the post-key sample before the test's explicit refresh.
    fn trace_enter(&self, phase: &str, since: Option<u64>) -> u64 {
        let focus = host_context(&self.harness.root_ui).get_text_input_focus();
        let host = self.harness.host.borrow();
        let map = |id: &str| host.workbench_window_bridge.binding_id_for_action_id(id);
        let unreal = crate::ui::template_runtime::builtin::UNREAL_EDITOR_WINDOW_DOCUMENT_ID;
        let active_template = host.active_activity_window_template_document_is(unreal);
        let document = host
            .editor_manager
            .enter()
            .ok()
            .and_then(|manager| manager.active_scene_identity_for_session());
        let document = document.as_ref().map(|scene| {
            (
                scene.document(),
                scene.scene_uri(),
                scene.source_revision().is_some(),
            )
        });
        let status = host.runtime.status_line();
        let journal = host.runtime.journal();
        let latest = journal
            .records()
            .last()
            .map_or(0, |record| record.sequence.0);
        eprintln!(
            "R21_DETAILS_ENTER_FOCUS {phase} control={:?} kind={:?} value={:?} action={:?}->{:?} edit={:?}->{:?} commit={:?}->{:?} unreal={active_template:?}",
            focus.control_id,
            focus.dispatch_kind,
            focus.value_text,
            focus.action_id,
            map(focus.action_id.as_str()),
            focus.edit_action_id,
            map(focus.edit_action_id.as_str()),
            focus.commit_action_id,
            map(focus.commit_action_id.as_str()),
        );
        let shell = host.runtime.shell().lock();
        let state = &shell.state;
        let selection = state.viewport_controller.selection();
        let selected = selection.active_primary();
        let selected_node = state.world.with_world(|scene| {
            selected.and_then(|id| {
                scene.find_node(id).map(|node| {
                    let translation = node.transform.translation;
                    (
                        node.name,
                        node.kind,
                        scene.mobility(id),
                        translation.to_array().map(f32::to_bits),
                    )
                })
            })
        });
        eprintln!(
            "R21_DETAILS_ENTER_STATE {phase} doc={document:?} active_history={:?} pinned_history={:?} gateway={:?} pinned_gateway={:?} domain={:?} selected={selected:?} pinned_node={:?} node_name_kind_mobility_xyz_bits={selected_node:?} history={:?} status={status:?}",
            state.active_scene_history_context(),
            self.history,
            state.context.authoring_gateway().identity(),
            self.world,
            selection.active_domain(),
            self.node,
            state.transactions().history_status(self.history),
        );
        if let Some(since) = since {
            eprintln!("R21_DETAILS_ENTER_JOURNAL after_seq={since} newest={latest}");
            for record in journal
                .records()
                .iter()
                .filter(|record| record.sequence.0 > since)
            {
                eprintln!("R21_DETAILS_ENTER_RECORD {record:?}");
            }
        }
        latest
    }

    fn type_axis(&self, control: &str, value: &str) {
        let before = self.observe();
        self.focus(control);
        let select_all = Key::Character("a".into());
        self.key(select_all, KeyCode::KeyA, None, ModifiersState::CONTROL);
        for ch in value.chars() {
            let code = match ch {
                '1' => KeyCode::Digit1,
                '2' => KeyCode::Digit2,
                '3' => KeyCode::Digit3,
                '4' => KeyCode::Digit4,
                '5' => KeyCode::Digit5,
                '6' => KeyCode::Digit6,
                '.' => KeyCode::Period,
                _ => unreachable!("unexpected test keystroke: {ch}"),
            };
            let mut buf = [0; 4];
            let text: &str = ch.encode_utf8(&mut buf);
            let logical = Key::Character(text.into());
            self.key(logical, code, Some(text), ModifiersState::empty());
        }
        let focus = host_context(&self.harness.root_ui).get_text_input_focus();
        assert_eq!(focus.value_text.as_str(), value);
        assert_eq!(self.observe(), before, "input before Enter is preview only");
        self.enter();
    }

    fn finish(self) {
        let host = &self.harness.host;
        host.borrow_mut().commit_project_close().unwrap();
        drop(self.harness);
        fs::remove_dir_all(self.location).expect("remove only the owned fixture directory");
    }
}

// Real Project startup; never construct a Welcome/demo controller on this Core.
struct ColdHostHarness {
    host: Rc<RefCell<RetainedEditorHost>>,
    root_ui: UiHostWindow,
    _core: CoreRuntime,
}

impl ColdHostHarness {
    fn new(location: &std::path::Path) -> Self {
        let restore_config =
            crate::tests::editor_event::support::EnvironmentVariableRestoreGuard::capture(
                "ZIRCON_CONFIG_PATH",
            );
        std::env::set_var("ZIRCON_CONFIG_PATH", location.join("config.json"));
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
            .expect("setup: activate the registered Scene module");
        core.activate_module(module::EDITOR_MODULE_NAME).unwrap();

        let provenance = crate::tests::support::test_project_creation_provenance();
        let intent = ProjectLaunchIntent::create_project(
            provenance.operation_id(),
            ProjectLaunchSource::Cli,
            ProjectLaunchProfile::Normal,
            "DetailsEnter",
            location,
            ProjectTemplateId::RenderableEmpty,
        )
        .unwrap();
        let startup = EditorGuiStartupRequest::project(intent)
            .preflight_project(&ProjectAuthority::default())
            .expect("setup: preflight the actual project request");
        let root_ui = UiHostWindow::new().expect("setup: native retained host state");
        root_ui.window().set_size(PhysicalSize::new(1672, 941));
        let mut host = RetainedEditorHost::new(
            core.handle(),
            // App's separate profiling transport is unused by this Edit-world fixture.
            Arc::new(DetachedEditorRuntimeGateway),
            root_ui.clone_strong(),
            Some(startup),
            Some(provenance.build_set_id().clone()),
            None,
        )
        .expect("setup: initialize the retained host directly in the admitted project");
        drop(restore_config);
        assert_eq!(host.startup_session.mode, EditorSessionMode::Project);
        let settings = host.runtime.context().settings().snapshot();
        apply_host_appearance_from_tokens(
            settings.design_tokens_handle(),
            Some(settings.generation()),
        );
        apply_host_paint_scale_factor(
            ResolutionContext::from_physical_size_with_scale_mode(
                host.shell_size,
                host.shell_scale_factor,
                host.shell_scale_mode,
            )
            .effective_scale_factor(),
        );
        root_ui.sync_host_paint_theme();
        install_editor_v2_design_tokens(settings.as_ref());
        host.sync_plugin_template_documents_if_changed()
            .expect("setup: actual template owners");
        let host = Rc::new(RefCell::new(host));
        wire_callbacks(&root_ui, &host, None);
        host.borrow_mut().self_handle = Some(Rc::downgrade(&host));
        root_ui
            .show()
            .expect("setup: enable retained hit testing without a desktop event loop");
        Self {
            host,
            root_ui,
            _core: core,
        }
    }

    fn dispatch_menu_action(&self, action: &str) {
        let mut host = self.host.borrow_mut();
        let effects = callback_dispatch::dispatch_menu_action(&host.runtime, action)
            .expect("menu action dispatch should succeed");
        host.apply_dispatch_effects(effects);
        host.recompute_if_dirty();
    }
}
