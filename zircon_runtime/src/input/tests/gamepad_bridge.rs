//! 手柄断开状态与宿主 ABI 桥接源码守卫的回归测试。
//!
//! 状态夹具只拥有局部输入管理器；桥接守卫读取源码文本，不创建设备或动态会话。

use crate::core::framework::input::InputManager;

use crate::input::{
    DefaultInputManager, GamepadAxis, GamepadButton, GamepadConnectionInfo, GamepadId, InputButton,
    InputEvent,
};

// 对照手柄断开的状态释放和宿主 ABI 桥接源码；释放边沿只在断开当帧可见。
#[test]
/// 通过连接、按键、轴输入与断开事件，验证 [DefaultInputManager] 在断开当帧撤销设备状态并产生一次释放边沿。
///
/// # 使用时机
/// 先用 [InputManager::submit_event] 建立连接和持有输入，再读 [InputManager::frame_snapshot]，提交断开，最后用 [InputManager::begin_frame] 推进边界。
///
/// # 返回与状态
/// 测试返回 ()。断开快照移除连接、按键值和轴值并报告一次 just_released；下一帧释放边沿清除，连接仍为空。夹具只拥有局部 manager 和固定 GamepadId。
///
/// # 参考
/// [DefaultInputManager]、[InputEvent]、[InputButton]、[GamepadConnectionInfo]、[GamepadAxis]、[InputManager::submit_event]、[InputManager::frame_snapshot]、[InputManager::begin_frame]
fn gamepad_disconnect_clears_held_state_without_panic() {
    let input = DefaultInputManager::default();
    let gamepad = GamepadId(42);
    let button = GamepadButton::South;
    let input_button = InputButton::Gamepad { gamepad, button };

    input.submit_event(InputEvent::GamepadConnection(GamepadConnectionInfo {
        generation: std::num::NonZeroU64::MIN,
        gamepad,
        connected: true,
        name: Some("Runtime Pad".to_string()),
        vendor_id: Some(1),
        product_id: Some(2),
    }));
    input.submit_event(InputEvent::GamepadButton {
        generation: std::num::NonZeroU64::MIN,
        gamepad,
        button,
        value: 1.0,
        pressed: true,
    });
    input.submit_event(InputEvent::GamepadAxis {
        generation: std::num::NonZeroU64::MIN,
        gamepad,
        axis: GamepadAxis::LeftStickX,
        value: 0.75,
    });

    let connected = input.frame_snapshot();

    assert_eq!(connected.connected_gamepads, vec![gamepad]);
    assert!(connected.buttons.pressed(&input_button));
    assert_eq!(connected.gamepad_axes.len(), 1);
    assert_eq!(connected.gamepad_button_values.len(), 1);

    input.submit_event(InputEvent::GamepadConnection(GamepadConnectionInfo {
        generation: std::num::NonZeroU64::MIN,
        gamepad,
        connected: false,
        name: None,
        vendor_id: None,
        product_id: None,
    }));

    let disconnected = input.frame_snapshot();

    assert!(disconnected.connected_gamepads.is_empty());
    assert!(!disconnected.buttons.pressed(&input_button));
    assert!(disconnected.buttons.just_released(&input_button));
    assert!(disconnected.gamepad_axes.is_empty());
    assert!(disconnected.gamepad_button_values.is_empty());

    input.begin_frame();
    let next_frame = input.frame_snapshot();

    assert!(!next_frame.buttons.just_released(&input_button));
    assert!(next_frame.connected_gamepads.is_empty());
}

#[test]
/// 读取宿主手柄事件、轮询和动态会话源码，守卫 ABI 构造器与 [InputEvent] 处理链的静态连接。
///
/// # 使用时机
/// 通过 include_str! 读取 App 事件/轮询和 Runtime 会话文件，以 contains 检查构造器、连接边沿、断连震动清理及输入事件 marker。
///
/// 扫描包含实际 gamepad 与 state 子模块，三个输入变体断言保留；历史 BUG marker 仍待原定执行验证。
///
/// # 返回与限制
/// 测试返回 ()；通过只证明被读文本含 marker，不证明运行时分发、ABI 数值、模块加载或平台行为。路径变化会在编译期失败。
///
/// # 参考
/// [InputEvent]
fn gamepad_host_bridge_uses_runtime_gamepad_abi_constructors() {
    let app_events =
        include_str!("../../../../zircon_app/src/entry/runtime_entry_app/gamepad/events.rs");
    let app_polling =
        include_str!("../../../../zircon_app/src/entry/runtime_entry_app/gamepad/polling.rs");
    let session = include_str!("../../dynamic_api/session.rs");
    let session_events = concat!(
        include_str!("../../dynamic_api/session/events.rs"),
        include_str!("../../dynamic_api/session/events/gamepad.rs"),
        include_str!("../../dynamic_api/session/state.rs"),
    );

    assert!(app_events.contains("ZrRuntimeEventV1::gamepad_connection_v2"));
    assert!(app_events.contains("ZrRuntimeEventV1::gamepad_button_v2"));
    assert!(app_events.contains("ZrRuntimeEventV1::gamepad_axis_v2"));
    assert!(app_polling.contains("EventType::Connected"));
    assert!(app_polling.contains("EventType::Disconnected"));
    assert!(app_polling.contains("clear_gamepad_rumble_effects_for_gamepad"));
    assert!(
        session.contains("handle_gamepad_connection")
            || session_events.contains("handle_gamepad_connection")
    );
    assert!(
        session.contains("handle_gamepad_button")
            || session_events.contains("handle_gamepad_button")
    );
    assert!(
        session.contains("handle_gamepad_axis") || session_events.contains("handle_gamepad_axis")
    );
    // BUG: [CR-INPUT-BRIDGE-0001] 历史漏读 owner 的回归断言保留；本次 include_str! 已纳入
    // session/events/gamepad.rs 和 state.rs，实际执行验证仍 OPEN，不能据词法接线关闭该验证门槛。
    assert!(
        session.contains("InputEvent::GamepadConnection")
            || session_events.contains("InputEvent::GamepadConnection")
    );
    assert!(
        session.contains("InputEvent::GamepadButton")
            || session_events.contains("InputEvent::GamepadButton")
    );
    assert!(
        session.contains("InputEvent::GamepadAxis")
            || session_events.contains("InputEvent::GamepadAxis")
    );
}

fn gamepad_generation(value: u64) -> std::num::NonZeroU64 {
    std::num::NonZeroU64::new(value).unwrap()
}

fn gamepad_lifecycle(gamepad: GamepadId, generation: u64, connected: bool) -> InputEvent {
    InputEvent::GamepadConnection(GamepadConnectionInfo {
        gamepad, generation: gamepad_generation(generation), connected,
        name: None, vendor_id: None, product_id: None,
    })
}

fn gamepad_axis_sample(gamepad: GamepadId, generation: u64, value: f32) -> InputEvent {
    InputEvent::GamepadAxis {
        gamepad, generation: gamepad_generation(generation),
        axis: GamepadAxis::LeftStickX, value,
    }
}

fn gamepad_button_sample(gamepad: GamepadId, generation: u64, pressed: bool) -> InputEvent {
    InputEvent::GamepadButton {
        gamepad, generation: gamepad_generation(generation),
        button: GamepadButton::South, value: if pressed { 1.0 } else { 0.0 }, pressed,
    }
}

#[test]
fn gamepad_samples_require_an_open_matching_generation() {
    use crate::core::framework::input::{GamepadInputRejection as Rejection, InputEventAdmission as Admission};
    let input = DefaultInputManager::default();
    let gamepad = GamepadId(101);
    input.set_event_recording_config(crate::input::InputEventRecordingConfig::enabled(64));
    for event in [gamepad_axis_sample(gamepad, 4, 0.75), gamepad_button_sample(gamepad, 4, true)] {
        assert_eq!(input.submit_event_with_admission(event), Admission::Rejected(
            Rejection::ConnectionNotObserved { gamepad, submitted: gamepad_generation(4) }
        ));
    }
    assert!(input.frame_snapshot().gamepad_axes.is_empty());
    assert!(input.frame_snapshot().buttons.pressed_inputs().is_empty());
    assert!(input.drain_events().is_empty());
    assert!(input.drain_event_records().is_empty());

    assert_eq!(input.submit_event_with_admission(gamepad_lifecycle(gamepad, 4, true)), Admission::Accepted);
    input.drain_events();
    input.drain_event_records();
    assert_eq!(input.submit_event_with_admission(gamepad_axis_sample(gamepad, 3, 1.0)), Admission::Rejected(
        Rejection::StaleGeneration { gamepad, submitted: gamepad_generation(3), current: gamepad_generation(4) }
    ));
    assert_eq!(input.submit_event_with_admission(gamepad_button_sample(gamepad, 5, true)), Admission::Rejected(
        Rejection::FutureGeneration { gamepad, submitted: gamepad_generation(5), current: gamepad_generation(4) }
    ));
    assert!(input.frame_snapshot().gamepad_axes.is_empty());
    assert!(input.frame_snapshot().gamepad_button_values.is_empty());
    assert!(input.drain_events().is_empty());
    assert!(input.drain_event_records().is_empty());
    assert_eq!(input.submit_event_with_admission(gamepad_axis_sample(gamepad, 4, 0.75)), Admission::Accepted);
    assert_eq!(input.submit_event_with_admission(gamepad_button_sample(gamepad, 4, true)), Admission::Accepted);
    assert_eq!(input.frame_snapshot().gamepad_axes[0].value,
        crate::input::GamepadAxisSettings::default().scaled_value(0.75));
    assert!(input.frame_snapshot().buttons.pressed(&InputButton::Gamepad {
        gamepad, button: GamepadButton::South,
    }));
}

#[test]
fn gamepad_reconnect_rejects_stale_samples_and_stale_disconnect() {
    use crate::core::framework::input::{GamepadInputRejection as Rejection, InputEventAdmission as Admission};
    let input = DefaultInputManager::default();
    let gamepad = GamepadId(102);
    input.set_event_recording_config(crate::input::InputEventRecordingConfig::enabled(64));
    for event in [
        gamepad_lifecycle(gamepad, 1, true), gamepad_axis_sample(gamepad, 1, 0.5),
        gamepad_lifecycle(gamepad, 1, false), gamepad_lifecycle(gamepad, 2, true),
        gamepad_axis_sample(gamepad, 2, -0.75), gamepad_button_sample(gamepad, 2, true),
    ] {
        assert_eq!(input.submit_event_with_admission(event), Admission::Accepted);
    }
    input.begin_frame();
    input.drain_events();
    input.drain_event_records();
    let before = input.frame_snapshot();
    for event in [
        gamepad_axis_sample(gamepad, 1, 1.0), gamepad_button_sample(gamepad, 1, false),
        gamepad_lifecycle(gamepad, 1, false), gamepad_lifecycle(gamepad, 1, true),
    ] {
        assert_eq!(input.submit_event_with_admission(event), Admission::Rejected(
            Rejection::StaleGeneration { gamepad, submitted: gamepad_generation(1), current: gamepad_generation(2) }
        ));
    }
    let after = input.frame_snapshot();
    assert_eq!(after.connected_gamepads, before.connected_gamepads);
    assert_eq!(after.gamepad_axes, before.gamepad_axes);
    assert_eq!(after.gamepad_button_values, before.gamepad_button_values);
    assert_eq!(after.gamepad_axis_transitions, before.gamepad_axis_transitions);
    assert_eq!(after.buttons.pressed_inputs(), before.buttons.pressed_inputs());
    assert!(after.buttons.just_released_inputs().is_empty());
    assert!(input.drain_events().is_empty());
    assert!(input.drain_event_records().is_empty());
}

#[test]
fn gamepad_closed_generation_cannot_reopen_and_future_disconnect_closes_once() {
    use crate::core::framework::input::{GamepadInputRejection as Rejection, InputEventAdmission as Admission};
    let input = DefaultInputManager::default();
    let gamepad = GamepadId(103);
    input.submit_event(gamepad_lifecycle(gamepad, 1, true));
    input.submit_event(gamepad_axis_sample(gamepad, 1, 0.75));
    input.submit_event(gamepad_button_sample(gamepad, 1, true));
    input.begin_frame();
    assert_eq!(input.submit_event_with_admission(gamepad_lifecycle(gamepad, 5, false)), Admission::Accepted);
    let closed = input.frame_snapshot();
    assert!(closed.connected_gamepads.is_empty());
    assert!(closed.gamepad_axes.is_empty());
    assert!(closed.buttons.just_released(&InputButton::Gamepad { gamepad, button: GamepadButton::South }));
    assert_eq!(closed.gamepad_axis_transitions.len(), 1);
    assert_eq!(closed.gamepad_axis_transitions[0].previous_value,
        crate::input::GamepadAxisSettings::default().scaled_value(0.75));
    assert_eq!(closed.gamepad_axis_transitions[0].value, 0.0);
    input.begin_frame();
    input.drain_events();
    assert_eq!(input.submit_event_with_admission(gamepad_lifecycle(gamepad, 5, false)), Admission::Accepted);
    assert!(input.frame_snapshot().buttons.just_released_inputs().is_empty());
    assert!(input.frame_snapshot().gamepad_axis_transitions.is_empty());
    for event in [gamepad_lifecycle(gamepad, 5, true), gamepad_axis_sample(gamepad, 5, 1.0)] {
        assert_eq!(input.submit_event_with_admission(event), Admission::Rejected(
            Rejection::ConnectionClosed { gamepad, submitted: gamepad_generation(5), current: gamepad_generation(5) }
        ));
    }
    assert_eq!(input.submit_event_with_admission(gamepad_lifecycle(gamepad, 6, true)), Admission::Accepted);
    assert_eq!(input.submit_event_with_admission(gamepad_axis_sample(gamepad, 6, -1.0)), Admission::Accepted);
    assert_eq!(input.frame_snapshot().gamepad_axes[0].value, -1.0);

    let unseen = GamepadId(104);
    assert_eq!(input.submit_event_with_admission(gamepad_lifecycle(unseen, 9, false)), Admission::Accepted);
    assert_eq!(input.submit_event_with_admission(gamepad_lifecycle(unseen, 9, true)), Admission::Rejected(
        Rejection::ConnectionClosed { gamepad: unseen, submitted: gamepad_generation(9), current: gamepad_generation(9) }
    ));
}

#[test]
fn gamepad_generic_button_events_require_generation() {
    use crate::core::framework::input::{GamepadInputRejection as Rejection, InputEventAdmission as Admission};
    let input = DefaultInputManager::default();
    let gamepad = GamepadId(105);
    let button = InputButton::Gamepad { gamepad, button: GamepadButton::South };
    input.set_event_recording_config(crate::input::InputEventRecordingConfig::enabled(64));
    input.submit_event(gamepad_lifecycle(gamepad, 1, true));
    input.submit_event(gamepad_button_sample(gamepad, 1, true));
    input.begin_frame();
    input.drain_events();
    input.drain_event_records();
    for event in [InputEvent::ButtonReleased(button.clone()), InputEvent::ButtonPressed(button.clone())] {
        assert_eq!(input.submit_event_with_admission(event), Admission::Rejected(
            Rejection::GenerationRequired { gamepad }
        ));
    }
    // The compatibility unit entry must use the same gate, including its release path.
    input.submit_event(InputEvent::ButtonReleased(button.clone()));
    let frame = input.frame_snapshot();
    assert!(frame.buttons.pressed(&button));
    assert!(frame.buttons.just_pressed_inputs().is_empty());
    assert!(frame.buttons.just_released_inputs().is_empty());
    assert!(input.drain_events().is_empty());
    assert!(input.drain_event_records().is_empty());
}
