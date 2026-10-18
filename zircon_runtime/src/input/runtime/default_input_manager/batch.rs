use std::collections::BTreeMap;
use std::mem::size_of;

use crate::core::framework::input::{
    CursorHostRequest, FileDragDropEvent, GamepadRumbleRequest, ImeEvent, ImeHostRequest,
    InputBatchLimits, InputBatchRejection, InputBatchSize, InputButton, InputEvent,
    InputEventAdmission, WindowStatusEvent,
};

use super::super::InputState;

pub(super) fn preflight(
    state: &InputState,
    events: &Vec<InputEvent>,
    limits: InputBatchLimits,
) -> Result<InputBatchSize, InputBatchRejection> {
    use InputBatchRejection as Rejection;
    if events.is_empty() {
        return Err(Rejection::Empty);
    }
    if events.len() > limits.max_events {
        return Err(Rejection::EventCapacity {
            requested: events.len(),
            remaining: limits.max_events,
        });
    }
    let mut owned_bytes = events
        .capacity()
        .checked_mul(size_of::<InputEvent>())
        .ok_or(Rejection::SizeOverflow)?;
    for event in events {
        owned_bytes = owned_bytes
            .checked_add(owned_string_bytes(event)?)
            .ok_or(Rejection::SizeOverflow)?;
    }
    if owned_bytes > limits.max_owned_bytes {
        return Err(Rejection::OwnedByteCapacity {
            requested: owned_bytes,
            remaining: limits.max_owned_bytes,
        });
    }

    // Only touched device identities enter this scratch map. The canonical
    // physical state and recorder remain untouched until the complete pass succeeds.
    let mut generations = BTreeMap::new();
    let mut wheel = state.wheel_accumulator;
    let mut mouse_wheel = state.mouse_wheel_accumulator;
    let mut motion = state.mouse_motion_accumulator;
    for (event_index, event) in events.iter().enumerate() {
        if !finite(event) {
            return Err(Rejection::NonFinite { event_index });
        }
        if !valid_payload(event) {
            return Err(Rejection::InvalidPayload { event_index });
        }
        let admission = super::gamepad_admission(event, |id| {
            generations
                .get(&id)
                .copied()
                .unwrap_or_else(|| state.gamepad_generations.get(&id).copied())
        });
        if let InputEventAdmission::Rejected(reason) = admission {
            return Err(Rejection::Gamepad {
                event_index,
                reason,
            });
        }
        if let InputEvent::GamepadConnection(info) = event {
            generations.insert(info.gamepad, Some((info.generation, info.connected)));
        }
        match event {
            InputEvent::MouseMotion { delta_x, delta_y } => {
                motion[0] += *delta_x;
                motion[1] += *delta_y;
            }
            InputEvent::WheelScrolled { delta } => {
                wheel += *delta;
                mouse_wheel[1] += *delta;
            }
            InputEvent::MouseWheel(event) => {
                wheel += event.vertical_line_delta();
                mouse_wheel[0] += event.x;
                mouse_wheel[1] += event.y;
            }
            InputEvent::FocusLost => {
                wheel = 0.0;
                mouse_wheel = [0.0, 0.0];
                motion = [0.0, 0.0];
            }
            _ => {}
        }
        if !wheel.is_finite()
            || !mouse_wheel
                .iter()
                .chain(motion.iter())
                .all(|value| value.is_finite())
        {
            return Err(Rejection::NonFinite { event_index });
        }
    }
    Ok(InputBatchSize {
        events: events.len(),
        owned_bytes,
    })
}

fn owned_string_bytes(event: &InputEvent) -> Result<usize, InputBatchRejection> {
    let one = match event {
        InputEvent::ButtonPressed(InputButton::Key(value))
        | InputEvent::ButtonReleased(InputButton::Key(value))
        | InputEvent::Ime(ImeEvent::Commit(value)) => value.capacity(),
        InputEvent::KeyboardInput {
            logical_key, text, ..
        } => {
            return logical_key
                .as_ref()
                .map_or(0, String::capacity)
                .checked_add(text.as_ref().map_or(0, String::capacity))
                .ok_or(InputBatchRejection::SizeOverflow);
        }
        InputEvent::Ime(ImeEvent::Preedit(value)) => value.value.capacity(),
        InputEvent::ImeHostRequest(ImeHostRequest::SetSurroundingText(value)) => {
            value.value.capacity()
        }
        InputEvent::FileDragDrop(FileDragDropEvent::Hovered { path })
        | InputEvent::FileDragDrop(FileDragDropEvent::Dropped { path }) => path.capacity(),
        InputEvent::GamepadConnection(info) => info.name.as_ref().map_or(0, String::capacity),
        InputEvent::CursorMoved { .. }
        | InputEvent::CursorEntered
        | InputEvent::CursorLeft
        | InputEvent::MouseMotion { .. }
        | InputEvent::ButtonPressed(_)
        | InputEvent::ButtonReleased(_)
        | InputEvent::WheelScrolled { .. }
        | InputEvent::MouseWheel(_)
        | InputEvent::WindowStatus(_)
        | InputEvent::FileDragDrop(FileDragDropEvent::Cancelled)
        | InputEvent::Ime(
            ImeEvent::Enabled | ImeEvent::Disabled | ImeEvent::DeleteSurrounding(_),
        )
        | InputEvent::ImeHostRequest(
            ImeHostRequest::Enable | ImeHostRequest::Disable | ImeHostRequest::SetCursorArea(_),
        )
        | InputEvent::CursorHostRequest(_)
        | InputEvent::FocusLost
        | InputEvent::Touch { .. }
        | InputEvent::GamepadButton { .. }
        | InputEvent::GamepadAxis { .. }
        | InputEvent::GamepadRumbleRequest(_) => 0,
    };
    Ok(one)
}

fn finite(event: &InputEvent) -> bool {
    match event {
        InputEvent::CursorMoved { x, y } | InputEvent::Touch { x, y, .. } => {
            x.is_finite() && y.is_finite()
        }
        InputEvent::MouseMotion { delta_x, delta_y } => delta_x.is_finite() && delta_y.is_finite(),
        InputEvent::WheelScrolled { delta } => delta.is_finite(),
        InputEvent::MouseWheel(wheel) => {
            wheel.x.is_finite() && wheel.y.is_finite() && wheel.vertical_line_delta().is_finite()
        }
        InputEvent::GamepadButton { value, .. } | InputEvent::GamepadAxis { value, .. } => {
            value.is_finite()
        }
        InputEvent::GamepadRumbleRequest(GamepadRumbleRequest::Add { intensity, .. }) => {
            intensity.strong_motor.is_finite() && intensity.weak_motor.is_finite()
        }
        InputEvent::ImeHostRequest(ImeHostRequest::SetCursorArea(area)) => {
            [area.x, area.y, area.width, area.height]
                .iter()
                .all(|value| value.is_finite())
        }
        InputEvent::CursorHostRequest(CursorHostRequest::SetPosition(position)) => {
            position.x.is_finite() && position.y.is_finite()
        }
        InputEvent::WindowStatus(
            WindowStatusEvent::ScaleFactorChanged { scale_factor }
            | WindowStatusEvent::BackendScaleFactorChanged { scale_factor },
        ) => scale_factor.is_finite(),
        _ => true,
    }
}

fn valid_payload(event: &InputEvent) -> bool {
    match event {
        InputEvent::Ime(ImeEvent::Preedit(preedit)) => {
            preedit.cursor.as_ref().map_or(true, |range| {
                range.start <= range.end
                    && preedit.value.is_char_boundary(range.start)
                    && preedit.value.is_char_boundary(range.end)
            })
        }
        InputEvent::ImeHostRequest(ImeHostRequest::SetSurroundingText(text)) => {
            text.value.is_char_boundary(text.cursor)
                && text.value.is_char_boundary(text.anchor)
                && text.composition_range.as_ref().map_or(true, |range| {
                    range.start <= range.end
                        && text.value.is_char_boundary(range.start)
                        && text.value.is_char_boundary(range.end)
                })
        }
        InputEvent::ImeHostRequest(ImeHostRequest::SetCursorArea(area)) => {
            area.width >= 0.0 && area.height >= 0.0
        }
        InputEvent::WindowStatus(
            WindowStatusEvent::ScaleFactorChanged { scale_factor }
            | WindowStatusEvent::BackendScaleFactorChanged { scale_factor },
        ) => *scale_factor > 0.0,
        _ => true,
    }
}

#[cfg(test)]
#[path = "tests/batch.rs"]
mod tests;
