//! 验证内存输入录制的帧捕获、完整性元数据和按存储顺序回放。
//!
//! # 使用时机
//! 由输入测试模块挂载，测试框架运行同步用例；捕获用例读取本地管理器的原始记录，
//! 回放用例使用构造的完整帧，分别检查记录值和回放后的输入快照。
//!
//! # 所有权与边界
//! 捕获帧拥有排空取得的记录，录制拥有加入的帧，回放游标借用录制。
//! 所有值随用例结束释放；快照内的 IME 和光标请求是待宿主处理的请求，
//! 这些用例不启动宿主服务，也不确定不完整帧的回放策略。
//!
//! # 参考
//! [crate::input::InputRecordingFrame]、[crate::input::InputRecording]、
//! [crate::input::InputReplayCursor]

use crate::core::framework::input::InputManager;
use crate::input::{
    CursorGrabMode, CursorHostRequest, DefaultInputManager, GamepadAxis, GamepadId, ImeHostRequest,
    InputButton, InputEvent, InputEventRecordingConfig, InputRecording, InputRecordingFrame,
};

// 记录器保留原始事件与完整性状态；游标回放应重建按键、轴和失焦释放结果。
/// 启用有界原始记录后捕获一帧，检查捕获排空记录，并验证该帧加入 InputRecording 后的计数和完整性。
///
/// # 调用与生命周期
/// 单个管理器的记录由 capture_from_manager 消费，再由本地 recording 持有。
///
/// # 参数
/// 无显式参数；测试框架调用此入口，夹具与提交序列由函数内部建立。
///
/// # 返回值
/// 正常返回 `()` 表示本用例的断言均通过；断言或 unwrap 失败会 panic，由测试框架报告失败，不返回业务数据。
///
/// # 验证范围
/// 容量 16 的记录器收到两个事件后捕获编号 17 的帧；捕获排空原始记录，加入帧的克隆后录制拥有一帧、两个事件及对应完整性。测试不捕获或排空独立帧事件队列。
///
/// # 参考
/// [crate::input::InputRecordingFrame::capture_from_manager]、[crate::input::InputRecording::push_frame]、[crate::input::InputRecording::is_complete]、[crate::core::framework::input::InputManager::drain_event_records]
#[test]
fn input_recording_captures_drainable_event_records_by_frame() {
    let input = DefaultInputManager::default();
    input.set_event_recording_config(InputEventRecordingConfig::enabled(16));
    input.submit_event(InputEvent::CursorMoved { x: 4.0, y: 9.0 });
    input.submit_event(InputEvent::ButtonPressed(InputButton::MouseLeft));

    let frame = InputRecordingFrame::capture_from_manager(17, &input);
    assert_eq!(frame.frame_index(), 17);
    assert_eq!(frame.event_count(), 2);
    assert!(frame.is_complete());
    assert!(input.drain_event_records().is_empty());

    let mut recording = InputRecording::new();
    recording.push_frame(frame.clone());

    assert_eq!(recording.frame_count(), 1);
    assert_eq!(recording.event_count(), 2);
    assert!(recording.is_complete());
    assert_eq!(recording.frames(), &[frame]);
}

/// 验证有界原始记录发生淘汰后，捕获帧与包含该帧的录制都报告不完整。
///
/// # 调用与生命周期
/// 启用容量 2 的记录周期在一次捕获后封入本地录制。此测试仅检查完整性元数据。
///
/// # 参数
/// 无显式参数；测试框架调用此入口，夹具与提交序列由函数内部建立。
///
/// # 返回值
/// 正常返回 `()` 表示本用例的断言均通过；断言或 unwrap 失败会 panic，由测试框架报告失败，不返回业务数据。
///
/// # 验证范围
/// 三个事件进入容量 2 的原始记录队列，捕获保留两个并报告一次淘汰；断言只检查捕获帧与录制的完整性，不确定不完整帧应如何回放。
///
/// # 参考
/// [crate::core::framework::input::InputManager::set_event_recording_config]、[crate::input::InputRecordingFrame::capture_from_manager]、[crate::input::InputRecording::from_frames]、[crate::input::InputRecording::is_complete]
#[test]
fn input_recording_marks_a_bounded_capture_incomplete_after_discard() {
    let input = DefaultInputManager::default();
    input.set_event_recording_config(InputEventRecordingConfig::enabled(2));
    input.submit_event(InputEvent::CursorMoved { x: 1.0, y: 0.0 });
    input.submit_event(InputEvent::CursorMoved { x: 2.0, y: 0.0 });
    input.submit_event(InputEvent::CursorMoved { x: 3.0, y: 0.0 });

    let frame = InputRecordingFrame::capture_from_manager(5, &input);
    assert_eq!(frame.event_count(), 2);
    assert_eq!(frame.discarded_record_count(), 1);
    assert!(!frame.is_complete());

    let recording = InputRecording::from_frames(vec![frame]);
    assert_eq!(recording.discarded_record_count(), 1);
    assert!(!recording.is_complete());
}

/// 验证未启用记录时，捕获帧为空且不完整，并使包含该帧的录制也不完整。
///
/// # 调用与生命周期
/// 一次普通输入提交后立即捕获，不读取独立帧事件队列。
///
/// # 参数
/// 无显式参数；测试框架调用此入口，夹具与提交序列由函数内部建立。
///
/// # 返回值
/// 正常返回 `()` 表示本用例的断言均通过；断言或 unwrap 失败会 panic，由测试框架报告失败，不返回业务数据。
///
/// # 验证范围
/// 空值仅指 [crate::input::InputRecordingFrame::is_empty] 检查的原始记录集合；普通输入已提交，本用例未检查独立帧事件队列或输入快照是否为空。
///
/// # 参考
/// [crate::input::InputRecordingFrame::capture_from_manager]、[crate::input::InputRecording::from_frames]、[crate::input::InputRecording::is_complete]
#[test]
fn input_recording_marks_capture_incomplete_when_recording_is_disabled() {
    let input = DefaultInputManager::default();
    input.submit_event(InputEvent::CursorMoved { x: 1.0, y: 2.0 });

    let frame = InputRecordingFrame::capture_from_manager(6, &input);

    assert!(!frame.recording_enabled());
    assert!(frame.is_empty());
    assert!(!frame.is_complete());
    assert!(!InputRecording::from_frames(vec![frame]).is_complete());
}

/// 按存储顺序重放两个合成帧，检查帧编号、按键边沿、光标快照和游标结束行为。
///
/// # 调用与生命周期
/// 游标借用本地完整录制，replay_next_frame 为每个帧开始新的管理器帧。
///
/// # 参数
/// 无显式参数；测试框架调用此入口，夹具与提交序列由函数内部建立。
///
/// # 返回值
/// 正常返回 `()` 表示本用例的断言均通过；断言或 unwrap 失败会 panic，由测试框架报告失败，不返回业务数据。
///
/// # 验证范围
/// 两个帧由 from_events 构造成启用记录且无淘汰的合成夹具，序号从 1 开始、时间戳为 0；断言关注存储顺序、快照和结束后的 None，不验证真实捕获完整性或时间戳调度。
///
/// # 参考
/// [crate::input::InputRecordingFrame::from_events]、[crate::input::InputRecording::replay_cursor]、[crate::input::InputReplayCursor::replay_next_frame]、[crate::input::InputReplayCursor::is_finished]
#[test]
fn input_replay_restores_frame_snapshots_in_recorded_order() {
    let key = InputButton::KeyCode(42);
    let recording = InputRecording::from_frames(vec![
        InputRecordingFrame::from_events(
            3,
            [
                InputEvent::CursorMoved { x: 8.0, y: 13.0 },
                InputEvent::ButtonPressed(key.clone()),
            ],
        ),
        InputRecordingFrame::from_events(4, [InputEvent::ButtonReleased(key.clone())]),
    ]);
    let input = DefaultInputManager::default();
    let mut cursor = recording.replay_cursor();

    assert_eq!(cursor.next_recording_frame_index(), Some(3));
    let pressed = cursor.replay_next_frame(&input).unwrap();
    assert_eq!(pressed.frame_index, 3);
    assert_eq!(pressed.event_count, 2);
    assert_eq!(pressed.snapshot.cursor_position, [8.0, 13.0]);
    assert!(pressed.snapshot.buttons.pressed(&key));
    assert!(pressed.snapshot.buttons.just_pressed(&key));

    assert_eq!(cursor.next_recording_frame_index(), Some(4));
    let released = cursor.replay_next_frame(&input).unwrap();
    assert_eq!(released.frame_index, 4);
    assert_eq!(released.event_count, 1);
    assert!(!released.snapshot.buttons.pressed(&key));
    assert!(released.snapshot.buttons.just_released(&key));
    assert!(cursor.is_finished());
    assert!(cursor.replay_next_frame(&input).is_none());
}

/// 重放按键、游戏杆轴与宿主请求后应用 FocusLost，检查按键释放、轴复位转换及 IME/光标待处理请求。
///
/// # 调用与生命周期
/// 游标按序消费两个合成完整帧，管理器和游标均为本地值，不启动宿主服务。
///
/// # 参数
/// 无显式参数；测试框架调用此入口，夹具与提交序列由函数内部建立。
///
/// # 返回值
/// 正常返回 `()` 表示本用例的断言均通过；断言或 unwrap 失败会 panic，由测试框架报告失败，不返回业务数据。
///
/// # 验证范围
/// 第二帧先重置帧增量，再处理失焦；断言检查按键释放、轴从 1.0 到 0.0 的转换和待处理的 IME Disable、光标 None 请求。请求不表示宿主已执行禁用或释放光标。
///
/// # 参考
/// [crate::input::InputReplayCursor::replay_next_frame]、[crate::core::framework::input::InputManager::frame_snapshot]、[crate::input::GamepadAxisTransition]、[crate::input::ImeHostRequest]、[crate::input::CursorHostRequest]
#[test]
fn input_replay_applies_focus_loss_as_a_release_transaction() {
    let key = InputButton::KeyCode(42);
    let gamepad = GamepadId(7);
    let axis = GamepadAxis::LeftStickX;
    let recording = InputRecording::from_frames(vec![
        InputRecordingFrame::from_events(
            3,
            [
                InputEvent::ButtonPressed(key.clone()),
                InputEvent::GamepadConnection(crate::input::GamepadConnectionInfo {
                    gamepad,
                    generation: std::num::NonZeroU64::MIN,
                    connected: true,
                    name: None,
                    vendor_id: None,
                    product_id: None,
                }),
                InputEvent::GamepadAxis {
                    generation: std::num::NonZeroU64::MIN,
                    gamepad,
                    axis,
                    value: 1.0,
                },
                InputEvent::ImeHostRequest(ImeHostRequest::Enable),
                InputEvent::CursorHostRequest(CursorHostRequest::set_grab_mode(
                    CursorGrabMode::Locked,
                )),
            ],
        ),
        InputRecordingFrame::from_events(4, [InputEvent::FocusLost]),
    ]);
    let input = DefaultInputManager::default();
    let mut cursor = recording.replay_cursor();

    let held = cursor.replay_next_frame(&input).unwrap();
    assert!(held.snapshot.buttons.pressed(&key));
    assert_eq!(held.snapshot.gamepad_axes.len(), 1);

    let released = cursor.replay_next_frame(&input).unwrap();
    assert!(!released.snapshot.buttons.pressed(&key));
    assert!(released.snapshot.buttons.just_released(&key));
    assert!(released.snapshot.gamepad_axes.is_empty());
    assert!(released
        .snapshot
        .gamepad_axis_transitions
        .iter()
        .any(|transition| {
            transition.gamepad == gamepad
                && transition.axis == axis
                && transition.previous_value == 1.0
                && transition.value == 0.0
        }));
    assert_eq!(
        released.snapshot.ime_host_requests,
        vec![ImeHostRequest::Disable]
    );
    assert_eq!(
        released.snapshot.cursor_host_requests,
        vec![CursorHostRequest::set_grab_mode(CursorGrabMode::None)]
    );
}

#[test]
fn gamepad_replay_preserves_generation_and_rejects_missing_connection_history() {
    use std::num::NonZeroU64;
    use crate::input::GamepadConnectionInfo;

    let gamepad = GamepadId(73);
    let generation = NonZeroU64::new(7).unwrap();
    let connection = |connected| InputEvent::GamepadConnection(GamepadConnectionInfo {
        gamepad,
        generation,
        connected,
        name: None,
        vendor_id: None,
        product_id: None,
    });
    let sample = InputEvent::GamepadAxis {
        gamepad,
        generation,
        axis: GamepadAxis::LeftStickX,
        value: 1.0,
    };
    let events = vec![connection(true), sample.clone()];
    let recording = InputRecording::from_frames(vec![
        InputRecordingFrame::from_events(12, events.clone()),
    ]);
    let fresh = DefaultInputManager::default();
    fresh.set_event_recording_config(InputEventRecordingConfig::enabled(16));
    let replay = recording.replay_cursor().replay_next_frame(&fresh).unwrap();
    assert_eq!(replay.snapshot.connected_gamepads, vec![gamepad]);
    assert_eq!(replay.snapshot.gamepad_axes.len(), 1);
    assert_eq!(fresh.drain_events(), events);
    assert_eq!(
        fresh.drain_event_records().into_iter().map(|record| record.event).collect::<Vec<_>>(),
        events,
    );

    // A cursor preserves recorded stamps; a sample-only fragment is not self-contained.
    let fragment = InputRecording::from_frames(vec![
        InputRecordingFrame::from_events(13, [sample.clone()]),
    ]);
    let missing_history = DefaultInputManager::default();
    let fragment_result = fragment.replay_cursor().replay_next_frame(&missing_history).unwrap();
    assert_eq!(fragment_result.event_count, 1); // attempted record count, not admission count
    assert!(fragment_result.snapshot.gamepad_axes.is_empty());
    assert!(missing_history.drain_events().is_empty());

    // Replaying an older complete capture into a later live tombstone cannot reopen it.
    let later = DefaultInputManager::default();
    later.submit_event(InputEvent::GamepadConnection(GamepadConnectionInfo {
        gamepad,
        generation: NonZeroU64::new(8).unwrap(),
        connected: false,
        name: None,
        vendor_id: None,
        product_id: None,
    }));
    later.drain_events();
    let rejected = recording.replay_cursor().replay_next_frame(&later).unwrap();
    assert!(rejected.snapshot.connected_gamepads.is_empty());
    assert!(rejected.snapshot.gamepad_axes.is_empty());
    assert!(later.drain_events().is_empty());

    let serialized = serde_json::to_value(&sample).unwrap();
    let mut missing_generation = serialized.clone();
    missing_generation["GamepadAxis"].as_object_mut().unwrap().remove("generation");
    assert!(serde_json::from_value::<InputEvent>(missing_generation).is_err());
    let mut zero_generation = serialized;
    zero_generation["GamepadAxis"]["generation"] = serde_json::json!(0);
    assert!(serde_json::from_value::<InputEvent>(zero_generation).is_err());
}
