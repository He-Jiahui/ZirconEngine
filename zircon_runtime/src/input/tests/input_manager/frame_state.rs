//! 检查默认输入管理器的帧窗口、原始录制和通知保留边界。
//!
//! # 使用时机与责任
//! 由输入管理器测试模块在测试构建中挂载；各情景独立构造状态，通过快照、排空及源文本守卫检查约定。源文本检查只说明所读取文件的词法形态，不代替动态会话执行。
//!
//! # 生命周期与限制
//! 管理器和返回数据由各测试本地拥有，不使用平台设备。旧帧快照是独立副本；帧清理、请求排空、原始记录和失焦取消具有不同保留范围。墙钟断言另有待确认前提。
//!
//! # 参考
//! 参见 [`crate::input::DefaultInputManager`]、[`crate::core::framework::input::InputManager`] 与 [`crate::input::InputFrameSnapshot`]。

use super::*;

/// 验证默认输入管理器把提交事件反映到兼容快照，并可独立排空即时事件。
///
/// # 使用时机
///
/// 检查 [crate::input::DefaultInputManager] 收到 [crate::core::framework::input::InputManager::submit_event()] 后的快照与排空。
///
/// # 覆盖范围与限制
///
/// 断言光标、鼠标按住项、滚轮累计值及首次排空三条事件；不覆盖合并、录制或帧重置。
///
/// # 生命周期
///
/// 用例不调用 begin_frame；第二次 drain_events 用于观察即时队列已取走。
///
/// # 参考
///
/// [crate::core::framework::input::InputManager::snapshot()] 与 [crate::core::framework::input::InputManager::drain_events()]
#[test]
fn input_manager_tracks_state_and_drains_events() {
    let input = DefaultInputManager::default();
    input.submit_event(InputEvent::CursorMoved { x: 42.0, y: 12.0 });
    input.submit_event(InputEvent::ButtonPressed(InputButton::MouseLeft));
    input.submit_event(InputEvent::WheelScrolled { delta: 1.5 });

    let snapshot = input.snapshot();
    assert_eq!(snapshot.cursor_position, [42.0, 12.0]);
    assert_eq!(snapshot.pressed_buttons, vec![InputButton::MouseLeft]);
    assert_eq!(snapshot.wheel_accumulator, 1.5);

    let drained = input.drain_events();
    assert_eq!(drained.len(), 3);
    assert!(input.drain_events().is_empty());
}

/// 验证原始录制为两条提交记录分配连续序号并可排空。
///
/// # 使用时机
///
/// 观察 [crate::core::framework::input::InputManager::set_event_recording_config()] 与记录排空的衔接。
///
/// # 覆盖范围与限制
///
/// 断言序号、正时间戳、样本内时间不回退及第二次排空为空；不保证墙钟单调或覆盖淘汰。
///
/// # 生命周期
///
/// 容量为 16；排空转移记录，不代表关闭录制或重置周期。
///
/// # 参考
///
/// [crate::input::DefaultInputManager] 与 [crate::core::framework::input::InputManager::drain_event_records()]
#[test]
fn input_manager_records_sequences_and_timestamps_for_ui_bridge_consumers() {
    let input = DefaultInputManager::default();
    input.set_event_recording_config(crate::input::InputEventRecordingConfig::enabled(16));
    input.submit_event(InputEvent::ButtonPressed(InputButton::MouseLeft));
    input.submit_event(InputEvent::ButtonReleased(InputButton::MouseLeft));

    let records = input.drain_event_records();

    assert_eq!(records.len(), 2);
    assert_eq!(records[0].sequence, 1);
    assert_eq!(records[1].sequence, 2);
    assert!(records[0].timestamp_millis > 0);
    // TODO: [CR-INPUT-FRAME-0002] 确认公共时间戳是否要求非递减；录制器直接使用 SystemTime::now()，时钟回退时下面比较可能失败。
    // 尚无可控回退复现或时钟夹具；下一步先明确契约，再用可注入时钟验证回退并决定断言去留。
    assert!(records[1].timestamp_millis >= records[0].timestamp_millis);
    assert!(input.drain_event_records().is_empty());
}

/// 验证按钮状态对象将按住集合与当前帧边沿分开维护。
///
/// # 使用时机
///
/// 说明首次/重复按下、清边沿和释放各自影响哪些状态。
///
/// # 覆盖范围与限制
///
/// 只直接操作一个鼠标按钮，不经过管理器路由或真实帧调度。
///
/// # 生命周期
///
/// clear_transitions 清除两个边沿而保留 pressed；release 在本帧产生释放边沿。
///
/// # 参考
///
/// [crate::input::ButtonInputState::press()]、[crate::input::ButtonInputState::clear_transitions()]、[crate::input::ButtonInputState::release()]
#[test]
fn button_input_state_tracks_bevy_style_frame_transitions() {
    let mut buttons = ButtonInputState::default();
    let left = InputButton::MouseLeft;

    assert!(buttons.press(left.clone()));
    assert!(!buttons.press(left.clone()));
    assert!(buttons.pressed(&left));
    assert!(buttons.just_pressed(&left));
    assert_eq!(buttons.just_pressed_inputs(), vec![left.clone()]);

    buttons.clear_transitions();

    assert!(buttons.pressed(&left));
    assert!(!buttons.just_pressed(&left));
    assert!(buttons.release(&left));
    assert!(!buttons.release(&left));
    assert!(!buttons.pressed(&left));
    assert!(buttons.just_released(&left));
}

/// 以源文本标记检查按钮 release 使用集合 take 取出存储键。
///
/// # 使用时机
///
/// 源码字符串守卫用于维护 release 的所有权实现约束。
///
/// # 覆盖范围与限制
///
/// 只检查 take 片段存在且 remove 片段不存在；不解析、不执行 [crate::input::ButtonInputState::release()]，不证明运行时行为。
///
/// # 生命周期
///
/// 字符串检查没有运行时状态生命周期。
///
/// # 参考
///
/// [crate::input::ButtonInputState::release()]
#[test]
fn button_release_moves_the_stored_owned_key_into_transition_state() {
    let source = include_str!("../../../core/framework/input/button_input_state.rs");
    assert!(source.contains("self.pressed.take(input)"));
    assert!(
        !source.contains("self.pressed.remove(input)"),
        "release must not discard the stored key and clone the lookup key"
    );
}

/// 验证连续快照保留 held 状态，并在 begin_frame 清除按钮边沿。
///
/// # 使用时机
///
/// 跟踪 [crate::core::framework::input::InputManager::begin_frame()] 前后的按下、保持和释放时点。
///
/// # 覆盖范围与限制
///
/// 只断言一个 Jump 键，不覆盖其他设备、重复按下或录制。
///
/// # 生命周期
///
/// held 跨下一帧保留；按下/释放边沿在对应帧出现并由后续 begin_frame 清除。
///
/// # 参考
///
/// [crate::input::DefaultInputManager] 与 [crate::core::framework::input::InputManager::frame_snapshot()]
#[test]
fn input_snapshot_just_pressed_is_true_for_exactly_one_frame() {
    let input = DefaultInputManager::default();
    let key = InputButton::Key("Jump".to_string());

    input.begin_frame();
    input.submit_event(InputEvent::ButtonPressed(key.clone()));
    let pressed_frame = input.frame_snapshot();

    assert!(pressed_frame.buttons.pressed(&key));
    assert!(pressed_frame.buttons.just_pressed(&key));
    assert!(!pressed_frame.buttons.just_released(&key));

    input.begin_frame();
    let held_frame = input.frame_snapshot();

    assert!(held_frame.buttons.pressed(&key));
    assert!(!held_frame.buttons.just_pressed(&key));
    assert!(!held_frame.buttons.just_released(&key));

    input.submit_event(InputEvent::ButtonReleased(key.clone()));
    let released_frame = input.frame_snapshot();

    assert!(!released_frame.buttons.pressed(&key));
    assert!(!released_frame.buttons.just_pressed(&key));
    assert!(released_frame.buttons.just_released(&key));

    input.begin_frame();
    let cleared_frame = input.frame_snapshot();

    assert!(!cleared_frame.buttons.pressed(&key));
    assert!(!cleared_frame.buttons.just_pressed(&key));
    assert!(!cleared_frame.buttons.just_released(&key));
}

/// 检查帧快照汇集按钮、光标、运动、滚轮及窗口/拖放通知。
///
/// # 使用时机
///
/// 读取 [crate::input::InputFrameSnapshot]，观察多种事件到下一次 begin_frame 的字段变化。
///
/// # 覆盖范围与限制
///
/// 混合 line/pixel 聚合只描述当前 fixture；不裁决尚待确认的单位契约。
///
/// # 生命周期
///
/// held 和 cursor inside 跨帧保留；motion、滚轮列表及窗口/拖放通知由 begin_frame 清除。
///
/// # 参考
///
/// [crate::input::InputFrameSnapshot] 与 [crate::input::DefaultInputManager]
#[test]
fn input_manager_frame_snapshot_tracks_transitions_and_motion() {
    let input = DefaultInputManager::default();
    input.begin_frame();
    input.submit_event(InputEvent::CursorEntered);
    input.submit_event(InputEvent::ButtonPressed(InputButton::MouseLeft));
    input.submit_event(InputEvent::MouseMotion {
        delta_x: 3.0,
        delta_y: -2.0,
    });
    input.submit_event(InputEvent::WheelScrolled { delta: 2.0 });
    input.submit_event(InputEvent::MouseWheel(MouseWheelEvent::pixels(4.0, 20.0)));
    input.submit_event(InputEvent::WindowStatus(WindowStatusEvent::Moved {
        x: 10,
        y: 20,
    }));
    input.submit_event(InputEvent::WindowStatus(WindowStatusEvent::ThemeChanged(
        WindowTheme::Dark,
    )));
    input.submit_event(InputEvent::WindowStatus(
        WindowStatusEvent::BackendScaleFactorChanged { scale_factor: 2.0 },
    ));
    input.submit_event(InputEvent::WindowStatus(
        WindowStatusEvent::ScaleFactorChanged { scale_factor: 1.5 },
    ));
    input.submit_event(InputEvent::FileDragDrop(FileDragDropEvent::Hovered {
        path: "C:/tmp/asset.png".to_string(),
    }));
    input.submit_event(InputEvent::FileDragDrop(FileDragDropEvent::Dropped {
        path: "C:/tmp/asset.png".to_string(),
    }));
    input.submit_event(InputEvent::FileDragDrop(FileDragDropEvent::Cancelled));

    let frame = input.frame_snapshot();

    assert!(frame.buttons.pressed(&InputButton::MouseLeft));
    assert!(frame.buttons.just_pressed(&InputButton::MouseLeft));
    assert!(frame.cursor_inside_window);
    assert_eq!(frame.mouse_motion_accumulator, [3.0, -2.0]);
    assert_eq!(
        MouseWheelEvent::pixels(4.0, 20.0).vertical_line_delta(),
        20.0 * PIXEL_SCROLL_LINE_DELTA_SCALE
    );
    assert_eq!(
        frame.wheel_accumulator,
        2.0 + 20.0 * PIXEL_SCROLL_LINE_DELTA_SCALE
    );
    assert_eq!(frame.mouse_wheel_accumulator, [4.0, 22.0]);
    assert_eq!(frame.mouse_wheel_unit, MouseScrollUnit::Pixel);
    assert_eq!(
        frame.mouse_wheel_events,
        vec![
            MouseWheelEvent::lines(0.0, 2.0),
            MouseWheelEvent::pixels(4.0, 20.0)
        ]
    );
    assert_eq!(
        frame.window_status_events,
        vec![
            WindowStatusEvent::Moved { x: 10, y: 20 },
            WindowStatusEvent::ThemeChanged(WindowTheme::Dark),
            WindowStatusEvent::BackendScaleFactorChanged { scale_factor: 2.0 },
            WindowStatusEvent::ScaleFactorChanged { scale_factor: 1.5 }
        ]
    );
    assert_eq!(
        frame.file_drag_drop_events,
        vec![
            FileDragDropEvent::Hovered {
                path: "C:/tmp/asset.png".to_string()
            },
            FileDragDropEvent::Dropped {
                path: "C:/tmp/asset.png".to_string()
            },
            FileDragDropEvent::Cancelled
        ]
    );

    input.submit_event(InputEvent::CursorLeft);
    let cursor_left_frame = input.frame_snapshot();

    assert!(!cursor_left_frame.cursor_inside_window);

    input.begin_frame();
    let next_frame = input.frame_snapshot();

    assert!(next_frame.buttons.pressed(&InputButton::MouseLeft));
    assert!(!next_frame.buttons.just_pressed(&InputButton::MouseLeft));
    assert!(!next_frame.cursor_inside_window);
    assert_eq!(next_frame.mouse_motion_accumulator, [0.0, 0.0]);
    assert_eq!(next_frame.wheel_accumulator, 0.0);
    assert_eq!(next_frame.mouse_wheel_accumulator, [0.0, 0.0]);
    assert_eq!(next_frame.mouse_wheel_unit, MouseScrollUnit::Line);
    assert!(next_frame.mouse_wheel_events.is_empty());
    assert!(next_frame.window_status_events.is_empty());
    assert!(next_frame.file_drag_drop_events.is_empty());
}

/// 验证交错提交多个手柄轴时每帧按轴保留一个合并变化记录。
///
/// # 使用时机
///
/// 观察默认缩放结果如何进入 [crate::input::InputFrameSnapshot] transition 列表。
///
/// # 覆盖范围与限制
///
/// 断言首旧值、同帧末新值、交错顺序及下一帧旧值；不逐项保留原始采样。
///
/// # 生命周期
///
/// transition 在 begin_frame 清空；当前轴值跨帧作为后续基线。
///
/// # 参考
///
/// [crate::input::GamepadAxisSettings::scaled_value()] 与 [crate::core::framework::input::InputManager::begin_frame()]
#[test]
fn input_manager_indexes_interleaved_gamepad_axis_transitions_per_frame() {
    let input = DefaultInputManager::default();
    let first_gamepad = GamepadId(1);
    let second_gamepad = GamepadId(2);

    input.submit_event(InputEvent::GamepadConnection(crate::input::GamepadConnectionInfo {
        gamepad: first_gamepad,
        generation: std::num::NonZeroU64::MIN,
        connected: true,
        name: None,
        vendor_id: None,
        product_id: None,
    }));
    input.submit_event(InputEvent::GamepadConnection(crate::input::GamepadConnectionInfo {
        gamepad: second_gamepad,
        generation: std::num::NonZeroU64::MIN,
        connected: true,
        name: None,
        vendor_id: None,
        product_id: None,
    }));
    input.begin_frame();
    input.submit_event(InputEvent::GamepadAxis {
        generation: std::num::NonZeroU64::MIN,
        gamepad: first_gamepad,
        axis: GamepadAxis::LeftStickX,
        value: 0.25,
    });
    input.submit_event(InputEvent::GamepadAxis {
        generation: std::num::NonZeroU64::MIN,
        gamepad: second_gamepad,
        axis: GamepadAxis::RightStickY,
        value: -0.5,
    });
    input.submit_event(InputEvent::GamepadAxis {
        generation: std::num::NonZeroU64::MIN,
        gamepad: first_gamepad,
        axis: GamepadAxis::LeftStickX,
        value: 0.75,
    });

    let frame = input.frame_snapshot();
    assert_eq!(frame.gamepad_axis_transitions.len(), 2);
    assert_eq!(frame.gamepad_axis_transitions[0].gamepad, first_gamepad);
    assert_eq!(
        frame.gamepad_axis_transitions[0].axis,
        GamepadAxis::LeftStickX
    );
    assert_eq!(frame.gamepad_axis_transitions[0].previous_value, 0.0);
    assert_eq!(
        frame.gamepad_axis_transitions[0].value,
        GamepadAxisSettings::default().scaled_value(0.75)
    );
    assert_eq!(frame.gamepad_axis_transitions[1].gamepad, second_gamepad);
    assert_eq!(
        frame.gamepad_axis_transitions[1].axis,
        GamepadAxis::RightStickY
    );
    assert_eq!(frame.gamepad_axis_transitions[1].previous_value, 0.0);
    assert_eq!(
        frame.gamepad_axis_transitions[1].value,
        GamepadAxisSettings::default().scaled_value(-0.5)
    );

    input.begin_frame();
    assert!(input.frame_snapshot().gamepad_axis_transitions.is_empty());

    input.submit_event(InputEvent::GamepadAxis {
        generation: std::num::NonZeroU64::MIN,
        gamepad: first_gamepad,
        axis: GamepadAxis::LeftStickX,
        value: -0.25,
    });
    let next_frame = input.frame_snapshot();
    assert_eq!(next_frame.gamepad_axis_transitions.len(), 1);
    assert_eq!(
        next_frame.gamepad_axis_transitions[0].previous_value,
        GamepadAxisSettings::default().scaled_value(0.75)
    );
    assert_eq!(
        next_frame.gamepad_axis_transitions[0].value,
        GamepadAxisSettings::default().scaled_value(-0.25)
    );
}

/// 验证 FocusLost 释放活动控件并清除触点/模拟值，同时保留已连接手柄。
///
/// # 使用时机
///
/// 观察 [crate::core::framework::input::InputEvent] 焦点丢失后读取帧快照。
///
/// # 覆盖范围与限制
///
/// 覆盖键鼠/手柄释放、触点/轴清空、轴归零和 IME 清理；连接仍保留。
///
/// # 生命周期
///
/// 先 begin_frame，再加入 motion/wheel 和 FocusLost；焦点丢失不等于设备断开。
///
/// # 参考
///
/// [crate::input::DefaultInputManager]、[crate::input::InputFrameSnapshot] 与 [crate::input::ButtonInputState::just_released()]
#[test]
fn focus_loss_releases_active_input_without_disconnect() {
    let input = DefaultInputManager::default();
    let gamepad = GamepadId(7);
    input.submit_event(InputEvent::KeyboardInput {
        key_code: 16,
        logical_key: Some("Shift".to_string()),
        text: None,
        pressed: true,
        repeat: false,
    });
    input.submit_event(InputEvent::ButtonPressed(InputButton::MouseLeft));
    input.submit_event(InputEvent::GamepadConnection(GamepadConnectionInfo {
        generation: std::num::NonZeroU64::MIN,
        gamepad,
        connected: true,
        name: None,
        vendor_id: None,
        product_id: None,
    }));
    input.submit_event(InputEvent::GamepadButton {
        generation: std::num::NonZeroU64::MIN,
        gamepad,
        button: GamepadButton::South,
        value: 1.0,
        pressed: true,
    });
    input.submit_event(InputEvent::GamepadAxis {
        generation: std::num::NonZeroU64::MIN,
        gamepad,
        axis: GamepadAxis::LeftStickX,
        value: 1.0,
    });
    input.submit_event(InputEvent::Touch {
        id: 42,
        phase: TouchPhase::Started,
        x: 12.0,
        y: 24.0,
    });
    input.submit_event(InputEvent::Ime(ImeEvent::Enabled));
    input.submit_event(InputEvent::Ime(ImeEvent::Preedit(ImePreedit::new(
        "composing",
        Some(ImeCursorRange::new(0, 9)),
    ))));

    input.begin_frame();
    input.submit_event(InputEvent::MouseMotion {
        delta_x: 4.0,
        delta_y: -2.0,
    });
    input.submit_event(InputEvent::WheelScrolled { delta: 1.5 });
    input.submit_event(InputEvent::FocusLost);
    let frame = input.frame_snapshot();

    assert!(!frame.buttons.pressed(&InputButton::KeyCode(16)));
    assert!(!frame
        .buttons
        .pressed(&InputButton::Key("Shift".to_string())));
    assert!(frame.buttons.just_released(&InputButton::KeyCode(16)));
    assert!(frame
        .buttons
        .just_released(&InputButton::Key("Shift".to_string())));
    assert!(!frame.buttons.pressed(&InputButton::MouseLeft));
    assert!(frame.buttons.just_released(&InputButton::MouseLeft));
    assert!(!frame.buttons.pressed(&InputButton::Gamepad {
        gamepad,
        button: GamepadButton::South,
    }));
    assert!(frame.buttons.just_released(&InputButton::Gamepad {
        gamepad,
        button: GamepadButton::South,
    }));
    assert!(frame.active_touches.is_empty());
    assert_eq!(frame.connected_gamepads, vec![gamepad]);
    assert!(frame.gamepad_axes.is_empty());
    assert!(frame.gamepad_button_values.is_empty());
    assert!(frame.gamepad_axis_transitions.iter().any(|transition| {
        transition.gamepad == gamepad
            && transition.axis == GamepadAxis::LeftStickX
            && transition.previous_value == 1.0
            && transition.value == 0.0
    }));
    assert_eq!(frame.mouse_motion_accumulator, [0.0, 0.0]);
    assert_eq!(frame.wheel_accumulator, 0.0);
    assert!(frame.mouse_wheel_events.is_empty());
    assert!(!frame.ime_enabled);
    assert_eq!(frame.ime_preedit, None);
}

/// 验证动作求值器在 FocusLost 清除键盘和手柄轴 held 后报告动作停用。
///
/// # 使用时机
///
/// 用测试内动作映射和 [crate::input::InputActionEvaluator::evaluate()] 比较焦点前后快照。
///
/// # 覆盖范围与限制
///
/// 只覆盖 jump 和 move_x；不执行动态会话、生产动作调度或宿主设备输入。
///
/// # 生命周期
///
/// 先 begin_frame 清边沿保留 held，再由同一 evaluator 对新快照求值。
///
/// # 参考
///
/// [crate::input::InputActionEvaluator::evaluate()] 与 [crate::input::InputFrameSnapshot]
#[test]
fn focus_loss_deactivates_held_button_and_axis_actions() {
    use crate::input::{
        InputAction, InputActionEvaluator, InputActionMap, InputAxisBinding, InputBinding,
    };

    let gamepad = GamepadId(7);
    let mut actions = InputActionMap::new();
    actions.add_action(InputAction::new("gameplay.jump"));
    actions.add_action(InputAction::new("gameplay.move_x"));
    actions.bind(InputBinding::button(
        "gameplay.jump",
        InputButton::KeyCode(32),
    ));
    actions.bind(InputBinding::axis(
        "gameplay.move_x",
        InputAxisBinding::new(gamepad, GamepadAxis::LeftStickX),
    ));
    let evaluator = InputActionEvaluator::new(actions);
    let input = DefaultInputManager::default();

    input.submit_event(InputEvent::KeyboardInput {
        key_code: 32,
        logical_key: Some("Space".to_string()),
        text: None,
        pressed: true,
        repeat: false,
    });
    input.submit_event(InputEvent::GamepadConnection(crate::input::GamepadConnectionInfo {
        gamepad: gamepad,
        generation: std::num::NonZeroU64::MIN,
        connected: true,
        name: None,
        vendor_id: None,
        product_id: None,
    }));
    input.submit_event(InputEvent::GamepadAxis {
        generation: std::num::NonZeroU64::MIN,
        gamepad,
        axis: GamepadAxis::LeftStickX,
        value: 1.0,
    });
    input.begin_frame();

    let held = evaluator.evaluate(&input.frame_snapshot());
    assert!(held.pressed("gameplay.jump"));
    assert!(held.pressed("gameplay.move_x"));

    input.submit_event(InputEvent::FocusLost);

    let released = evaluator.evaluate(&input.frame_snapshot());
    assert!(!released.pressed("gameplay.jump"));
    assert!(released.just_deactivated("gameplay.jump"));
    assert!(!released.pressed("gameplay.move_x"));
    assert!(released.just_deactivated("gameplay.move_x"));
}

/// 验证同一帧按键后立即 FocusLost 时动作不会被激活。
///
/// # 使用时机
///
/// 把 [crate::core::framework::input::InputEvent] 的按下和焦点顺序交给测试内求值器。
///
/// # 覆盖范围与限制
///
/// 断言 KeyCode 69 同时有两个边沿，interact 不 pressed/activated 且 deactivated。
///
/// # 生命周期
///
/// 事件之间无 begin_frame，因此两个边沿共存于一个帧快照。
///
/// # 参考
///
/// [crate::input::InputActionEvaluator::evaluate()] 与 [crate::input::InputFrameSnapshot]
#[test]
fn focus_loss_after_same_frame_press_does_not_activate_an_action() {
    use crate::input::{InputAction, InputActionEvaluator, InputActionMap, InputBinding};

    let mut actions = InputActionMap::new();
    actions.add_action(InputAction::new("gameplay.interact"));
    actions.bind(InputBinding::button(
        "gameplay.interact",
        InputButton::KeyCode(69),
    ));
    let evaluator = InputActionEvaluator::new(actions);
    let input = DefaultInputManager::default();

    input.submit_event(InputEvent::KeyboardInput {
        key_code: 69,
        logical_key: Some("E".to_string()),
        text: None,
        pressed: true,
        repeat: false,
    });
    input.submit_event(InputEvent::FocusLost);

    let frame = input.frame_snapshot();
    assert!(frame.buttons.just_pressed(&InputButton::KeyCode(69)));
    assert!(frame.buttons.just_released(&InputButton::KeyCode(69)));

    let actions = evaluator.evaluate(&frame);
    assert!(!actions.pressed("gameplay.interact"));
    assert!(!actions.just_activated("gameplay.interact"));
    assert!(actions.just_deactivated("gameplay.interact"));
}

/// 验证 FocusLost 将待处理 IME 与光标捕获请求替换成清理请求。
///
/// # 使用时机
///
/// 观察焦点清理后的 request 帧视图及 [crate::core::framework::input::InputManager::drain_ime_host_requests()]、drain_cursor_host_requests()。
///
/// # 覆盖范围与限制
///
/// 两次 FocusLost 后每类只剩一条清理请求；不执行宿主 API。
///
/// # 生命周期
///
/// 没有 begin_frame；focus-loss 路径重置队列后分别 drain。
///
/// # 参考
///
/// [crate::input::InputFrameSnapshot] 与 [crate::core::framework::input::InputManager::drain_cursor_host_requests()]
#[test]
fn focus_loss_cancels_pending_ime_and_cursor_capture_requests() {
    let input = DefaultInputManager::default();
    input.submit_event(InputEvent::ImeHostRequest(ImeHostRequest::Enable));
    input.submit_event(InputEvent::CursorHostRequest(
        CursorHostRequest::set_grab_mode(CursorGrabMode::Locked),
    ));

    input.submit_event(InputEvent::FocusLost);
    input.submit_event(InputEvent::FocusLost);

    let frame = input.frame_snapshot();
    assert_eq!(frame.ime_host_requests, vec![ImeHostRequest::Disable]);
    assert_eq!(
        frame.cursor_host_requests,
        vec![CursorHostRequest::set_grab_mode(CursorGrabMode::None)]
    );
    assert_eq!(
        input.drain_ime_host_requests(),
        vec![ImeHostRequest::Disable]
    );
    assert_eq!(
        input.drain_cursor_host_requests(),
        vec![CursorHostRequest::set_grab_mode(CursorGrabMode::None)]
    );
}

/// 验证 IME 状态、帧内 commit/delete 通知和 host request 排空具有不同生命周期。
///
/// # 使用时机
///
/// 提交 [crate::core::framework::input::InputEvent] IME 事件后读取 [crate::input::InputFrameSnapshot]。
///
/// # 覆盖范围与限制
///
/// 断言 enabled/preedit、commit/delete 和 request 内容顺序；不调用平台输入法。
///
/// # 生命周期
///
/// enabled 跨 begin_frame 保留；commit/delete 下一帧清除；request view 与 drain 队列有不同边界。
///
/// # 参考
///
/// [crate::core::framework::input::InputManager::begin_frame()] 与 [crate::core::framework::input::InputManager::drain_ime_host_requests()]
#[test]
fn input_manager_tracks_ime_preedit_and_frame_commits() {
    let input = DefaultInputManager::default();

    input.submit_event(InputEvent::Ime(ImeEvent::Enabled));
    input.submit_event(InputEvent::Ime(ImeEvent::Preedit(ImePreedit::new(
        "ni",
        Some(ImeCursorRange::new(0, 2)),
    ))));
    input.submit_event(InputEvent::Ime(ImeEvent::Commit("你".to_string())));
    input.submit_event(InputEvent::Ime(ImeEvent::DeleteSurrounding(
        ImeDeleteSurrounding::new(1, 2),
    )));
    input.submit_event(InputEvent::ImeHostRequest(ImeHostRequest::Enable));
    input.submit_event(InputEvent::ImeHostRequest(ImeHostRequest::SetCursorArea(
        ImeCursorArea::new(16.0, 24.0, 1.0, 18.0),
    )));
    input.submit_event(InputEvent::ImeHostRequest(
        ImeHostRequest::SetSurroundingText(ImeSurroundingText::new("hello", 5, 0)),
    ));

    let frame = input.frame_snapshot();

    assert!(frame.ime_enabled);
    assert_eq!(frame.ime_preedit, None);
    assert_eq!(frame.ime_commits, vec!["你".to_string()]);
    assert_eq!(
        frame.ime_delete_surrounding,
        vec![ImeDeleteSurrounding::new(1, 2)]
    );
    assert_eq!(
        frame.ime_host_requests,
        vec![
            ImeHostRequest::Enable,
            ImeHostRequest::SetCursorArea(ImeCursorArea::new(16.0, 24.0, 1.0, 18.0)),
            ImeHostRequest::SetSurroundingText(ImeSurroundingText::new("hello", 5, 0))
        ]
    );

    let drained_host_requests = input.drain_ime_host_requests();
    assert_eq!(
        drained_host_requests,
        vec![
            ImeHostRequest::Enable,
            ImeHostRequest::SetCursorArea(ImeCursorArea::new(16.0, 24.0, 1.0, 18.0)),
            ImeHostRequest::SetSurroundingText(ImeSurroundingText::new("hello", 5, 0))
        ]
    );
    assert!(input.frame_snapshot().ime_host_requests.is_empty());

    input.begin_frame();
    let next_frame = input.frame_snapshot();

    assert!(next_frame.ime_enabled);
    assert!(next_frame.ime_commits.is_empty());
    assert!(next_frame.ime_delete_surrounding.is_empty());
    assert!(next_frame.ime_host_requests.is_empty());

    input.submit_event(InputEvent::ImeHostRequest(ImeHostRequest::Enable));
    input.begin_frame();
    assert!(input.frame_snapshot().ime_host_requests.is_empty());
    assert_eq!(
        input.drain_ime_host_requests(),
        vec![ImeHostRequest::Enable]
    );
    assert!(input.drain_ime_host_requests().is_empty());

    input.submit_event(InputEvent::Ime(ImeEvent::Preedit(ImePreedit::new(
        "hao", None,
    ))));
    let preedit_frame = input.frame_snapshot();

    assert_eq!(
        preedit_frame.ime_preedit,
        Some(ImePreedit::new("hao", None))
    );

    input.submit_event(InputEvent::Ime(ImeEvent::Disabled));
    let disabled_frame = input.frame_snapshot();

    assert!(!disabled_frame.ime_enabled);
    assert_eq!(disabled_frame.ime_preedit, None);
}

#[test]
fn gamepad_same_frame_reconnect_preserves_raw_generations_and_routing_axis_aggregates() {
    use crate::core::framework::input::InputEventAdmission;
    use crate::input::{GamepadAxisTransition, InputAction, InputActionEvaluator, InputActionMap,
        InputAxisBinding, InputBinding, InputEventRecordingConfig};
    let input = DefaultInputManager::default();
    let first = GamepadId(201);
    let other = GamepadId(202);
    let axis = GamepadAxis::LeftStickX;
    let lifecycle = |gamepad, generation, connected| {
        InputEvent::GamepadConnection(GamepadConnectionInfo {
            gamepad, generation: std::num::NonZeroU64::new(generation).unwrap(), connected,
            name: None, vendor_id: None, product_id: None,
        })
    };
    let sample = |gamepad, generation, value| InputEvent::GamepadAxis {
        gamepad, generation: std::num::NonZeroU64::new(generation).unwrap(), axis, value,
    };
    input.set_event_recording_config(InputEventRecordingConfig::enabled(64));
    input.submit_event(lifecycle(first, 1, true));
    input.submit_event(lifecycle(other, 1, true));
    input.begin_frame();
    input.drain_event_records();
    let events = vec![
        sample(first, 1, 0.25), sample(other, 1, 0.5),
        lifecycle(first, 1, false), lifecycle(first, 2, true),
        sample(first, 2, -0.75), sample(other, 1, 0.8),
    ];
    for event in &events {
        assert_eq!(input.submit_event_with_admission(event.clone()), InputEventAdmission::Accepted);
    }
    let frame = input.frame_snapshot();
    let scale = |value| GamepadAxisSettings::default().scaled_value(value);
    assert_eq!(frame.gamepad_axis_transitions, vec![
        GamepadAxisTransition { gamepad: first, axis, previous_value: 0.0, value: scale(0.25) },
        GamepadAxisTransition { gamepad: other, axis, previous_value: 0.0, value: scale(0.8) },
        GamepadAxisTransition { gamepad: first, axis, previous_value: scale(0.25), value: 0.0 },
        GamepadAxisTransition { gamepad: first, axis, previous_value: 0.0, value: scale(-0.75) },
    ]);
    assert_eq!(frame.gamepad_axes.iter().find(|state| state.gamepad == first).unwrap().value, scale(-0.75));
    // Actions retain their established routing-ID/last-source-item aggregate contract.
    let mut map = InputActionMap::new();
    map.add_action(InputAction::new("gamepad.reconnected"));
    map.bind(InputBinding::axis("gamepad.reconnected", InputAxisBinding::new(first, axis)));
    let evaluator = InputActionEvaluator::new(map);
    let evaluated = evaluator.evaluate(&frame);
    assert_eq!(evaluated.value("gamepad.reconnected"), scale(-0.75));
    assert!(evaluated.pressed("gamepad.reconnected"));
    assert!(evaluated.just_activated("gamepad.reconnected"));
    assert!(!evaluated.just_deactivated("gamepad.reconnected"));

    assert!(matches!(input.submit_event_with_admission(sample(first, 1, 1.0)), InputEventAdmission::Rejected(_)));
    assert_eq!(input.frame_snapshot().gamepad_axis_transitions, frame.gamepad_axis_transitions);
    // Raw frame events and recorded events, rather than the lossy aggregate DTO, carry exact incarnation identity.
    assert_eq!(input.drain_events(), events);
    assert_eq!(input.drain_event_records().into_iter().map(|record| record.event).collect::<Vec<_>>(), events);
}
