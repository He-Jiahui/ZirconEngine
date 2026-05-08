use serde::{Deserialize, Serialize};

use crate::core::math::{Real, Vec4};

pub const DEFAULT_GIZMO_LINE_WIDTH: Real = 2.0;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GizmoConfigGroupId(String);

impl GizmoConfigGroupId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for GizmoConfigGroupId {
    fn default() -> Self {
        Self::new("default")
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GizmoRenderLayer(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum GizmoColorPolicy {
    UseCommandColor,
    Override(Vec4),
    Multiply(Vec4),
}

impl GizmoColorPolicy {
    pub fn apply(self, color: Vec4) -> Vec4 {
        match self {
            Self::UseCommandColor => color,
            Self::Override(override_color) => override_color,
            Self::Multiply(multiplier) => color * multiplier,
        }
    }
}

impl Default for GizmoColorPolicy {
    fn default() -> Self {
        Self::UseCommandColor
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum GizmoScreenScalePolicy {
    World,
    Screen { scale: Real },
}

impl Default for GizmoScreenScalePolicy {
    fn default() -> Self {
        Self::World
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GizmoLineConfig {
    pub width: Real,
}

impl Default for GizmoLineConfig {
    fn default() -> Self {
        Self {
            width: DEFAULT_GIZMO_LINE_WIDTH,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GizmoConfig {
    pub group: GizmoConfigGroupId,
    pub enabled: bool,
    pub line: GizmoLineConfig,
    pub depth_bias: Real,
    pub render_layer: GizmoRenderLayer,
    pub color_policy: GizmoColorPolicy,
    pub screen_scale_policy: GizmoScreenScalePolicy,
}

impl Default for GizmoConfig {
    fn default() -> Self {
        Self {
            group: GizmoConfigGroupId::default(),
            enabled: true,
            line: GizmoLineConfig::default(),
            depth_bias: 0.0,
            render_layer: GizmoRenderLayer::default(),
            color_policy: GizmoColorPolicy::default(),
            screen_scale_policy: GizmoScreenScalePolicy::default(),
        }
    }
}
