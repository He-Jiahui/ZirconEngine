use serde::{Deserialize, Serialize};

use super::{GamepadAxis, GamepadId, InputButton};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum InputAxisDirection {
    #[default]
    Full,
    Positive,
    Negative,
}

impl InputAxisDirection {
    pub fn value(self, source: f32) -> f32 {
        let value = if source.is_finite() {
            source.clamp(-1.0, 1.0)
        } else {
            0.0
        };

        match self {
            Self::Full => normalized_axis_value(value),
            Self::Positive => normalized_axis_value(value.max(0.0)),
            Self::Negative => normalized_axis_value((-value).max(0.0)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputAxisBinding {
    pub gamepad: GamepadId,
    pub axis: GamepadAxis,
    #[serde(default)]
    pub direction: InputAxisDirection,
}

impl InputAxisBinding {
    pub const fn new(gamepad: GamepadId, axis: GamepadAxis) -> Self {
        Self {
            gamepad,
            axis,
            direction: InputAxisDirection::Full,
        }
    }

    pub const fn positive(gamepad: GamepadId, axis: GamepadAxis) -> Self {
        Self {
            gamepad,
            axis,
            direction: InputAxisDirection::Positive,
        }
    }

    pub const fn negative(gamepad: GamepadId, axis: GamepadAxis) -> Self {
        Self {
            gamepad,
            axis,
            direction: InputAxisDirection::Negative,
        }
    }

    pub fn value(self, source: f32) -> f32 {
        self.direction.value(source)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputBinding {
    pub action: String,
    #[serde(default)]
    pub buttons: Vec<InputButton>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub axes: Vec<InputAxisBinding>,
}

impl InputBinding {
    pub fn button(action: impl Into<String>, button: InputButton) -> Self {
        Self::chord(action, [button])
    }

    pub fn chord(
        action: impl Into<String>,
        buttons: impl IntoIterator<Item = InputButton>,
    ) -> Self {
        let mut buttons = buttons.into_iter().collect::<Vec<_>>();
        buttons.sort();
        buttons.dedup();
        Self {
            action: action.into(),
            buttons,
            axes: Vec::new(),
        }
    }

    pub fn axis(action: impl Into<String>, axis: InputAxisBinding) -> Self {
        Self::axes(action, [axis])
    }

    pub fn axes(
        action: impl Into<String>,
        axes: impl IntoIterator<Item = InputAxisBinding>,
    ) -> Self {
        Self::buttons_and_axes(action, std::iter::empty(), axes)
    }

    pub fn buttons_and_axes(
        action: impl Into<String>,
        buttons: impl IntoIterator<Item = InputButton>,
        axes: impl IntoIterator<Item = InputAxisBinding>,
    ) -> Self {
        let mut binding = Self::chord(action, buttons);
        let mut axes = axes.into_iter().collect::<Vec<_>>();
        axes.sort_by(|left, right| {
            left.gamepad
                .cmp(&right.gamepad)
                .then(left.axis.cmp(&right.axis))
                .then(left.direction.cmp(&right.direction))
        });
        axes.dedup();
        binding.axes = axes;
        binding
    }

    pub fn is_empty(&self) -> bool {
        self.buttons.is_empty() && self.axes.is_empty()
    }
}

fn normalized_axis_value(value: f32) -> f32 {
    if value == 0.0 {
        0.0
    } else {
        value
    }
}
