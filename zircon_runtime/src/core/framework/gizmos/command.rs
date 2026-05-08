use serde::{Deserialize, Serialize};

use crate::core::math::{Real, Transform, Vec2, Vec3, Vec4};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GizmoAxis {
    X,
    Y,
    Z,
}

impl GizmoAxis {
    pub const fn direction(self) -> Vec3 {
        match self {
            Self::X => Vec3::X,
            Self::Y => Vec3::Y,
            Self::Z => Vec3::Z,
        }
    }
}

/// Runtime-neutral gizmo drawing command collected by immediate or retained buffers.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum GizmoCommand {
    Line {
        start: Vec3,
        end: Vec3,
        color: Vec4,
    },
    Ray {
        start: Vec3,
        vector: Vec3,
        color: Vec4,
    },
    LineStrip {
        points: Vec<Vec3>,
        color: Vec4,
    },
    Rect {
        transform: Transform,
        size: Vec2,
        color: Vec4,
    },
    Circle {
        center: Vec3,
        normal: Vec3,
        radius: Real,
        color: Vec4,
    },
    Sphere {
        center: Vec3,
        radius: Real,
        color: Vec4,
    },
    Cube {
        transform: Transform,
        size: Vec3,
        color: Vec4,
    },
    Aabb {
        min: Vec3,
        max: Vec3,
        color: Vec4,
    },
    Axis {
        origin: Vec3,
        axis: GizmoAxis,
        length: Real,
        color: Vec4,
    },
}

impl GizmoCommand {
    pub const fn color(&self) -> Vec4 {
        match self {
            Self::Line { color, .. }
            | Self::Ray { color, .. }
            | Self::LineStrip { color, .. }
            | Self::Rect { color, .. }
            | Self::Circle { color, .. }
            | Self::Sphere { color, .. }
            | Self::Cube { color, .. }
            | Self::Aabb { color, .. }
            | Self::Axis { color, .. } => *color,
        }
    }
}
