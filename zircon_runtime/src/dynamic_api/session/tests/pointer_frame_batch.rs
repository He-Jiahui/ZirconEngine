use zircon_runtime_interface::{
    ZrRuntimeEventV1, ZrRuntimeViewportHandle, ZrStatusCode, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZR_RUNTIME_BUTTON_STATE_PRESSED_V1, ZR_RUNTIME_BUTTON_STATE_RELEASED_V1,
    ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1, ZR_RUNTIME_TOUCH_PHASE_CANCELLED_V1,
    ZR_RUNTIME_TOUCH_PHASE_ENDED_V1, ZR_RUNTIME_TOUCH_PHASE_MOVED_V1,
    ZR_RUNTIME_TOUCH_PHASE_STARTED_V1,
};

use crate::core::framework::input::{InputButton, InputEvent, TouchPhase};
use crate::core::math::Transform;
use crate::dynamic_api::session::{RuntimeDynamicSession, RuntimeDynamicSessionProfile};

const VIEWPORT: ZrRuntimeViewportHandle = ZrRuntimeViewportHandle::new(1);

#[test]
fn pointer_sample_bursts_wait_for_one_frame_camera_apply() {
    for sample_count in [125, 500, 1_000] {
        let mut session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
            .expect("headless runtime session");
        let mut idle = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
            .expect("idle headless runtime session");
        let input = session
            .resolve_input_manager()
            .expect("runtime input manager");
        let (camera_before, generation_before) = camera_state(&session);
        let idle_generation_before = camera_state(&idle).1;

        assert_ok(session.handle_event(mouse_button(
            ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
            100.0,
            100.0,
        )));
        for sample in 1..=sample_count {
            let x = 100.0 + sample as f32 * 0.1;
            let y = 100.0 + sample as f32 * 0.01;
            assert_ok(session.handle_event(ZrRuntimeEventV1::pointer_moved(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                VIEWPORT,
                x,
                y,
            )));
        }
        let latest = [
            100.0 + sample_count as f32 * 0.1,
            100.0 + sample_count as f32 * 0.01,
        ];
        assert_eq!(input.snapshot().cursor_position, latest);
        assert_eq!(
            session.input_diagnostics.snapshot().pointer_move_count,
            sample_count as u64,
            "every physical sample must still reach Runtime12"
        );

        assert_ok(session.handle_event(mouse_button(
            ZR_RUNTIME_BUTTON_STATE_RELEASED_V1,
            latest[0],
            latest[1],
        )));
        assert!(!input.button_pressed(&InputButton::MouseRight));
        let button_edges: Vec<_> = input
            .drain_events()
            .into_iter()
            .filter(|event| {
                matches!(
                    event,
                    InputEvent::ButtonPressed(InputButton::MouseRight)
                        | InputEvent::ButtonReleased(InputButton::MouseRight)
                )
            })
            .collect();
        assert_eq!(
            button_edges,
            [
                InputEvent::ButtonPressed(InputButton::MouseRight),
                InputEvent::ButtonReleased(InputButton::MouseRight),
            ]
        );
        assert_eq!(
            camera_state(&session),
            (camera_before, generation_before),
            "pointer ingress must leave the World unchanged until the frame boundary"
        );

        let _ = session.current_extract();
        assert_eq!(camera_state(&session), (camera_before, generation_before));
        session
            .tick_frame()
            .expect("apply pointer input at frame tick");
        idle.tick_frame().expect("apply idle comparison frame tick");
        let (camera_after, generation_after) = camera_state(&session);
        let idle_generation_delta = camera_state(&idle).1 - idle_generation_before;
        assert_ne!(camera_after, camera_before);
        assert_eq!(
            generation_after - generation_before,
            idle_generation_delta + 1,
            "one camera transform revision must be added to the idle frame"
        );
        let _ = session.current_extract();
        assert_eq!(camera_state(&session), (camera_after, generation_after));
    }
}

#[test]
fn tick_applies_pending_drag_without_a_capture_request() {
    let mut session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
        .expect("headless runtime session");
    let before = camera_state(&session).0;
    assert_ok(session.handle_event(mouse_button(
        ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
        100.0,
        100.0,
    )));
    assert_ok(session.handle_event(ZrRuntimeEventV1::pointer_moved(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        VIEWPORT,
        140.0,
        110.0,
    )));
    assert_ok(session.handle_event(mouse_button(
        ZR_RUNTIME_BUTTON_STATE_RELEASED_V1,
        140.0,
        110.0,
    )));
    assert_eq!(camera_state(&session).0, before);

    session.tick_frame().expect("apply runtime frame");
    assert_ne!(camera_state(&session).0, before);
}

#[test]
fn wheel_events_preserve_two_zoom_steps_with_one_world_commit() {
    let mut two_steps = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
        .expect("headless runtime session");
    let mut one_step = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
        .expect("headless runtime session");
    let mut idle = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
        .expect("idle headless runtime session");
    let (before, generation_before) = camera_state(&two_steps);
    let idle_generation_before = camera_state(&idle).1;
    assert_eq!(camera_state(&one_step).0, before);

    for _ in 0..2 {
        assert_ok(two_steps.handle_event(ZrRuntimeEventV1::mouse_wheel(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            VIEWPORT,
            1.0,
        )));
    }
    assert_ok(one_step.handle_event(ZrRuntimeEventV1::mouse_wheel(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        VIEWPORT,
        2.0,
    )));
    assert_eq!(camera_state(&two_steps), (before, generation_before));
    assert_eq!(camera_state(&one_step), (before, generation_before));

    two_steps.tick_frame().expect("apply two wheel steps");
    one_step.tick_frame().expect("apply one wheel event");
    idle.tick_frame().expect("apply idle comparison frame tick");
    let idle_generation_delta = camera_state(&idle).1 - idle_generation_before;
    assert_eq!(
        camera_state(&two_steps).1 - generation_before,
        idle_generation_delta + 1
    );
    assert_eq!(
        camera_state(&one_step).1 - generation_before,
        idle_generation_delta + 1
    );
    assert_ne!(camera_state(&two_steps).0, camera_state(&one_step).0);
}

#[test]
fn touch_edges_reach_input_manager_in_order_before_the_frame() {
    let mut session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
        .expect("headless runtime session");
    let input = session
        .resolve_input_manager()
        .expect("runtime input manager");
    for (id, phase, x, y) in [
        (17, ZR_RUNTIME_TOUCH_PHASE_STARTED_V1, 10.0, 10.0),
        (17, ZR_RUNTIME_TOUCH_PHASE_MOVED_V1, 20.0, 20.0),
        (17, ZR_RUNTIME_TOUCH_PHASE_ENDED_V1, 30.0, 30.0),
        (18, ZR_RUNTIME_TOUCH_PHASE_STARTED_V1, 40.0, 40.0),
        (18, ZR_RUNTIME_TOUCH_PHASE_CANCELLED_V1, 50.0, 50.0),
    ] {
        assert_ok(session.handle_event(ZrRuntimeEventV1::touch(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            VIEWPORT,
            id,
            phase,
            x,
            y,
        )));
    }
    assert_eq!(input.snapshot().cursor_position, [50.0, 50.0]);
    let edges: Vec<_> = input
        .drain_events()
        .into_iter()
        .filter_map(|event| match event {
            InputEvent::Touch { id, phase, .. } => Some((id, phase)),
            _ => None,
        })
        .collect();
    assert_eq!(
        edges,
        [
            (17, TouchPhase::Started),
            (17, TouchPhase::Moved),
            (17, TouchPhase::Ended),
            (18, TouchPhase::Started),
            (18, TouchPhase::Cancelled),
        ]
    );
}

fn mouse_button(state: u32, x: f32, y: f32) -> ZrRuntimeEventV1 {
    ZrRuntimeEventV1::mouse_button(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        VIEWPORT,
        ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1,
        state,
        x,
        y,
    )
}

fn camera_state(session: &RuntimeDynamicSession) -> (Transform, u64) {
    session.level.with_world(|world| {
        let camera = world.active_camera();
        (
            world
                .local_transform(camera)
                .expect("active camera transform"),
            world.world_generation(),
        )
    })
}

fn assert_ok(status: zircon_runtime_interface::ZrStatus) {
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
}
