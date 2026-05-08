use serde::{Deserialize, Serialize};

use crate::core::math::{Real, Transform, Vec2, Vec3, Vec4};

use super::{GizmoAxis, GizmoCommand, GizmoConfig};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GizmoBuffer {
    config: GizmoConfig,
    commands: Vec<GizmoCommand>,
}

impl Default for GizmoBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl GizmoBuffer {
    pub fn new() -> Self {
        Self {
            config: GizmoConfig::default(),
            commands: Vec::new(),
        }
    }

    pub fn with_config(config: GizmoConfig) -> Self {
        Self {
            config,
            commands: Vec::new(),
        }
    }

    pub fn config(&self) -> &GizmoConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut GizmoConfig {
        &mut self.config
    }

    pub fn commands(&self) -> &[GizmoCommand] {
        &self.commands
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn push_command(&mut self, command: GizmoCommand) -> &mut Self {
        if self.config.enabled {
            self.commands.push(command);
        }
        self
    }

    pub fn line(&mut self, start: Vec3, end: Vec3, color: Vec4) -> &mut Self {
        self.push_command(GizmoCommand::Line { start, end, color })
    }

    pub fn ray(&mut self, start: Vec3, vector: Vec3, color: Vec4) -> &mut Self {
        self.push_command(GizmoCommand::Ray {
            start,
            vector,
            color,
        })
    }

    pub fn linestrip(&mut self, points: impl IntoIterator<Item = Vec3>, color: Vec4) -> &mut Self {
        self.push_command(GizmoCommand::LineStrip {
            points: points.into_iter().collect(),
            color,
        })
    }

    pub fn rect(&mut self, transform: Transform, size: Vec2, color: Vec4) -> &mut Self {
        self.push_command(GizmoCommand::Rect {
            transform,
            size,
            color,
        })
    }

    pub fn circle(&mut self, center: Vec3, normal: Vec3, radius: Real, color: Vec4) -> &mut Self {
        self.push_command(GizmoCommand::Circle {
            center,
            normal,
            radius,
            color,
        })
    }

    pub fn sphere(&mut self, center: Vec3, radius: Real, color: Vec4) -> &mut Self {
        self.push_command(GizmoCommand::Sphere {
            center,
            radius,
            color,
        })
    }

    pub fn cube(&mut self, transform: Transform, size: Vec3, color: Vec4) -> &mut Self {
        self.push_command(GizmoCommand::Cube {
            transform,
            size,
            color,
        })
    }

    pub fn aabb(&mut self, min: Vec3, max: Vec3, color: Vec4) -> &mut Self {
        self.push_command(GizmoCommand::Aabb { min, max, color })
    }

    pub fn axis(&mut self, origin: Vec3, axis: GizmoAxis, length: Real, color: Vec4) -> &mut Self {
        self.push_command(GizmoCommand::Axis {
            origin,
            axis,
            length,
            color,
        })
    }
}
