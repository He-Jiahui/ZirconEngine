use std::error::Error;
use std::fmt;

use crate::core::framework::window::{WindowCommandId, WindowId, WindowRequestedGeneration};

/// Typed admission and state-machine failures for one platform-host command
/// broker. Native operation failures belong in terminal receipts instead.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HostCommandBrokerError {
    OutstandingLimitReached {
        limit: usize,
    },
    RequestIdExhausted,
    RequestedGenerationExhausted {
        window: WindowId,
        current: WindowRequestedGeneration,
    },
    AllocationFailed,
    SnapshotTargetMismatch {
        expected: WindowId,
        actual: WindowId,
    },
    DuplicateInFlightRequest {
        request_id: WindowCommandId,
    },
    DuplicateTerminalReceipt {
        request_id: WindowCommandId,
    },
    UnknownInFlightRequest {
        request_id: WindowCommandId,
    },
}

impl fmt::Display for HostCommandBrokerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutstandingLimitReached { limit } => write!(
                formatter,
                "host command broker cannot admit more than {limit} outstanding window commands"
            ),
            Self::RequestIdExhausted => formatter
                .write_str("host command broker exhausted window command request identifiers"),
            Self::RequestedGenerationExhausted { window, current } => write!(
                formatter,
                "host command broker cannot advance requested state generation {} for window {}:{}:{}",
                current.get(),
                window.registry().raw(),
                window.slot(),
                window.generation()
            ),
            Self::AllocationFailed => {
                formatter.write_str("host command broker could not reserve command state")
            }
            Self::SnapshotTargetMismatch { expected, actual } => write!(
                formatter,
                "host command broker expected state snapshot for window {}:{}:{}, received {}:{}:{}",
                expected.registry().raw(),
                expected.slot(),
                expected.generation(),
                actual.registry().raw(),
                actual.slot(),
                actual.generation()
            ),
            Self::DuplicateInFlightRequest { request_id } => write!(
                formatter,
                "host command broker request {} is already in flight",
                request_id.raw()
            ),
            Self::DuplicateTerminalReceipt { request_id } => write!(
                formatter,
                "host command broker request {} already has a terminal receipt",
                request_id.raw()
            ),
            Self::UnknownInFlightRequest { request_id } => write!(
                formatter,
                "host command broker request {} is not in flight",
                request_id.raw()
            ),
        }
    }
}

impl Error for HostCommandBrokerError {}
