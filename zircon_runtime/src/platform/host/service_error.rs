use std::error::Error;
use std::fmt;

use crate::core::framework::platform::{
    PlatformHostBackendRequestError, PlatformHostInstanceId, PlatformHostLifecycleState,
    PlatformHostOperationId,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlatformHostServiceError {
    AlreadyInstalled {
        state: PlatformHostLifecycleState,
    },
    NoHostInstalled,
    StaleInstance {
        expected: PlatformHostInstanceId,
        received: PlatformHostInstanceId,
    },
    InvalidLifecycleState {
        operation: &'static str,
        state: PlatformHostLifecycleState,
    },
    BackendBridgeMissing,
    BackendRejected {
        reason: PlatformHostBackendRequestError,
    },
    OperationMismatch {
        expected: PlatformHostOperationId,
        received: PlatformHostOperationId,
    },
    InstanceIdExhausted,
    OperationIdExhausted,
    SnapshotGenerationExhausted,
}

impl fmt::Display for PlatformHostServiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyInstalled { state } => {
                write!(formatter, "platform host is already installed in {state:?}")
            }
            Self::NoHostInstalled => formatter.write_str("no platform host is installed"),
            Self::StaleInstance { expected, received } => write!(
                formatter,
                "stale platform host instance {} does not match active instance {}",
                received.raw(),
                expected.raw()
            ),
            Self::InvalidLifecycleState { operation, state } => {
                write!(
                    formatter,
                    "cannot {operation} while platform host is {state:?}"
                )
            }
            Self::BackendBridgeMissing => {
                formatter.write_str("installed platform host has no backend control bridge")
            }
            Self::BackendRejected { reason } => {
                write!(formatter, "platform host rejected request: {reason}")
            }
            Self::OperationMismatch { expected, received } => write!(
                formatter,
                "stale platform host operation {} does not match in-flight operation {}",
                received.raw(),
                expected.raw()
            ),
            Self::InstanceIdExhausted => {
                formatter.write_str("platform host instance IDs exhausted")
            }
            Self::OperationIdExhausted => {
                formatter.write_str("platform host operation IDs exhausted")
            }
            Self::SnapshotGenerationExhausted => {
                formatter.write_str("platform host snapshot generations exhausted")
            }
        }
    }
}

impl Error for PlatformHostServiceError {}
