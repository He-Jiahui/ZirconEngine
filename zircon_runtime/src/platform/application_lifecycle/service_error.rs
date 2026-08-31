use std::error::Error;
use std::fmt;

use crate::core::framework::platform::{
    ApplicationLifecycleOperationId, ApplicationLifecycleState,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplicationLifecycleServiceError {
    InvalidState {
        operation: &'static str,
        state: ApplicationLifecycleState,
    },
    OperationMismatch {
        expected: ApplicationLifecycleOperationId,
        received: ApplicationLifecycleOperationId,
    },
    OperationIdExhausted,
    GenerationExhausted,
}

impl fmt::Display for ApplicationLifecycleServiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidState { operation, state } => {
                write!(
                    formatter,
                    "cannot {operation} while application lifecycle is {state:?}"
                )
            }
            Self::OperationMismatch { expected, received } => write!(
                formatter,
                "stale application lifecycle operation {} does not match active operation {}",
                received.raw(),
                expected.raw()
            ),
            Self::OperationIdExhausted => {
                formatter.write_str("application lifecycle operation IDs exhausted")
            }
            Self::GenerationExhausted => {
                formatter.write_str("application lifecycle snapshot generations exhausted")
            }
        }
    }
}

impl Error for ApplicationLifecycleServiceError {}
