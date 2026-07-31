use woc_runtime::PresentationTimelineError;

use super::ClientMovementInputError;

#[derive(Debug, PartialEq)]
pub enum ClientFrameDriverInitError {
    ZeroCatchUpBudget,
    Movement(ClientMovementInputError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClientCommandQueueError {
    Full { maximum: usize },
}

#[derive(Debug, PartialEq)]
pub enum ClientFrameDriverError<E> {
    Authority(E),
    Timeline(PresentationTimelineError),
    Movement(ClientMovementInputError),
}
