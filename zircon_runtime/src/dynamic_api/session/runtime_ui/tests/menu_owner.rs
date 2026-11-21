use serde_json::{json, Value};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiTreeId},
    layout::{UiFrame, UiPoint, UiSize},
    surface::UiRenderCommandKind,
    tree::UiInputPolicy,
};
use zircon_runtime_interface::{
    ZrRuntimeEventV1, ZrRuntimeViewportSizeV1, ZrStatusCode, ZIRCON_RUNTIME_ABI_VERSION_V1 as ABI,
    ZR_RUNTIME_BUTTON_STATE_PRESSED_V1 as DOWN, ZR_RUNTIME_BUTTON_STATE_RELEASED_V1 as UP,
    ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1 as LEFT, ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1 as RIGHT,
};

use super::super::menu::GAMEPLAY_MENU_COMPONENT as MENU;
use super::super::{RuntimeDynamicSession, RuntimeDynamicSessionProfile, DEFAULT_VIEWPORT as VP};
use super::input_publication::RuntimeUiInputQueryAdmission;
use super::{global_node_id, RuntimeUiSurface, RUNTIME_PROJECT_UI_TREE_ID as PROJECT_UI};
use crate::core::framework::input::{InputButton, InputEvent, InputEventRecordingConfig};
use crate::core::math::UVec2;
use crate::scene::{components::NodeKind, EntityId};
use crate::text::text_runtime_context_for_core;
use crate::ui::dispatch::UiInputManager;
use crate::ui::v2::{UiV2PrototypeStoreBuilder, UiV2SurfaceBuilder, UiZuiAssetLoader};

const CONTROL: &str = "gameplay.control_state";
const FALLBACK_TREE: &str = "runtime.gameplay.menu";
const VIEWPORT: UVec2 = UVec2::new(640, 360);
const HUD_TEXT: &str = "Authored HUD";
const HUD_ZUI: &str = r#"
[asset]
kind = "view"
id = "test.runtime.menu_owner_hud"
version = 2
display_name = "Menu owner HUD"
[root]
node = "hud"
[nodes.hud]
component = "Text"
control_id = "MenuOwnerHud"
props = { text = "Authored HUD" }
layout = { input_policy = "Ignore", width = { min = 640.0, preferred = 640.0, max = 640.0, stretch = "Fixed" }, height = { min = 360.0, preferred = 360.0, max = 360.0, stretch = "Fixed" } }
"#;

#[test]
fn fallback_menu_owner_visible_start_and_retry_activate_on_release() {
    paired_primary("start", "Start Game", "start_game", false);
    paired_primary("game_over", "Retry", "retry_game", false);
}

#[test]
fn fallback_menu_owner_authored_hud_keeps_start_inert() {
    paired_primary("start", "Start Game", "none", true);
}

#[test]
fn fallback_menu_owner_authored_hud_keeps_retry_inert() {
    paired_primary("game_over", "Retry", "none", true);
}

fn paired_primary(mode: &str, fallback_label: &str, expected: &str, authored: bool) {
    let (mut session, entity, frame) = fixture(mode);
    let point = frame.center();
    let fallback = view(&mut session, point);
    let hud = authored.then(|| mount_hud(&mut session));
    let rendered = view(&mut session, point);
    let before = stored(&session, entity);
    let down = mouse(&mut session, LEFT, DOWN, point);
    let held = session
        .resolve_input_manager()
        .unwrap()
        .button_pressed(&InputButton::MouseLeft);
    let during = stored(&session, entity);
    let focus_on_press = session.runtime_ui.focused_surface;
    let up = mouse(&mut session, LEFT, UP, point);
    let after = stored(&session, entity);
    let (events, counts) = input(&session, InputButton::MouseLeft);
    let rendered_after = view(&mut session, point);
    println!(
        "menu-owner mode={mode} authored={authored} world={:?} entity={entity} point={point:?} button_quad=14 frame={frame:?} \
         fallback={fallback:?} view={rendered:?}->{rendered_after:?} status={down:?}/{up:?} \
         held={held} focus_on_press={focus_on_press:?} counts={counts:?} journal={events:?} stored={before:?}->{during:?}->{after:?}",
        session.level.world_handle(),
    );

    assert_eq!(session.camera_controller.viewport_size(), VIEWPORT);
    assert_view(
        &fallback,
        FALLBACK_TREE,
        UiNodeId::new(15),
        fallback_label,
        None,
    );
    if let Some(hud) = hud {
        assert_view(&rendered, PROJECT_UI, hud, HUD_TEXT, Some(0));
    } else {
        assert_eq!(rendered, fallback);
    }
    assert_eq!(rendered_after, rendered);
    assert_eq!([down, up], [ZrStatusCode::Ok; 2]);
    assert!(held);
    assert_eq!(focus_on_press, None);
    assert_eq!(counts, (1, 1, 0, false));
    use InputButton::MouseLeft;
    use InputEvent::{ButtonPressed, ButtonReleased};
    assert_eq!(
        events,
        [ButtonPressed(MouseLeft), ButtonReleased(MouseLeft)]
    );
    assert_eq!(before.0, Some(json!({ "state": mode })));
    assert_eq!(before.1, Some(json!("none")));
    assert_eq!(during, before);
    assert_eq!(
        after,
        (
            before.0,
            Some(json!(expected)),
            before.2 + u64::from(!authored)
        ),
        "only the submitted menu may activate its button"
    );
}

#[test]
fn fallback_menu_owner_authored_hud_preserves_camera_and_off_target_input() {
    let (mut session, entity, frame) = fixture("start");
    let hud = mount_hud(&mut session);
    let center = frame.center();
    let moved = UiPoint::new(center.x + frame.width * 0.1, center.y + frame.height * 0.1);
    let off = UiPoint::new(8.0, 8.0);
    let rendered = view(&mut session, center);
    let before = stored(&session, entity);
    let pose = |s: &RuntimeDynamicSession| {
        s.level.with_world(|w| {
            w.world_transform(w.active_camera())
                .expect("fixture: default camera")
        })
    };
    let statuses = [
        mouse(&mut session, LEFT, DOWN, off),
        mouse(&mut session, LEFT, UP, off),
    ];
    let after_left = stored(&session, entity);
    let base = pose(&session);
    let drag = [
        mouse(&mut session, RIGHT, DOWN, center),
        move_to(&mut session, moved),
    ];
    let turned = pose(&session);
    let release = [
        mouse(&mut session, RIGHT, UP, moved),
        move_to(&mut session, off),
    ];
    let remains = pose(&session);
    let after = stored(&session, entity);
    let (events, counts) = input(&session, InputButton::MouseRight);
    println!(
        "menu-owner case=camera world={:?} entity={entity} center={center:?} view={rendered:?} \
         counts={counts:?} journal={events:?} stored={before:?}->{after_left:?}->{after:?} \
         camera_moved={} drag_cleared={}",
        session.level.world_handle(),
        base != turned,
        turned == remains,
    );

    assert_view(&rendered, PROJECT_UI, hud, HUD_TEXT, Some(0));
    assert!(!frame.contains_point(off) && frame.contains_point(moved));
    assert_eq!([statuses, drag, release], [[ZrStatusCode::Ok; 2]; 3]);
    assert_eq!(counts, (2, 2, 2, false));
    use InputButton::{MouseLeft, MouseRight};
    use InputEvent::{ButtonPressed, ButtonReleased, CursorMoved};
    let cursor = |p: UiPoint| CursorMoved { x: p.x, y: p.y };
    assert_eq!(
        events,
        [
            ButtonPressed(MouseLeft),
            ButtonReleased(MouseLeft),
            ButtonPressed(MouseRight),
            cursor(moved),
            ButtonReleased(MouseRight),
            cursor(off)
        ]
    );
    assert_eq!(after_left, before);
    assert_eq!(
        after, before,
        "secondary release over the unseen button is not activation"
    );
    assert_ne!(
        turned, base,
        "unowned pointer must still orbit the real camera"
    );
    assert_eq!(remains, turned, "secondary release must clear drag");
}

fn fixture(mode: &str) -> (RuntimeDynamicSession, EntityId, UiFrame) {
    let mut session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Minimal, None)
        .expect("fixture: real core session and default Level");
    let size = ZrRuntimeViewportSizeV1::new(VIEWPORT.x, VIEWPORT.y);
    let resized = session.handle_event(ZrRuntimeEventV1::viewport_resized(ABI, VP, size));
    if resized.status_code() != ZrStatusCode::Ok {
        panic!("fixture: viewport resize failed {resized:?}");
    }
    let manager = session
        .resolve_input_manager()
        .expect("fixture: physical InputManager");
    manager.set_event_recording_config(InputEventRecordingConfig::enabled(16));
    drop(manager);
    let entity = session.level.with_world_mut(|world| {
        let owner = world
            .spawn_node(NodeKind::Empty)
            .expect("fixture: menu entity");
        world
            .set_dynamic_component(owner, MENU, json!({ "state": mode }))
            .expect("fixture: menu state");
        world
            .set_dynamic_component(owner, CONTROL, json!("none"))
            .expect("fixture: control state");
        owner
    });
    // Learn geometry from the visible control in this exact Level, not `menu_layout`.
    let fallback = session
        .current_ui_submission()
        .unwrap()
        .expect("fixture: fallback menu");
    let frame = fallback
        .commands()
        .find(|c| c.node_id == UiNodeId::new(14) && c.kind == UiRenderCommandKind::Quad)
        .expect("fixture: submitted fallback button quad")
        .frame;
    (session, entity, frame)
}

fn mount_hud(session: &mut RuntimeDynamicSession) -> UiNodeId {
    // Substitute only project/asset ingress; the source parser, compiler and text
    // service are real, and current_ui_submission will publish the actual owned UI.
    let uri = "res://ui/menu-owner.zui";
    let document = UiZuiAssetLoader::load_zui_str(HUD_ZUI).expect("fixture: authored .zui HUD");
    let mut builder = UiV2PrototypeStoreBuilder::new();
    let document = builder.insert_with_aliases_and_source_path(document, [uri], uri);
    let store = builder
        .build_for_roots([uri])
        .expect("fixture: prototype store");
    let text = text_runtime_context_for_core(&session.runtime.handle())
        .expect("fixture: session text context");
    let mut surface = UiV2SurfaceBuilder::build_surface_with_prototype_store_and_text_context(
        UiTreeId::new(format!("{PROJECT_UI}:0")),
        &document,
        &store,
        &text,
    )
    .expect("fixture: compile authored HUD");
    surface
        .compute_layout(UiSize::new(1280.0, 720.0))
        .expect("fixture: HUD initial layout");
    let root = surface.tree.roots[0];
    surface
        .tree
        .node(root)
        .filter(|node| node.input_policy == UiInputPolicy::Ignore)
        .expect("fixture: compiler preserved the authored Ignore policy");
    session.runtime_ui.surfaces.push(RuntimeUiSurface {
        surface,
        input: UiInputManager::summary(),
    });
    global_node_id(0, root)
}

#[derive(Debug, PartialEq)]
struct View {
    trees: Vec<String>,
    commands: usize,
    text: Vec<(UiNodeId, String)>,
    candidates: Option<usize>,
    focused: Option<usize>,
    captured: bool,
}

fn view(session: &mut RuntimeDynamicSession, point: UiPoint) -> View {
    let submission = session
        .current_ui_submission()
        .unwrap()
        .expect("fixture: UI submission");
    let ui = &session.runtime_ui;
    let candidates =
        match ui
            .input_publication
            .query(session.camera_controller.viewport_size(), point, point)
        {
            RuntimeUiInputQueryAdmission::Unpublished => None,
            RuntimeUiInputQueryAdmission::Published(q) => Some(q.candidate_count()),
            other => panic!("fixture: expected valid pointer query, got {other:?}"),
        };
    let text = submission
        .commands()
        .filter_map(|c| {
            let visible = c.kind == UiRenderCommandKind::Text
                && c.frame.contains_point(point)
                && c.opacity > 0.0
                && c.text_layout.as_ref()?.measured_width > 0.0;
            visible.then(|| (c.node_id, c.text.clone().unwrap_or_default()))
        })
        .collect();
    View {
        trees: submission
            .segments()
            .iter()
            .map(|s| s.route_tree_id().to_string())
            .collect(),
        commands: submission.command_count(),
        text,
        candidates,
        focused: ui.focused_surface,
        captured: !ui.pointer_capture_surfaces.is_empty(),
    }
}

fn assert_view(view: &View, tree: &str, id: UiNodeId, label: &str, candidates: Option<usize>) {
    assert_eq!(view.trees, [tree.to_owned()]);
    assert!(view.commands > 0);
    assert_eq!(view.text, [(id, label.to_owned())]);
    assert_eq!(
        (view.candidates, view.focused, view.captured),
        (candidates, None, false)
    );
}

fn stored(
    session: &RuntimeDynamicSession,
    entity: EntityId,
) -> (Option<Value>, Option<Value>, u64) {
    session.level.with_world(|world| {
        (
            world.dynamic_component(entity, MENU).cloned(),
            world.dynamic_component(entity, CONTROL).cloned(),
            world.dynamic_component_generation(CONTROL),
        )
    })
}

fn input(
    session: &RuntimeDynamicSession,
    button: InputButton,
) -> (Vec<InputEvent>, (u64, u64, u64, bool)) {
    let d = session.input_diagnostics.snapshot();
    let manager = session.resolve_input_manager().unwrap();
    let events = manager
        .drain_event_records()
        .into_iter()
        .map(|r| r.event)
        .collect();
    (
        events,
        (
            d.mouse_button_press_count,
            d.mouse_button_release_count,
            d.pointer_move_count,
            manager.button_pressed(&button),
        ),
    )
}

fn mouse(s: &mut RuntimeDynamicSession, button: u32, state: u32, p: UiPoint) -> ZrStatusCode {
    s.handle_event(ZrRuntimeEventV1::mouse_button(
        ABI, VP, button, state, p.x, p.y,
    ))
    .status_code()
}

fn move_to(s: &mut RuntimeDynamicSession, p: UiPoint) -> ZrStatusCode {
    s.handle_event(ZrRuntimeEventV1::pointer_moved(ABI, VP, p.x, p.y))
        .status_code()
}
