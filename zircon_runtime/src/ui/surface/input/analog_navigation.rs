use std::borrow::Cow;

use zircon_runtime_interface::ui::{dispatch::UiAnalogInputEvent, surface::UiNavigationEventKind};

use super::state::{UiSurfaceAnalogNavigationState, UiSurfaceInputState};

const ANALOG_NAVIGATION_THRESHOLD: f32 = 0.5;
const ANALOG_NAVIGATION_FIRST_REPEAT_MICROS: u64 = 500_000;
const ANALOG_NAVIGATION_REPEAT_MICROS: u64 = 250_000;
const ANALOG_NAVIGATION_HIGH_PRESSURE_THRESHOLD: f32 = 0.9;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AnalogNavigationDecision {
    Navigate(UiNavigationEventKind),
    Suppressed(UiNavigationEventKind),
    Inactive,
}

pub(super) fn analog_navigation_decision(
    input: &mut UiSurfaceInputState,
    analog: &UiAnalogInputEvent,
    analog_value: f32,
) -> AnalogNavigationDecision {
    let normalized_control = normalized_control_name(analog.control.as_str());
    let Some(axis) = analog_navigation_axis(normalized_control.as_ref()) else {
        return AnalogNavigationDecision::Inactive;
    };
    let Some(kind) = analog_navigation_kind(axis, analog_value) else {
        reset_analog_navigation_axis(input, analog, normalized_control.as_ref(), axis);
        return AnalogNavigationDecision::Inactive;
    };
    if allow_analog_navigation_repeat(
        input,
        analog,
        analog_value,
        normalized_control.as_ref(),
        kind,
    ) {
        AnalogNavigationDecision::Navigate(kind)
    } else {
        AnalogNavigationDecision::Suppressed(kind)
    }
}

fn analog_navigation_kind(axis: AnalogNavigationAxis, value: f32) -> Option<UiNavigationEventKind> {
    match axis {
        AnalogNavigationAxis::Horizontal if value < -ANALOG_NAVIGATION_THRESHOLD => {
            Some(UiNavigationEventKind::Left)
        }
        AnalogNavigationAxis::Horizontal if value > ANALOG_NAVIGATION_THRESHOLD => {
            Some(UiNavigationEventKind::Right)
        }
        AnalogNavigationAxis::Vertical if value > ANALOG_NAVIGATION_THRESHOLD => {
            Some(UiNavigationEventKind::Up)
        }
        AnalogNavigationAxis::Vertical if value < -ANALOG_NAVIGATION_THRESHOLD => {
            Some(UiNavigationEventKind::Down)
        }
        _ => None,
    }
}

fn allow_analog_navigation_repeat(
    input: &mut UiSurfaceInputState,
    analog: &UiAnalogInputEvent,
    analog_value: f32,
    normalized_control: &str,
    kind: UiNavigationEventKind,
) -> bool {
    let key = analog_navigation_state_key(analog, normalized_control, kind);
    let now = analog.metadata.timestamp.monotonic_micros;
    match input.analog_navigation.get_mut(key.as_str()) {
        Some(state) => {
            let elapsed = now.saturating_sub(state.last_navigation_time_micros);
            if elapsed > repeat_rate_micros(analog_value.abs(), state.repeats) {
                state.last_navigation_time_micros = now;
                state.repeats = state.repeats.saturating_add(1);
                true
            } else {
                false
            }
        }
        None => {
            input.analog_navigation.insert(
                key,
                UiSurfaceAnalogNavigationState {
                    kind,
                    last_navigation_time_micros: now,
                    repeats: 1,
                },
            );
            true
        }
    }
}

fn reset_analog_navigation_axis(
    input: &mut UiSurfaceInputState,
    analog: &UiAnalogInputEvent,
    normalized_control: &str,
    axis: AnalogNavigationAxis,
) {
    let directions = match axis {
        AnalogNavigationAxis::Horizontal => {
            [UiNavigationEventKind::Left, UiNavigationEventKind::Right]
        }
        AnalogNavigationAxis::Vertical => [UiNavigationEventKind::Up, UiNavigationEventKind::Down],
    };
    for kind in directions {
        input
            .analog_navigation
            .remove(analog_navigation_state_key(analog, normalized_control, kind).as_str());
    }
}

fn repeat_rate_micros(pressure: f32, repeats: u32) -> u64 {
    let base = if repeats <= 1 {
        ANALOG_NAVIGATION_FIRST_REPEAT_MICROS
    } else {
        ANALOG_NAVIGATION_REPEAT_MICROS
    };
    if pressure > ANALOG_NAVIGATION_HIGH_PRESSURE_THRESHOLD {
        base / 2
    } else {
        base
    }
}

fn analog_navigation_state_key(
    analog: &UiAnalogInputEvent,
    normalized_control: &str,
    kind: UiNavigationEventKind,
) -> String {
    let user_id = analog
        .metadata
        .user_id
        .map(|user_id| user_id.0)
        .unwrap_or(0);
    format!("user{user_id}:{normalized_control}:{kind:?}")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AnalogNavigationAxis {
    Horizontal,
    Vertical,
}

fn analog_navigation_axis(normalized_control: &str) -> Option<AnalogNavigationAxis> {
    match normalized_control {
        "gamepadleftx" | "gamepadleftstickx" | "gamepadleftanalogx" | "leftx" | "leftstickx"
        | "axisleftx" => Some(AnalogNavigationAxis::Horizontal),
        "gamepadlefty" | "gamepadleftsticky" | "gamepadleftanalogy" | "lefty" | "leftsticky"
        | "axislefty" => Some(AnalogNavigationAxis::Vertical),
        _ => None,
    }
}

fn normalized_control_name(control: &str) -> Cow<'_, str> {
    if control
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    {
        Cow::Borrowed(control)
    } else {
        Cow::Owned(
            control
                .chars()
                .filter(|ch| ch.is_ascii_alphanumeric())
                .map(|ch| ch.to_ascii_lowercase())
                .collect(),
        )
    }
}

#[cfg(test)]
#[path = "analog_navigation/single_normalization_tests.rs"]
mod single_normalization_tests;
