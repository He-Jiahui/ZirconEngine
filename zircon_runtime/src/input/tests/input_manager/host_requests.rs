//! 输入管理器震动与光标宿主请求的快照窗口及排空契约测试。
//!
//! 每个测试独占局部队列，区分本帧视图和跨帧仍可消费的请求；实际宿主执行不在此模块中。

use super::*;
use std::num::NonZeroU64;

use crate::core::framework::input::{
    GamepadConnectionInfo, GamepadInputRejection, InputEventAdmission, InputEventRecordingConfig,
    QueuedGamepadRumbleRequest,
};

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
    let generation = NonZeroU64::new(1).unwrap();
    input.submit_event(rumble_connection_event(gamepad, generation, true));
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

    assert_eq!(
        input.drain_gamepad_rumble_requests(),
        vec![
            QueuedGamepadRumbleRequest::new(add, generation),
            QueuedGamepadRumbleRequest::new(stop, generation),
        ],
    );
    assert!(input.frame_snapshot().gamepad_rumble_requests.is_empty());

    input.submit_event(InputEvent::GamepadRumbleRequest(add));
    input.begin_frame();

    assert!(input.frame_snapshot().gamepad_rumble_requests.is_empty());
    assert_eq!(
        input.drain_gamepad_rumble_requests(),
        vec![QueuedGamepadRumbleRequest::new(add, generation)],
    );
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

fn rumble_connection_event(
    gamepad: GamepadId,
    generation: NonZeroU64,
    connected: bool,
) -> InputEvent {
    InputEvent::GamepadConnection(GamepadConnectionInfo {
        gamepad,
        generation,
        connected,
        name: None,
        vendor_id: None,
        product_id: None,
    })
}

#[test]
fn gamepad_rumble_rejects_unobserved_and_closed_targets_without_retention() {
    let input = DefaultInputManager::default();
    let gamepad = GamepadId(51);
    let generation = NonZeroU64::new(1).unwrap();
    input.set_event_recording_config(InputEventRecordingConfig::enabled(16));
    let requests = [
        GamepadRumbleRequest::add(gamepad, GamepadRumbleIntensity::new(0.5, 1.0), 125),
        GamepadRumbleRequest::stop(gamepad),
    ];
    for connected_once in [false, true] {
        if connected_once {
            input.submit_event(rumble_connection_event(gamepad, generation, true));
            input.submit_event(rumble_connection_event(gamepad, generation, false));
            let _ = input.drain_events();
            let _ = input.drain_event_records();
        }
        for request in requests {
            assert_eq!(
                input.submit_event_with_admission(InputEvent::GamepadRumbleRequest(request)),
                InputEventAdmission::Rejected(GamepadInputRejection::RumbleTargetUnavailable {
                    gamepad
                },),
            );
        }
        assert!(input.frame_snapshot().gamepad_rumble_requests.is_empty());
        assert!(input.drain_gamepad_rumble_requests().is_empty());
        assert!(input.drain_events().is_empty());
        assert!(input.drain_event_records().is_empty());
    }
}

#[test]
fn gamepad_rumble_queue_retains_enqueue_generations_across_reconnects() {
    let input = DefaultInputManager::default();
    let gamepad = GamepadId(52);
    let other = GamepadId(53);
    let first = NonZeroU64::new(1).unwrap();
    let second = NonZeroU64::new(2).unwrap();
    let add = GamepadRumbleRequest::add(gamepad, GamepadRumbleIntensity::new(0.5, 1.0), 125);
    let stop = GamepadRumbleRequest::stop(gamepad);
    let other_stop = GamepadRumbleRequest::stop(other);
    input.submit_event(rumble_connection_event(gamepad, first, true));
    input.submit_event(InputEvent::GamepadRumbleRequest(add));
    input.submit_event(InputEvent::GamepadRumbleRequest(stop));
    assert_eq!(
        input.frame_snapshot().gamepad_rumble_requests,
        vec![add, stop]
    );
    input.begin_frame();
    // Same-generation connect is idempotent and cannot discard the accepted FIFO prefix.
    input.submit_event(rumble_connection_event(gamepad, first, true));
    input.submit_event(rumble_connection_event(gamepad, first, false));
    input.submit_event(rumble_connection_event(gamepad, second, true));
    input.submit_event(rumble_connection_event(other, first, true));
    input.submit_event(InputEvent::GamepadRumbleRequest(add));
    input.submit_event(InputEvent::GamepadRumbleRequest(other_stop));
    assert_eq!(
        input.frame_snapshot().gamepad_rumble_requests,
        vec![add, other_stop]
    );
    assert_eq!(
        input.drain_gamepad_rumble_requests(),
        vec![
            QueuedGamepadRumbleRequest::new(add, first),
            QueuedGamepadRumbleRequest::new(stop, first),
            QueuedGamepadRumbleRequest::new(add, second),
            QueuedGamepadRumbleRequest::new(other_stop, first),
        ],
    );
    assert!(input.frame_snapshot().gamepad_rumble_requests.is_empty());
    assert!(input.drain_gamepad_rumble_requests().is_empty());
}
