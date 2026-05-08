use serde::{Deserialize, Serialize};

use crate::core::math::Transform;

use super::{GizmoBuffer, GizmoCommand, GizmoConfig};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GizmoAsset {
    commands: Vec<GizmoCommand>,
}

impl GizmoAsset {
    pub fn from_buffer(buffer: &GizmoBuffer) -> Self {
        Self {
            commands: buffer.commands().to_vec(),
        }
    }

    pub fn commands(&self) -> &[GizmoCommand] {
        &self.commands
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RetainedGizmo {
    pub asset: GizmoAsset,
    pub transform: Transform,
    pub config: GizmoConfig,
}

impl RetainedGizmo {
    pub fn new(asset: GizmoAsset) -> Self {
        Self {
            asset,
            transform: Transform::identity(),
            config: GizmoConfig::default(),
        }
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    pub fn with_config(mut self, config: GizmoConfig) -> Self {
        self.config = config;
        self
    }
}
