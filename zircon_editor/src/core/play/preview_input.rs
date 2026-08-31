use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::core::gateway::GatewayError;

use super::PlayModeKind;

#[derive(Debug)]
pub enum PlayPreviewInputError {
    GatewayUnavailable { mode: PlayModeKind },
    Dispatch(GatewayError),
}

impl Display for PlayPreviewInputError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GatewayUnavailable { mode } => write!(
                formatter,
                "play preview input gateway is unavailable while mode is {mode:?}"
            ),
            Self::Dispatch(source) => {
                write!(formatter, "failed to route play preview input: {source}")
            }
        }
    }
}

impl Error for PlayPreviewInputError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::GatewayUnavailable { .. } => None,
            Self::Dispatch(source) => Some(source),
        }
    }
}
