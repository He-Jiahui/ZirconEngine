use woc_protocol::ProtocolError;
use woc_runtime::PresentationTimelineError;

use super::ClientMovementInputError;

#[derive(Debug, PartialEq)]
pub enum ClientFrameDriverInitError {
    ZeroCatchUpBudget,
    Movement(ClientMovementInputError),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ClientCommandQueueError {
    Full { maximum: usize },
    Command(ProtocolError),
}

#[derive(Debug, PartialEq)]
pub enum ClientFrameDriverError<E> {
    ElapsedTooLarge {
        elapsed_ns: u64,
        maximum_ns: u64,
    },
    Authority(E),
    AuthorityRollback {
        source: E,
        rollback: E,
    },
    Timeline(PresentationTimelineError),
    TimelineRollback {
        timeline: PresentationTimelineError,
        rollback: E,
    },
    Movement(ClientMovementInputError),
}
