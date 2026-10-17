//! Gameplay host 导出把场景、导航和 JSON 失败收敛为脚本宿主错误的边界。

use crate::core::framework::navigation::NavigationError;
use crate::core::framework::script::ScriptHostError;
use crate::scene::{EntityId, SceneError};

pub(super) type GameplayHostResult<T> = std::result::Result<T, GameplayHostError>;

#[derive(Debug, thiserror::Error)]
pub(super) enum GameplayHostError {
    #[error(transparent)]
    Scene(#[from] SceneError),
    #[error(transparent)]
    Navigation(#[from] NavigationError),
    #[error("invalid JSON payload: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{operation} entity {entity} is missing")]
    MissingEntity {
        operation: &'static str,
        entity: EntityId,
    },
    #[error(
        "cannot convert world transform for entity {entity}: parent {parent} has a non-invertible world transform"
    )]
    NonInvertibleParentTransform { entity: EntityId, parent: EntityId },
    #[error(
        "cannot represent world transform for entity {entity} as local TRS under parent {parent}"
    )]
    NonRepresentableParentTransform { entity: EntityId, parent: EntityId },
}

impl GameplayHostError {
    pub(super) fn missing_entity(operation: &'static str, entity: EntityId) -> Self {
        Self::Scene(SceneError::MissingEntity { operation, entity })
    }
}

impl From<GameplayHostError> for ScriptHostError {
    // HostExportRegistry 只接收 `ScriptHostError`，因此在离开 gameplay
    // 调用边界时保留格式化后的操作名和实体 ID，供脚本调用方诊断。
    fn from(error: GameplayHostError) -> Self {
        Self::new(error.to_string())
    }
}
