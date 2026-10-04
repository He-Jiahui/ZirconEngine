use serde::{Deserialize, Serialize};

use super::{GamepadAxis, GamepadId, InputButton};

/// 选择整根轴或单侧轴参与动作；单侧值折为非负幅值，便于同一动作绑定不同方向。
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

/// 指定单台手柄的一根轴作为动作来源；求值先按设备和轴匹配帧样本，再按 direction 保留整轴值或折为单侧幅值。
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

/// 将一个命名动作关联到按钮和模拟轴；按钮作为组合键，含轴时还要求轴产生有效值。
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
        buttons.sort_unstable();
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

    /// 构造时统一排序去重，使重复设备输入不会改变求值或序列化后的绑定身份。
    pub fn buttons_and_axes(
        action: impl Into<String>,
        buttons: impl IntoIterator<Item = InputButton>,
        axes: impl IntoIterator<Item = InputAxisBinding>,
    ) -> Self {
        let mut binding = Self::chord(action, buttons);
        let mut axes = axes.into_iter().collect::<Vec<_>>();
        axes.sort_unstable_by(|left, right| {
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

#[cfg(test)]
#[path = "tests/input_binding.rs"]
mod tests;
