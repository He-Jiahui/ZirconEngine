#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlatformHostFailureReason {
    BackendRejectedRequest,
    OwnerExited,
    QuiesceDeadlineElapsed,
    ProtocolViolation,
}
