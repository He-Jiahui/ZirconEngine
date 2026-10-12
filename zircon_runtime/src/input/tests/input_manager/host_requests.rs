//! 输入管理器震动与光标宿主请求的快照窗口及排空契约测试。
//!
//! 每个测试独占局部队列，区分本帧视图和跨帧仍可消费的请求；实际宿主执行不在此模块中。

use super::*;

// 运行时生成的震动和光标请求由宿主排空；帧快照只呈现本帧新请求。
#[test]
/// 验证手柄震动请求提供本帧快照视图和一次性宿主排空队列。
///
/// # 使用时机
/// 用 [InputManager::submit_event] 提交 Add/Stop，先读 [InputManager::frame_snapshot]，再调 [InputManager::drain_gamepad_rumble_requests]；用 [InputManager::begin_frame] 区分快照窗口和仍可排空的队列。
///
/// # 返回与状态
/// 测试返回 ()。快照保序，排空转移并清空队列；帧边界隐藏未消费请求但排空仍取得它们。GamepadRumbleIntensity::clamped 断言独立验证 NaN/越界归一化，不改变队列值。
///
/// # 参考
/// [DefaultInputManager]、[GamepadRumbleRequest]、[GamepadRumbleIntensity]、[InputEvent]、[InputManager::submit_event]、[InputManager::frame_snapshot]、[InputManager::drain_gamepad_rumble_requests]、[InputManager::begin_frame]
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
/// 验证光标宿主请求在帧快照和排空接口之间保持顺序及窗口边界。
///
/// # 使用时机
/// 将四种 [CursorHostRequest] 依序提交给 [DefaultInputManager]，读 [InputManager::frame_snapshot] 后调 [InputManager::drain_cursor_host_requests]；提交 None 后推进 [InputManager::begin_frame]。
///
/// # 返回与状态
/// 测试返回 ()。首批按序出现在快照和排空结果，排空后快照为空；帧边界隐藏新请求但后续排空仍转移它们，再次排空为空。
///
/// # 参考
/// [DefaultInputManager]、[CursorHostRequest]、[CursorGrabMode]、[InputEvent]、[InputManager::submit_event]、[InputManager::frame_snapshot]、[InputManager::drain_cursor_host_requests]、[InputManager::begin_frame]
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
