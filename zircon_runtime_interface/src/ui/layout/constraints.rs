use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum StretchMode {
    Fixed,
    #[default]
    Stretch,
}

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
