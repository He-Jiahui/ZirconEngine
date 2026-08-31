use std::fmt::{Display, Formatter};

use super::PlayStartRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlayModeKind {
    #[default]
    Edit,
    Building,
    Playing,
    CleanupFailed,
}

impl PlayModeKind {
    /// Only `Playing` has a live runtime backend that can receive runtime work.
    pub const fn has_active_runtime(self) -> bool {
        matches!(self, Self::Playing)
    }
}

/// A cleanup owner that remains retryable after the runtime has reached its terminal state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlayCleanupFailure {
    PluginDeactivation {
        message: String,
    },
    BackendRetirement {
        message: String,
        plugin_deactivation: Option<String>,
    },
}

impl Display for PlayCleanupFailure {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PluginDeactivation { message } => {
                write!(formatter, "plugin deactivation failed: {message}")
            }
            Self::BackendRetirement {
                message,
                plugin_deactivation,
            } => {
                write!(formatter, "play session retirement failed: {message}")?;
                if let Some(plugin_deactivation) = plugin_deactivation {
                    write!(
                        formatter,
                        "; plugin deactivation is also pending: {plugin_deactivation}"
                    )?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayKind {
    #[default]
    Play,
    Simulate,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum PlayMode {
    #[default]
    Edit,
    Building {
        request: PlayStartRequest,
        play_after_build: bool,
    },
    Playing {
        kind: PlayKind,
    },
    CleanupFailed {
        kind: PlayKind,
        failure: PlayCleanupFailure,
    },
}

impl PlayMode {
    pub const fn kind(&self) -> PlayModeKind {
        match self {
            Self::Edit => PlayModeKind::Edit,
            Self::Building { .. } => PlayModeKind::Building,
            Self::Playing { .. } => PlayModeKind::Playing,
            Self::CleanupFailed { .. } => PlayModeKind::CleanupFailed,
        }
    }
}
