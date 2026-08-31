use thiserror::Error;

use crate::core::context::ToolSchedulerServiceError;
use crate::core::settings::{SettingsError, SettingsMutationError};
use crate::core::tools::{AcquireDenial, ToolDefinitionIdError};
use crate::scene::modes::{SceneModeActivationError, SceneModeRegistryError, SceneModeStackError};

use super::ViewportOverlayProviderError;

#[derive(Debug, Error, PartialEq)]
pub(crate) enum SceneViewportControllerError {
    #[error(transparent)]
    SceneModeActivation(#[from] SceneModeActivationError),
    #[error(transparent)]
    SceneModeRegistry(#[from] SceneModeRegistryError),
    #[error(transparent)]
    SceneModeStack(#[from] SceneModeStackError),
    #[error(transparent)]
    Settings(#[from] SettingsError),
    #[error(transparent)]
    SettingsMutation(#[from] SettingsMutationError),
    #[error(transparent)]
    ViewportOverlayProvider(#[from] ViewportOverlayProviderError),
    #[error(transparent)]
    ToolScheduler(#[from] ToolSchedulerServiceError),
    #[error(transparent)]
    ToolDefinition(#[from] ToolDefinitionIdError),
    #[error("scene viewport tool is queued at position {position}")]
    SceneToolQueued { position: usize },
    #[error("scene viewport tool admission denied: {reason:?}")]
    SceneToolDenied { reason: AcquireDenial },
    #[error("viewport snap step {value:?} must be finite")]
    InvalidSnapStep { value: f32 },
}
