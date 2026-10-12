//! 检查触点、手柄状态及多类输入快照的投影。
//!
//! # 使用时机与责任
//! 在测试构建中由输入管理器测试模块挂载；直接提交协议事件并观察帧视图，验证触点移除、手柄过滤和断连，以及选定输入字段的摘要。这里没有真实设备采样或宿主动作执行。
//!
//! # 生命周期与限制
//! 每个情景拥有独立管理器；快照和摘要字符串不借用其后续状态。摘要辅助函数按类别组织选定字段和固定精度，原始提交顺序另通过录制记录观察，不由摘要恢复。
//!
//! # 参考
//! 参见 [`crate::input::InputEvent`]、[`crate::input::InputFrameSnapshot`] 与 [`crate::input::DefaultInputManager`]。

use super::*;

// 从触摸和手柄输入到帧快照的整链测试，兼顾断开清理与模拟输入阈值。
/// 验证 touch 与 gamepad 事件共同更新触点、连接、按钮和轴幅值。
///
/// # 使用时机
///
/// 从合成 [crate::core::framework::input::InputEvent] 提交后读取 [crate::input::InputFrameSnapshot]。
///
/// # 覆盖范围与限制
///
/// 检查 Started/Ended、连接/断开、South 按住和默认映射；不检查边沿、断开 transition 或自定义设置。
///
/// # 生命周期
///
/// 触点随 Ended 移除，轴/按钮值随 disconnect 清理；没有显式 begin_frame。
///
/// # 返回与失败
///
/// 各断言成立时正常返回 `()`；断言失败通过 panic 交给测试框架。返回值不携带生产输入、设备执行结果或持久资源。
///
/// # 参考
///
/// [crate::input::GamepadAxisSettings::scaled_value()] 与 [crate::input::GamepadButtonAxisSettings::scaled_value()]
#[test]
fn input_manager_tracks_touch_and_gamepad_state() {
    let input = DefaultInputManager::default();
    let gamepad = GamepadId(7);

    input.submit_event(InputEvent::Touch {
        id: 42,
        phase: TouchPhase::Started,
        x: 11.0,
        y: 22.0,
    });
    input.submit_event(InputEvent::GamepadConnection(GamepadConnectionInfo {
        gamepad,
        connected: true,
        name: Some("Pad".to_string()),
        vendor_id: Some(1),
        product_id: Some(2),
    }));
    input.submit_event(InputEvent::GamepadButton {
        gamepad,
        button: GamepadButton::South,
        value: 1.0,
        pressed: true,
    });
    input.submit_event(InputEvent::GamepadAxis {
        gamepad,
        axis: GamepadAxis::LeftStickX,
        value: 0.5,
    });

    let frame = input.frame_snapshot();

    assert_eq!(frame.active_touches.len(), 1);
    assert_eq!(frame.active_touches[0].position, [11.0, 22.0]);
    assert_eq!(frame.connected_gamepads, vec![gamepad]);
    assert!(frame.buttons.pressed(&InputButton::Gamepad {
        gamepad,
        button: GamepadButton::South
    }));
    assert_eq!(frame.gamepad_axes.len(), 1);
    assert_eq!(
        frame.gamepad_axes[0].value,
        GamepadAxisSettings::default().scaled_value(0.5)
    );
    assert_eq!(frame.gamepad_button_values.len(), 1);
    assert_eq!(
        frame.gamepad_button_values[0].value,
        GamepadButtonAxisSettings::default().scaled_value(1.0)
    );

    input.submit_event(InputEvent::Touch {
        id: 42,
        phase: TouchPhase::Ended,
        x: 11.0,
        y: 22.0,
    });
    input.submit_event(InputEvent::GamepadConnection(GamepadConnectionInfo {
        gamepad,
        connected: false,
        name: None,
        vendor_id: None,
        product_id: None,
    }));

    let cleared = input.frame_snapshot();

    assert!(cleared.active_touches.is_empty());
    assert!(cleared.connected_gamepads.is_empty());
    assert!(cleared.gamepad_axes.is_empty());
    assert!(cleared.gamepad_button_values.is_empty());
    assert!(!cleared.buttons.pressed(&InputButton::Gamepad {
        gamepad,
        button: GamepadButton::South
    }));
}

/// 用测试本地 formatter 检查多类帧摘要，并单独检查原始记录序号。
///
/// # 使用时机
///
/// 将 [crate::input::InputFrameSnapshot] 传给本文件 formatter，再 drain records。
///
/// # 覆盖范围与限制
///
/// 字符串按字段类别组织，不表示原始提交时序；records 断言才检查序号。
///
/// # 生命周期
///
/// 录制在 begin_frame 后接收合成事件；formatter 只借用快照。
///
/// # 返回与失败
///
/// 各断言成立时正常返回 `()`；断言失败通过 panic 交给测试框架。返回值不携带生产输入、设备执行结果或持久资源。
///
/// # 参考
///
/// [crate::core::framework::input::InputManager::drain_event_records()] 与 [crate::input::InputFrameSnapshot]
#[test]
fn input_manager_event_log_harness_covers_window_keyboard_mouse_touch_and_gamepad() {
    let input = DefaultInputManager::default();
    input.set_event_recording_config(crate::input::InputEventRecordingConfig::enabled(32));
    let gamepad = GamepadId(9);

    input.begin_frame();
    input.submit_event(InputEvent::WindowStatus(WindowStatusEvent::Moved {
        x: 640,
        y: 480,
    }));
    input.submit_event(InputEvent::WindowStatus(WindowStatusEvent::CloseRequested));
    input.submit_event(InputEvent::KeyboardInput {
        key_code: 65,
        logical_key: Some("KeyA".to_string()),
        text: Some("a".to_string()),
        pressed: true,
        repeat: false,
    });
    input.submit_event(InputEvent::CursorEntered);
    input.submit_event(InputEvent::CursorMoved { x: 320.0, y: 180.0 });
    input.submit_event(InputEvent::MouseMotion {
        delta_x: 2.5,
        delta_y: -1.0,
    });
    input.submit_event(InputEvent::MouseWheel(MouseWheelEvent::lines(0.0, -1.0)));
    input.submit_event(InputEvent::ButtonPressed(InputButton::MouseLeft));
    input.submit_event(InputEvent::Touch {
        id: 12,
        phase: TouchPhase::Started,
        x: 10.0,
        y: 20.0,
    });
    input.submit_event(InputEvent::GamepadConnection(GamepadConnectionInfo {
        gamepad,
        connected: true,
        name: Some("Harness Pad".to_string()),
        vendor_id: Some(100),
        product_id: Some(200),
    }));
    input.submit_event(InputEvent::GamepadButton {
        gamepad,
        button: GamepadButton::South,
        value: 1.0,
        pressed: true,
    });
    input.submit_event(InputEvent::GamepadAxis {
        gamepad,
        axis: GamepadAxis::LeftStickX,
        value: 0.5,
    });

    let frame = input.frame_snapshot();

    assert_eq!(
        runtime_input_event_log_harness(&frame),
        vec![
            "window:moved(640,480)",
            "window:close_requested",
            "keyboard:key_code:65:pressed",
            "keyboard:key:KeyA:pressed",
            "mouse:cursor(320.0,180.0):inside",
            "mouse:motion(2.5,-1.0)",
            "mouse:wheel:line(0.0,-1.0)",
            "mouse:button:left:pressed",
            "touch:12:Started(10.0,20.0)",
            "gamepad:9:connected",
            "gamepad:9:South:pressed",
            "gamepad:9:LeftStickX=0.47",
        ]
    );
    assert_eq!(
        input
            .drain_event_records()
            .iter()
            .map(|record| record.sequence)
            .collect::<Vec<_>>(),
        (1..=12).collect::<Vec<_>>()
    );
}

/// 把单份输入帧快照选定字段按固定类别格式化为测试字符串。
///
/// # 参数
///
/// frame 是只读的 [crate::input::InputFrameSnapshot] 借用。
///
/// # 返回
///
/// 返回 Vec<String>，组合窗口、键鼠、触摸和手柄摘要；它不是生产日志。
///
/// # 使用时机与限制
///
/// 仅由本文件的多类输入快照情景调用。类别顺序固定为窗口、键盘、光标与位移、滚轮、鼠标按钮、触点、手柄连接、手柄按钮和轴；不保证跨类别原始提交顺序，也不覆盖全部快照字段。坐标及增量按一位小数、轴幅值按两位小数格式化，适合本夹具的摘要断言。
///
///
///
/// # 生命周期
///
/// 只读的 [crate::input::InputFrameSnapshot] 借用。
///
/// 返回的向量与其中字符串各自拥有内容；函数不修改快照，返回后不保留 frame 借用，之后修改来源管理器不改变这些摘要。
///
/// # 参考
///
/// [crate::input::InputFrameSnapshot]
fn runtime_input_event_log_harness(frame: &InputFrameSnapshot) -> Vec<String> {
    let mut log = Vec::new();

    for event in &frame.window_status_events {
        match event {
            WindowStatusEvent::Moved { x, y } => log.push(format!("window:moved({x},{y})")),
            WindowStatusEvent::CloseRequested => log.push("window:close_requested".to_string()),
            other => log.push(format!("window:{other:?}")),
        }
    }

    for button in frame.buttons.just_pressed_inputs() {
        match button {
            InputButton::KeyCode(code) => log.push(format!("keyboard:key_code:{code}:pressed")),
            InputButton::Key(key) => log.push(format!("keyboard:key:{key}:pressed")),
            _ => {}
        }
    }

    if frame.cursor_inside_window || frame.cursor_position != [0.0, 0.0] {
        let boundary = if frame.cursor_inside_window {
            "inside"
        } else {
            "outside"
        };
        log.push(format!(
            "mouse:cursor({:.1},{:.1}):{boundary}",
            frame.cursor_position[0], frame.cursor_position[1]
        ));
    }
    if frame.mouse_motion_accumulator != [0.0, 0.0] {
        log.push(format!(
            "mouse:motion({:.1},{:.1})",
            frame.mouse_motion_accumulator[0], frame.mouse_motion_accumulator[1]
        ));
    }
    for wheel in &frame.mouse_wheel_events {
        let unit = match wheel.unit {
            MouseScrollUnit::Line => "line",
            MouseScrollUnit::Pixel => "pixel",
        };
        log.push(format!("mouse:wheel:{unit}({:.1},{:.1})", wheel.x, wheel.y));
    }
    for button in frame.buttons.just_pressed_inputs() {
        if let Some(label) = mouse_button_label(&button) {
            log.push(format!("mouse:button:{label}:pressed"));
        }
    }

    for touch in &frame.active_touches {
        log.push(format!(
            "touch:{}:{:?}({:.1},{:.1})",
            touch.id, touch.phase, touch.position[0], touch.position[1]
        ));
    }

    for gamepad in &frame.connected_gamepads {
        log.push(format!("gamepad:{}:connected", gamepad.0));
    }
    for button in frame.buttons.just_pressed_inputs() {
        if let InputButton::Gamepad { gamepad, button } = button {
            log.push(format!("gamepad:{}:{button:?}:pressed", gamepad.0));
        }
    }
    for axis in &frame.gamepad_axes {
        log.push(format!(
            "gamepad:{}:{:?}={:.2}",
            axis.gamepad.0, axis.axis, axis.value
        ));
    }

    log
}

/// 将鼠标按钮变体转换为 harness 稳定标签。
///
/// # 参数
///
/// button 是只读的 [crate::core::framework::input::InputButton] 借用。
///
/// # 返回
///
/// 鼠标按钮和 MouseOther 返回 Some(String)，其他变体返回 None。
///
/// # 使用时机与限制
///
/// 仅供上面的快照摘要辅助函数筛选鼠标按钮。已知按钮产生固定标签，MouseOther 保留编号；键盘和手柄身份返回 None，由调用者跳过，不能据此推导它们是否按下。
///
///
///
/// # 生命周期
///
/// 只读的 [crate::core::framework::input::InputButton] 借用。
///
/// 返回类型为 `Option<String>`；Some 内的 String 独立拥有标签；函数不修改按钮，返回后不保留 button 借用。None 仅表示不是鼠标按钮变体。
///
/// # 参考
///
/// [crate::core::framework::input::InputButton]
fn mouse_button_label(button: &InputButton) -> Option<String> {
    match button {
        InputButton::MouseLeft => Some("left".to_string()),
        InputButton::MouseRight => Some("right".to_string()),
        InputButton::MouseMiddle => Some("middle".to_string()),
        InputButton::MouseBack => Some("back".to_string()),
        InputButton::MouseForward => Some("forward".to_string()),
        InputButton::MouseOther(code) => Some(format!("other:{code}")),
        _ => None,
    }
}

/// 验证默认管理器先处理模拟按钮幅值，再结合按住状态表现双阈值滞回。
///
/// # 使用时机
///
/// 提交 sample 后用 [crate::input::GamepadButtonAxisSettings::scaled_value()] 解释快照幅值并观察边沿。
///
/// # 覆盖范围与限制
///
/// host pressed 始终 true；以 .70/.80/新帧 .70/.60 观察未按下、按下、保持和释放。
///
/// # 生命周期
///
/// begin_frame 清边沿但保留 held；按钮变化基线跨帧使用。
///
/// # 返回与失败
///
/// 各断言成立时正常返回 `()`；断言失败通过 panic 交给测试框架。返回值不携带生产输入、设备执行结果或持久资源。
///
/// # 参考
///
/// [crate::input::GamepadButtonSettings::transition_for_value()] 与 [crate::input::GamepadButtonAxisSettings::process_value()]
#[test]
fn gamepad_button_values_use_runtime_thresholds_and_hysteresis() {
    let input = DefaultInputManager::default();
    let gamepad = GamepadId(3);
    let button = GamepadButton::South;
    let input_button = InputButton::Gamepad { gamepad, button };

    input.submit_event(InputEvent::GamepadButton {
        gamepad,
        button,
        value: 0.70,
        pressed: true,
    });
    let below_press = input.frame_snapshot();
    assert!(!below_press.buttons.pressed(&input_button));
    assert_eq!(
        below_press.gamepad_button_values[0].value,
        GamepadButtonAxisSettings::default().scaled_value(0.70)
    );

    input.submit_event(InputEvent::GamepadButton {
        gamepad,
        button,
        value: 0.80,
        pressed: true,
    });
    let pressed = input.frame_snapshot();
    assert!(pressed.buttons.pressed(&input_button));
    assert!(pressed.buttons.just_pressed(&input_button));

    input.begin_frame();
    input.submit_event(InputEvent::GamepadButton {
        gamepad,
        button,
        value: 0.70,
        pressed: true,
    });
    let held_by_hysteresis = input.frame_snapshot();
    assert!(held_by_hysteresis.buttons.pressed(&input_button));
    assert!(!held_by_hysteresis.buttons.just_pressed(&input_button));

    input.submit_event(InputEvent::GamepadButton {
        gamepad,
        button,
        value: 0.60,
        pressed: true,
    });
    let released = input.frame_snapshot();
    assert!(!released.buttons.pressed(&input_button));
    assert!(released.buttons.just_released(&input_button));
}

/// 验证默认轴设置的死区/活区映射、变化筛选和输入钳制。
///
/// # 使用时机
///
/// 以 [crate::input::GamepadAxisSettings::scaled_value()] 得到期望映射，从默认管理器快照观察已接受状态。
///
/// # 覆盖范围与限制
///
/// 断言 .03/.04 零、.50 映射、.505 保持、-1.25 到 -1；不覆盖非有限或自定义异常设置。
///
/// # 生命周期
///
/// 变化筛选以最后接受值为基线；过滤值不替换基线，轴状态跨帧存在。
///
/// # 返回与失败
///
/// 各断言成立时正常返回 `()`；断言失败通过 panic 交给测试框架。返回值不携带生产输入、设备执行结果或持久资源。
///
/// # 参考
///
/// [crate::input::GamepadAxisSettings::process_value()]
#[test]
fn gamepad_axis_values_use_deadzone_livezone_and_change_threshold() {
    let input = DefaultInputManager::default();
    let gamepad = GamepadId(5);
    let axis = GamepadAxis::LeftStickX;

    input.submit_event(InputEvent::GamepadAxis {
        gamepad,
        axis,
        value: 0.03,
    });
    let deadzone = input.frame_snapshot();
    assert_eq!(deadzone.gamepad_axes.len(), 1);
    assert_eq!(deadzone.gamepad_axes[0].value, 0.0);

    input.submit_event(InputEvent::GamepadAxis {
        gamepad,
        axis,
        value: 0.04,
    });
    let unchanged = input.frame_snapshot();
    assert_eq!(unchanged.gamepad_axes.len(), 1);
    assert_eq!(unchanged.gamepad_axes[0].value, 0.0);

    input.submit_event(InputEvent::GamepadAxis {
        gamepad,
        axis,
        value: 0.50,
    });
    let moved = input.frame_snapshot();
    assert_eq!(
        moved.gamepad_axes[0].value,
        GamepadAxisSettings::default().scaled_value(0.50)
    );

    input.submit_event(InputEvent::GamepadAxis {
        gamepad,
        axis,
        value: 0.505,
    });
    let filtered = input.frame_snapshot();
    assert_eq!(filtered.gamepad_axes[0].value, moved.gamepad_axes[0].value);

    input.submit_event(InputEvent::GamepadAxis {
        gamepad,
        axis,
        value: -1.25,
    });
    let clamped = input.frame_snapshot();
    assert_eq!(clamped.gamepad_axes[0].value, -1.0);
}
