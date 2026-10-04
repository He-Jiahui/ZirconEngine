use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::core::math::Transform;

use super::{GizmoBuffer, GizmoCommand, GizmoConfig};

/// 从命令缓冲冻结出的可共享几何资产；克隆实例共享命令，配置由每个 RetainedGizmo 单独持有。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GizmoAsset {
    commands: Arc<[GizmoCommand]>,
}

impl GizmoAsset {
    /// 只复制当前命令，不继承源缓冲的配置；创建 RetainedGizmo 后按实例设置策略。
    pub fn from_buffer(buffer: &GizmoBuffer) -> Self {
        Self {
            commands: Arc::from(buffer.commands()),
        }
    }

    pub fn commands(&self) -> &[GizmoCommand] {
        &self.commands
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

/// 对共享命令资产附加实例变换和绘制策略，提取时才投影到场景覆盖层。
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
