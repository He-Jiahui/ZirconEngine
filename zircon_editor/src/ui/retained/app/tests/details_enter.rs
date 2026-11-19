#![cfg(windows)]

use super::super::support::*;
use crate::core::editing::engine::{HistoryContextId, HistoryStatus};
use crate::core::gateway::GatewaySessionIdentity;
use crate::core::play::WorldDomain;
use crate::core::project::{NewProjectDraft, ProjectTemplateId};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime::scene::{components::NodeKind, NodeId, Scene};
use zircon_runtime_interface::math::Transform;

const X: &str = "WorkbenchTransformPositionX";
const Y: &str = "WorkbenchTransformPositionY";
const UNDO: &str = "workbench.history.undo";
const REDO: &str = "workbench.history.redo";

#[test]
#[ignore = "requires an isolated physical Editor fixture and user profile"]
fn details_enter_untouched_preserves_saved_precision_and_redo() {
    let _guard = lock_env();
    let f = DetailsFixture::new();
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
    f.enter(); // No caret change or text input since pointer focus.
    let after_enter = f.observe();
    let displayed_after = f.axis(X).value_text;
    let (_, disk_after) = f.read_saved_source();
    f.harness.dispatch_menu_action(REDO);
    let (after_redo, history_after_redo) = f.observe();
    assert_eq!(
        (after_enter, after_redo, history_after_redo.top),
        ((exact, before), moved, edited.top),
        "untouched native Enter must preserve the exact X, clean checkpoint and Y redo branch"
    );
    assert_eq!(displayed_after.as_str(), "12.35");
    assert_eq!(disk_after, saved_bytes);
    f.harness.dispatch_menu_action(UNDO);
    f.finish();
}

#[test]
#[ignore = "requires an isolated physical Editor fixture and user profile"]
fn details_enter_explicitly_retyping_displayed_value_is_a_real_edit() {
    let _guard = lock_env();
    let f = DetailsFixture::new();
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
    harness: ChildWindowHostHarness,
    location: PathBuf,
    cube: NodeId,
    history: HistoryContextId,
    world: GatewaySessionIdentity,
}

impl DetailsFixture {
    fn new() -> Self {
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
        let harness = ChildWindowHostHarness::new_at_config_path(location.join("config.json"));
        harness
            ._core
            .activate_module(zircon_runtime::core::framework::scene::SCENE_MODULE_NAME)
            .expect("activate the linked Scene module, not a substitute service");
        crate::tests::support::configure_editor_test_runtime_build_set(&harness._core);
        let window = harness.root_ui.window();
        window.set_size(PhysicalSize::new(1672, 941));
        let (cube, history, world) = {
            let mut host = harness.host.borrow_mut();
            let draft = NewProjectDraft {
                project_name: "DetailsEnter".into(),
                location: location.to_string_lossy().into_owned(),
                template: ProjectTemplateId::RenderableEmpty,
            };
            let manager = host.editor_manager.enter().expect("real EditorManager");
            let session = manager
                .create_project_and_open(draft)
                .expect("real project admission");
            drop(manager);
            host.apply_startup_session(session)
                .expect("bind the startup scene document, not a detached replacement");
            let cube = {
                let shell = host.runtime.shell().lock();
                shell.state.world.expect_with_world(|scene| {
                    let nodes = scene.nodes();
                    nodes
                        .iter()
                        .find(|node| node.kind == NodeKind::Cube)
                        .unwrap()
                        .id
                })
            };
            let effects = callback_dispatch::dispatch_hierarchy_selection(&host.runtime, cube)
                .expect("select the real Cube in the authoring world");
            host.apply_dispatch_effects(effects);
            host.sync_shell_size();
            host.refresh_ui();
            let history = host.runtime.active_scene_history_context().unwrap();
            assert!(matches!(history, HistoryContextId::Document(_)));
            let world = host.runtime.context().authoring_gateway().identity();
            (cube, history, world)
        };
        Self {
            harness,
            location,
            cube,
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
        assert_eq!(selection.active_primary(), Some(self.cube));
        assert_eq!(selection.active_items().len(), 1);
        let transform = state
            .world
            .expect_with_world(|scene| scene.find_node(self.cube).unwrap().transform);
        let history = state.transactions().history_status(self.history).unwrap();
        (transform, history)
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
        let x = scene.find_node(self.cube).unwrap().transform.translation.x;
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
        let enter = Key::Named(NamedKey::Enter);
        self.key(enter, KeyCode::Enter, None, ModifiersState::empty());
        self.harness.host.borrow_mut().refresh_ui();
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
