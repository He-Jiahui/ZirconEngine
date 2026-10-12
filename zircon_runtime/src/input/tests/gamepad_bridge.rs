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
        gamepad,
        connected: true,
        name: Some("Runtime Pad".to_string()),
        vendor_id: Some(1),
        product_id: Some(2),
    }));
    input.submit_event(InputEvent::GamepadButton {
        gamepad,
        button,
        value: 1.0,
        pressed: true,
    });
    input.submit_event(InputEvent::GamepadAxis {
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
/// 当前扫描范围漏读已拆分的 gamepad 子模块，三个输入变体断言会失败；具体证据见断言附近的 BUG 标签。
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
    let session_events = include_str!("../../dynamic_api/session/events.rs");

    assert!(app_events.contains("ZrRuntimeEventV1::gamepad_connection_with_ids"));
    assert!(app_events.contains("ZrRuntimeEventV1::gamepad_button"));
    assert!(app_events.contains("ZrRuntimeEventV1::gamepad_axis"));
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
    // BUG: [CR-INPUT-BRIDGE-0001] 在当前模块拆分下运行本测试时，下面三个变体断言必然失败：
    // session.rs 与 session/events.rs 均无这些 InputEvent 字符串；实际提交位于 session/events/gamepad.rs，而 include_str! 未读取该子文件。
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
