//! Domain-neutral bounded keyed I/O admission and execution lane.

#[path = "admission.rs"]
mod admission;
#[path = "diagnostics.rs"]
mod diagnostics;
#[path = "fence.rs"]
mod fence;
#[path = "key.rs"]
mod key;
#[path = "lane.rs"]
mod lane;
#[path = "ticket.rs"]
mod ticket;

pub use admission::{
    BoundedKeyedIoAdmission, BoundedKeyedIoAdmissionError, BoundedKeyedIoCancelAuthority,
    BoundedKeyedIoWork, BoundedKeyedIoWorkDeadline,
};
pub use diagnostics::{BoundedKeyedIoDiagnostics, BoundedKeyedIoShutdownReport};
pub use fence::{BoundedKeyedIoFence, GlobalAdmissionEpoch};
pub use key::BoundedKeyedIoKey;
pub use lane::{BoundedKeyedIoLane, BoundedKeyedIoLimits, BoundedKeyedIoShutdownGuard};
pub use ticket::{
    BoundedKeyedIoCancelError, BoundedKeyedIoFailure, BoundedKeyedIoTerminal, BoundedKeyedIoTicket,
    BoundedKeyedIoWaitResult,
};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
