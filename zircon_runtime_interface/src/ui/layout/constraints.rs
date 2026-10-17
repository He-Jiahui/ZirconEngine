//! 轴约束在测量前规整为有限下界、可选上界与可用权重，供 Runtime 布局求解。

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum StretchMode {
    Fixed,
    #[default]
    Stretch,
}

/// 负数 max 表示无上界；非正权重在 resolved 时规整为默认权重。
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct AxisConstraint {
    pub min: f32,
    pub max: f32,
    pub preferred: f32,
    pub priority: i32,
    pub weight: f32,
    pub stretch_mode: StretchMode,
}

impl Default for AxisConstraint {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: -1.0,
            preferred: 0.0,
            priority: 0,
            weight: 1.0,
            stretch_mode: StretchMode::Stretch,
        }
    }
}

impl AxisConstraint {
    /// 将作者输入规整为布局求解可消费的区间与初始解析值。
    pub fn resolved(self) -> ResolvedAxisConstraint {
        let min = self.min.max(0.0);
        let max = if self.max < 0.0 {
            None
        } else {
            Some(self.max.max(min))
        };
        let preferred = clamp_axis_value(self.preferred.max(0.0), min, max);
        ResolvedAxisConstraint {
            min,
            max,
            preferred,
            priority: self.priority,
            // BUG: [CR-ZCA-R039-0001] max < 0（无上界）、Stretch、min/preferred 为 0 且 weight 为 NaN 时，此分支保留 NaN；无 slot sizing 覆盖、主轴 slot padding 为 0 的 Linear 父容器中单个可见子节点在可用主轴空间 > 0.001（例如 100）时进入 Runtime native growth 循环，每轮分配量与无上界容量均为 NaN，退出比较不成立，布局线程无法退出。
            weight: if self.weight <= 0.0 { 1.0 } else { self.weight },
            stretch_mode: self.stretch_mode,
            resolved: preferred,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResolvedAxisConstraint {
    pub min: f32,
    pub max: Option<f32>,
    pub preferred: f32,
    pub priority: i32,
    pub weight: f32,
    pub stretch_mode: StretchMode,
    pub resolved: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct BoxConstraints {
    #[serde(default)]
    pub width: AxisConstraint,
    #[serde(default)]
    pub height: AxisConstraint,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DesiredSize {
    pub width: f32,
    pub height: f32,
}

impl DesiredSize {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LayoutBoundary {
    #[default]
    ContentDriven,
    ParentDirected,
    Fixed,
}

impl LayoutBoundary {
    pub const fn propagates_child_layout_invalidation(self) -> bool {
        matches!(self, Self::ContentDriven)
    }
}

fn clamp_axis_value(value: f32, min: f32, max: Option<f32>) -> f32 {
    max.map(|max| value.clamp(min, max))
        .unwrap_or_else(|| value.max(min))
}
