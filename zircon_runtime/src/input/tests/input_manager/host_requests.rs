use super::*;

#[test]
fn gamepad_rumble_requests_are_frame_local_and_drainable() {
    let input = DefaultInputManager::default();
    let gamepad = GamepadId(11);
    let add = GamepadRumbleRequest::add(gamepad, GamepadRumbleIntensity::new(1.2, 0.4), 125);
    let stop = GamepadRumbleRequest::stop(gamepad);

    assert_eq!(
        GamepadRumbleIntensity::new(f32::NAN, 1.5).clamped(),
        GamepadRumbleIntensity::new(0.0, 1.0)
    );

    input.submit_event(InputEvent::GamepadRumbleRequest(add));
    input.submit_event(InputEvent::GamepadRumbleRequest(stop));

    let frame = input.frame_snapshot();
    assert_eq!(frame.gamepad_rumble_requests, vec![add, stop]);

    assert_eq!(input.drain_gamepad_rumble_requests(), vec![add, stop]);
    assert!(input.frame_snapshot().gamepad_rumble_requests.is_empty());

    input.submit_event(InputEvent::GamepadRumbleRequest(add));
    input.begin_frame();

    assert!(input.frame_snapshot().gamepad_rumble_requests.is_empty());
    assert_eq!(input.drain_gamepad_rumble_requests(), vec![add]);
    assert!(input.drain_gamepad_rumble_requests().is_empty());
}

#[test]
fn cursor_host_requests_are_frame_local_and_drainable() {
    let input = DefaultInputManager::default();
    let requests = vec![
        CursorHostRequest::set_visible(false),
        CursorHostRequest::set_grab_mode(CursorGrabMode::Locked),
        CursorHostRequest::set_hit_test(false),
        CursorHostRequest::set_position(320.0, 180.0),
    ];

    for request in requests.iter().copied() {
        input.submit_event(InputEvent::CursorHostRequest(request));
    }

    assert_eq!(input.frame_snapshot().cursor_host_requests, requests);
    assert_eq!(input.drain_cursor_host_requests(), requests);
    assert!(input.frame_snapshot().cursor_host_requests.is_empty());

    input.submit_event(InputEvent::CursorHostRequest(
        CursorHostRequest::set_grab_mode(CursorGrabMode::None),
    ));
    input.begin_frame();

    assert!(input.frame_snapshot().cursor_host_requests.is_empty());
    assert_eq!(
        input.drain_cursor_host_requests(),
        vec![CursorHostRequest::set_grab_mode(CursorGrabMode::None)]
    );
    assert!(input.drain_cursor_host_requests().is_empty());
}
