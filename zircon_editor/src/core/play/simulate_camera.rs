use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::core::gateway::GatewayError;

use super::PlayModeKind;

#[derive(Debug)]
pub enum PlaySimulateCameraError {
    GatewayUnavailable { mode: PlayModeKind },
    Encode(serde_json::Error),
    PayloadTooLarge { len: usize, limit: usize },
    Dispatch(GatewayError),
}

impl Display for PlaySimulateCameraError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GatewayUnavailable { mode } => write!(
                formatter,
                "simulate camera gateway is unavailable while mode is {mode:?}"
            ),
            Self::Encode(source) => write!(formatter, "failed to encode simulate camera: {source}"),
            Self::PayloadTooLarge { len, limit } => write!(
                formatter,
                "simulate camera payload is {len} bytes and exceeds the {limit}-byte limit"
            ),
            Self::Dispatch(source) => {
                write!(formatter, "failed to route simulate camera: {source}")
            }
        }
    }
}

impl Error for PlaySimulateCameraError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Encode(source) => Some(source),
            Self::Dispatch(source) => Some(source),
            Self::GatewayUnavailable { .. } | Self::PayloadTooLarge { .. } => None,
        }
    }
}
