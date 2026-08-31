/// A normalized platform-host failure recorded in a window command receipt.
/// Backends translate native errors into this contract instead of exposing
/// native objects or backend-specific error types across the runtime boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WindowCommandFailure {
    HostUnavailable,
    UnsupportedRequestedState,
    NativeOperationFailed,
}
