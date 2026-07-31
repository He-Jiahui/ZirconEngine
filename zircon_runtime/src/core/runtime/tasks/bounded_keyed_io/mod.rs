//! Domain-neutral bounded keyed I/O admission and execution lane.

mod admission;
mod diagnostics;
mod fence;
mod lane;
mod ticket;

pub use admission::{
    BoundedKeyedIoAdmission, BoundedKeyedIoAdmissionError, BoundedKeyedIoCancelAuthority,
    BoundedKeyedIoWork, BoundedKeyedIoWorkDeadline,
};
pub use diagnostics::BoundedKeyedIoDiagnostics;
pub use fence::{BoundedKeyedIoFence, GlobalAdmissionEpoch};
pub use lane::{BoundedKeyedIoLane, BoundedKeyedIoLimits, BoundedKeyedIoShutdownGuard};
pub use ticket::{
    BoundedKeyedIoCancelError, BoundedKeyedIoFailure, BoundedKeyedIoTerminal, BoundedKeyedIoTicket,
    BoundedKeyedIoWaitResult,
};

#[cfg(test)]
mod tests;
