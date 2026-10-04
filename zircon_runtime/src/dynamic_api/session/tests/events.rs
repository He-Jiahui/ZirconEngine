use super::super::profile::RuntimeDynamicSessionProfile;
use super::gamepad::{ui_gamepad_analog_control, ui_gamepad_navigation};
use super::{
    clock_discontinuity_for_lifecycle_state, clock_discontinuity_for_window_status,
    RuntimeDynamicSession,
};
use crate::core::framework::input::WindowStatusEvent;
use crate::core::{
    ClockDiscontinuity, ClockLifecycleTransition, FrameClockRebaseCause, FrameTimeDiscontinuity,
};
use zircon_runtime_interface::ui::surface::UiNavigationEventKind;
use zircon_runtime_interface::{
    ZrByteSlice, ZrRuntimeEventV1, ZrRuntimeViewportCameraV1, ZrRuntimeViewportHandle,
    ZrStatusCode, ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_X_V1,
    ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_Y_V1, ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_DOWN_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_LEFT_V1, ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_RIGHT_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_UP_V1, ZR_RUNTIME_GAMEPAD_BUTTON_EAST_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_SOUTH_V1, ZR_RUNTIME_LIFECYCLE_STATE_LOW_MEMORY_V1,
    ZR_RUNTIME_LIFECYCLE_STATE_SUSPENDED_V1, ZR_RUNTIME_VIEWPORT_CAMERA_PROJECTION_ORTHOGRAPHIC_V1,
};

#[test]
fn gamepad_buttons_map_to_shared_ui_navigation_semantics() {
    assert_eq!(
        ui_gamepad_navigation(ZR_RUNTIME_GAMEPAD_BUTTON_SOUTH_V1),
        Some(UiNavigationEventKind::Activate)
    );
    assert_eq!(
        ui_gamepad_navigation(ZR_RUNTIME_GAMEPAD_BUTTON_EAST_V1),
        Some(UiNavigationEventKind::Cancel)
    );
    assert_eq!(
        ui_gamepad_navigation(ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_UP_V1),
        Some(UiNavigationEventKind::Up)
    );
    assert_eq!(
        ui_gamepad_navigation(ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_DOWN_V1),
        Some(UiNavigationEventKind::Down)
    );
    assert_eq!(
        ui_gamepad_navigation(ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_LEFT_V1),
        Some(UiNavigationEventKind::Left)
    );
    assert_eq!(
        ui_gamepad_navigation(ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_RIGHT_V1),
        Some(UiNavigationEventKind::Right)
    );
    assert_eq!(ui_gamepad_navigation(u32::MAX), None);
}

#[test]
fn gamepad_left_stick_axes_use_the_shared_ui_analog_navigation_controls() {
    assert_eq!(
        ui_gamepad_analog_control(ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_X_V1),
        Some("gamepad_left_stick_x")
    );
    assert_eq!(
        ui_gamepad_analog_control(ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_Y_V1),
        Some("gamepad_left_stick_y")
    );
    assert_eq!(ui_gamepad_analog_control(u32::MAX), None);
}

#[test]
fn lifecycle_clock_mapping_keeps_low_memory_out_of_the_time_authority() {
    assert_eq!(
        clock_discontinuity_for_lifecycle_state(ZR_RUNTIME_LIFECYCLE_STATE_SUSPENDED_V1),
        Some(ClockDiscontinuity::ApplicationLifecycle(
            ClockLifecycleTransition::Suspended,
        ))
    );
    assert_eq!(
        clock_discontinuity_for_lifecycle_state(ZR_RUNTIME_LIFECYCLE_STATE_LOW_MEMORY_V1),
        None
    );
}

#[test]
fn window_clock_mapping_marks_occlusion_and_surface_recreation() {
    assert_eq!(
        clock_discontinuity_for_window_status(&WindowStatusEvent::Occluded(true)),
        Some(ClockDiscontinuity::WindowOcclusionChanged { occluded: true })
    );
    assert_eq!(
        clock_discontinuity_for_window_status(&WindowStatusEvent::SurfaceRecreated),
        Some(ClockDiscontinuity::WindowSurfaceRecreated)
    );
}

#[test]
fn dynamic_lifecycle_event_replaces_activation_rebase_with_a_typed_clock_cause() {
    let mut session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
        .expect("headless runtime session should construct");

    let status = session.handle_event(ZrRuntimeEventV1::lifecycle(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(1),
        ZR_RUNTIME_LIFECYCLE_STATE_SUSPENDED_V1,
    ));
    let snapshot = session
        .runtime
        .tick_time(session.time_policy.max_fixed_steps_per_frame());

    assert_eq!(status.status_code(), ZrStatusCode::Ok);
    assert!(matches!(
        snapshot.discontinuity(),
        Some(FrameTimeDiscontinuity::FrameClockRebased(receipt))
            if receipt.cause()
                == FrameClockRebaseCause::ClockDiscontinuity(
                    ClockDiscontinuity::ApplicationLifecycle(
                        ClockLifecycleTransition::Suspended,
                    ),
                )
    ));
}

#[test]
fn simulate_camera_event_overrides_render_extract_without_mutating_the_play_world() {
    let mut session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
        .expect("headless runtime session should construct");
    let active_camera = session.level.with_world(|world| world.active_camera());
    let world_transform_before = session
        .level
        .with_world(|world| world.world_transform(active_camera).unwrap());
    let camera = ZrRuntimeViewportCameraV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        crate::core::math::Transform::from_translation(crate::core::math::Vec3::new(7.0, 8.0, 9.0)),
        ZR_RUNTIME_VIEWPORT_CAMERA_PROJECTION_ORTHOGRAPHIC_V1,
        60.0_f32.to_radians(),
        18.0,
        0.5,
        750.0,
    );
    let payload = serde_json::to_vec(&camera).expect("camera DTO should encode");

    let status = session.handle_event(ZrRuntimeEventV1::viewport_camera(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(1),
        ZrByteSlice {
            data: payload.as_ptr(),
            len: payload.len(),
        },
    ));
    let extract = session.current_extract();
    let world_transform_after = session
        .level
        .with_world(|world| world.world_transform(active_camera).unwrap());

    assert_eq!(status.status_code(), ZrStatusCode::Ok);
    assert_eq!(extract.view.camera.transform, camera.transform);
    assert_eq!(extract.view.camera.ortho_size, 18.0);
    assert_eq!(world_transform_after, world_transform_before);
}
