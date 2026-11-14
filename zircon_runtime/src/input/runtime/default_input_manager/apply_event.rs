//! Apply one admitted input event to persistent state and the frame window.
use crate::core::framework::input::{
    GamepadInputRejection, InputEventAdmission, QueuedGamepadRumbleRequest,
};
use crate::input::{
    CursorHostRequest, GamepadAxisSettings, GamepadButtonAxisSettings, GamepadButtonSettings,
    ImeEvent, ImeHostRequest, InputButton, InputEvent, MouseScrollUnit, MouseWheelEvent,
    TouchPhase, TouchPoint,
};
use super::super::InputState;

pub(super) fn apply_input_event(state: &mut InputState, event: InputEvent) {
    match &event {
        InputEvent::CursorMoved { x, y } => {
            state.cursor_position = [*x, *y];
        }
        InputEvent::CursorEntered => {
            state.cursor_inside_window = true;
        }
        InputEvent::CursorLeft => {
            state.cursor_inside_window = false;
        }
        InputEvent::MouseMotion { delta_x, delta_y } => {
            state.mouse_motion_accumulator[0] += *delta_x;
            state.mouse_motion_accumulator[1] += *delta_y;
        }
        InputEvent::ButtonPressed(button) => {
            state.buttons.press(button.clone());
        }
        InputEvent::ButtonReleased(button) => {
            state.buttons.release(button);
        }
        InputEvent::WheelScrolled { delta } => {
            state.wheel_accumulator += *delta;
            state.mouse_wheel_accumulator[1] += *delta;
            state.mouse_wheel_unit = MouseScrollUnit::Line;
            state
                .mouse_wheel_events
                .push(MouseWheelEvent::lines(0.0, *delta));
        }
        InputEvent::MouseWheel(wheel) => {
            // 兼容纵向值换算为行；xy 聚合仍直接累加各事件原值，单位只记录最后一项，精确消费须读取逐事件列表。
            state.wheel_accumulator += wheel.vertical_line_delta();
            state.mouse_wheel_accumulator[0] += wheel.x;
            state.mouse_wheel_accumulator[1] += wheel.y;
            state.mouse_wheel_unit = wheel.unit;
            state.mouse_wheel_events.push(*wheel);
        }
        InputEvent::KeyboardInput {
            key_code,
            logical_key,
            pressed,
            ..
        } => {
            // 物理码与本次可选逻辑名是两个独立按钮身份；text 和 repeat 保留在原事件中，不参与这里的按住更新。
            let key_code = InputButton::KeyCode(*key_code);
            if *pressed {
                state.buttons.press(key_code);
                if let Some(logical_key) = logical_key {
                    state.buttons.press(InputButton::Key(logical_key.clone()));
                }
            } else {
                state.buttons.release(&key_code);
                if let Some(logical_key) = logical_key {
                    state
                        .buttons
                        .release(&InputButton::Key(logical_key.clone()));
                }
            }
        }
        InputEvent::Ime(ime) => match ime {
            ImeEvent::Enabled => {
                state.ime_enabled = true;
            }
            ImeEvent::Disabled => {
                state.ime_enabled = false;
                state.ime_preedit = None;
            }
            ImeEvent::Preedit(preedit) => {
                state.ime_preedit = if preedit.value.is_empty() {
                    None
                } else {
                    Some(preedit.clone())
                };
            }
            ImeEvent::Commit(value) => {
                state.ime_preedit = None;
                state.ime_commits.push(value.clone());
            }
            ImeEvent::DeleteSurrounding(delete) => {
                state.ime_delete_surrounding.push(*delete);
            }
        },
        InputEvent::ImeHostRequest(request) => {
            state.ime_host_requests.push(request.clone());
        }
        InputEvent::CursorHostRequest(request) => {
            state.cursor_host_requests.push(*request);
        }
        InputEvent::WindowStatus(event) => {
            state.window_status_events.push(event.clone());
        }
        InputEvent::FileDragDrop(event) => {
            state.file_drag_drop_events.push(event.clone());
        }
        InputEvent::FocusLost => {
            state.clear_active_input_for_focus_loss();
        }
        InputEvent::Touch { id, phase, x, y } => match phase {
            TouchPhase::Started | TouchPhase::Moved => {
                state.active_touches.insert(
                    *id,
                    TouchPoint {
                        id: *id,
                        position: [*x, *y],
                        phase: *phase,
                    },
                );
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                state.active_touches.remove(id);
            }
        },
        InputEvent::GamepadConnection(info) => {
            let supersedes = state
                .gamepad_generations
                .get(&info.gamepad)
                .map_or(false, |(generation, _)| info.generation > *generation);
            if !info.connected || supersedes {
                // 先从断连前的轴缓存追加归零记录，再删除该设备样本；直接追加可与本帧既有记录重复。
                state.connected_gamepads.remove(&info.gamepad);
                state.append_disconnected_axis_transitions(info.gamepad);
                state
                    .gamepad_axes
                    .retain(|(gamepad, _), _| gamepad != &info.gamepad);
                state
                    .gamepad_button_values
                    .retain(|(gamepad, _), _| gamepad != &info.gamepad);
                state.buttons.release_where(|button| {
                    matches!(button, InputButton::Gamepad { gamepad, .. } if gamepad == &info.gamepad)
                });
                state.forget_gamepad_axis_transition_indices(info.gamepad);
            }
            if info.connected {
                state.connected_gamepads.insert(info.gamepad);
            }
            state
                .gamepad_generations
                .insert(info.gamepad, (info.generation, info.connected));
        }
        InputEvent::GamepadButton {
            gamepad,
            button,
            value,
            pressed,
            ..
        } => {
            let input_button = InputButton::Gamepad {
                gamepad: *gamepad,
                button: *button,
            };
            let previous_value = state
                .gamepad_button_values
                .get(&(*gamepad, *button))
                .copied();
            let settings = GamepadButtonAxisSettings::default();
            // 只有幅值处理接受后才更新幅值和布尔按压；被过滤的采样即使 host 标志改变，也不进入下面的转换。
            if let Some(value) = settings.process_value(*value, previous_value) {
                state
                    .gamepad_button_values
                    .insert((*gamepad, *button), value);
                let currently_pressed = state.buttons.pressed(&input_button);
                if button_should_press(value, currently_pressed, *pressed) {
                    state.buttons.press(input_button);
                } else if button_should_release(value, *pressed) {
                    state.buttons.release(&input_button);
                }
            }
        }
        InputEvent::GamepadAxis {
            gamepad,
            axis,
            value,
            ..
        } => {
            let previous_value = state.gamepad_axes.get(&(*gamepad, *axis)).copied();
            if let Some(value) =
                GamepadAxisSettings::default().process_value(*value, previous_value)
            {
                let previous_value = previous_value.unwrap_or(0.0);
                if previous_value != value {
                    state.record_gamepad_axis_transition(*gamepad, *axis, previous_value, value);
                }
                // 接受零值也保存当前样本；上面的端点比较只决定是否生成变化记录。
                state.gamepad_axes.insert((*gamepad, *axis), value);
            }
        }
        InputEvent::GamepadRumbleRequest(request) => {
            let gamepad = request.gamepad();
            let generation = state.gamepad_generations[&gamepad].0;
            // Admission and capture share this state lock. No queue/event/recorder mutation
            // occurs on rejection; later disconnects must not relabel accepted commands.
            state
                .gamepad_rumble_requests
                .push(QueuedGamepadRumbleRequest::new(*request, generation));
        }
    }
    // 先尝试保存未合并的原事件，再移入可合并的瞬时缓冲；状态过滤不等于丢弃这次事件。
    state.event_recorder.record(&event);
    state.frame_events.push(event);
}

fn gamepad_input_admission(state: &InputState, event: &InputEvent) -> InputEventAdmission {
    gamepad_admission(event, |gamepad| {
        state.gamepad_generations.get(&gamepad).copied()
    })
}

pub(super) fn gamepad_admission(
    event: &InputEvent,
    mut lookup: impl FnMut(crate::input::GamepadId) -> Option<(std::num::NonZeroU64, bool)>,
) -> InputEventAdmission {
    use GamepadInputRejection as Rejection;
    let (gamepad, submitted, lifecycle) = match event {
        InputEvent::ButtonPressed(InputButton::Gamepad { gamepad, .. })
        | InputEvent::ButtonReleased(InputButton::Gamepad { gamepad, .. }) => {
            return InputEventAdmission::Rejected(Rejection::GenerationRequired {
                gamepad: *gamepad,
            });
        }
        InputEvent::GamepadConnection(info) => {
            (info.gamepad, info.generation, Some(info.connected))
        }
        InputEvent::GamepadButton {
            gamepad,
            generation,
            ..
        }
        | InputEvent::GamepadAxis {
            gamepad,
            generation,
            ..
        } => (*gamepad, *generation, None),
        InputEvent::GamepadRumbleRequest(request) => {
            let gamepad = request.gamepad();
            return if matches!(lookup(gamepad), Some((_, true))) {
                InputEventAdmission::Accepted
            } else {
                InputEventAdmission::Rejected(Rejection::RumbleTargetUnavailable { gamepad })
            };
        }
        _ => return InputEventAdmission::Accepted,
    };
    let Some((current, connected)) = lookup(gamepad) else {
        return if lifecycle.is_some() {
            InputEventAdmission::Accepted
        } else {
            InputEventAdmission::Rejected(Rejection::ConnectionNotObserved { gamepad, submitted })
        };
    };
    let rejection = if submitted < current {
        Some(Rejection::StaleGeneration {
            gamepad,
            submitted,
            current,
        })
    } else if submitted > current && lifecycle.is_none() {
        Some(Rejection::FutureGeneration {
            gamepad,
            submitted,
            current,
        })
    } else if submitted == current && !connected && lifecycle != Some(false) {
        Some(Rejection::ConnectionClosed {
            gamepad,
            submitted,
            current,
        })
    } else {
        None
    };
    rejection.map_or(InputEventAdmission::Accepted, InputEventAdmission::Rejected)
}

/// 判断已接受幅值是否允许进入按钮按下分支。
///
/// # 参数
/// `value` 是默认管理器已接受的处理后幅值；`currently_pressed` 是该完整按钮身份当前按住状态；`host_pressed` 是这次事件的宿主按压标志。这些标量按值传入，函数不借用按钮集合。
///
/// # 返回与调用时机
/// 返回宿主标志为真且默认滞回候选为 `Some(true)` 时的布尔真值。候选缺失或建议释放均返回假；函数不更新集合，调用方只在幅值接受后使用结果。
///
/// # 参考
/// 参见 [`GamepadButtonSettings::transition_for_value`] 与 [`button_should_release`]。
fn button_should_press(value: f32, currently_pressed: bool, host_pressed: bool) -> bool {
    host_pressed
        && GamepadButtonSettings::default()
            .transition_for_value(value, currently_pressed)
            .unwrap_or(false)
}

/// 判断已接受幅值是否允许进入按钮释放分支。
///
/// # 参数
/// `value` 是默认管理器已接受的处理后幅值；`host_pressed` 是事件宿主按压标志。该函数不查询当前按住集合，也不自行执行幅值过滤。
///
/// # 返回与调用时机
/// 宿主标志为假，或默认释放阈值比较成立时返回真；只在按下判定未成立时由提交入口检查。被幅值过滤的事件不会调用此函数；真值仅允许尝试释放，重复释放的边沿由按钮集合决定。
///
/// # 参考
/// 参见 [`GamepadButtonSettings::is_released`] 与 [`button_should_press`]。
fn button_should_release(value: f32, host_pressed: bool) -> bool {
    !host_pressed || GamepadButtonSettings::default().is_released(value)
}
