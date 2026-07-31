use std::fmt::{Display, Formatter};

use super::{PlayEditBeginError, PlayModeKind};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlaySessionError {
    PendingEditDecisionRequired {
        pending_count: usize,
    },
    PendingEditResolutionInProgress,
    EditProtectionStart {
        reason: PlayEditBeginError,
        activation_rollback: Option<String>,
    },
    InvalidTransition {
        mode: PlayModeKind,
        event: &'static str,
    },
    PluginActivation(String),
    BackendStart {
        message: String,
        activation_rollback: Option<String>,
    },
    BackendStop(String),
    BackendPoll(String),
}

impl Display for PlaySessionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PendingEditDecisionRequired { pending_count } => write!(
                formatter,
                "{pending_count} pending edit intents require an apply or discard decision"
            ),
            Self::PendingEditResolutionInProgress => {
                formatter.write_str("pending edits are currently being resolved")
            }
            Self::EditProtectionStart {
                reason,
                activation_rollback,
            } => {
                write!(formatter, "failed to start play edit protection: {reason}")?;
                if let Some(rollback) = activation_rollback {
                    write!(
                        formatter,
                        "; plugin activation rollback also failed: {rollback}"
                    )?;
                }
                Ok(())
            }
            Self::InvalidTransition { mode, event } => {
                write!(
                    formatter,
                    "play event `{event}` is invalid while mode is {mode:?}"
                )
            }
            Self::PluginActivation(message) => formatter.write_str(message),
            Self::BackendStart {
                message,
                activation_rollback,
            } => {
                write!(formatter, "failed to start play backend: {message}")?;
                if let Some(rollback) = activation_rollback {
                    write!(
                        formatter,
                        "; plugin activation rollback also failed: {rollback}"
                    )?;
                }
                Ok(())
            }
            Self::BackendStop(message) => {
                write!(formatter, "failed to stop play backend: {message}")
            }
            Self::BackendPoll(message) => {
                write!(formatter, "failed to poll play backend: {message}")
            }
        }
    }
}

impl std::error::Error for PlaySessionError {}
