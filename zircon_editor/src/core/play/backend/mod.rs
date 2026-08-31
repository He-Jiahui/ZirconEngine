mod contract;
mod noop;
mod report;

pub use contract::{PlayBackend, SharedPlayBackend};
pub use noop::NoopPlayBackend;
#[cfg(test)]
pub(crate) use noop::TestAttachablePlayBackend;
pub use report::{
    PlayBackendPoll, PlayBackendRetireReport, PlayBackendStartFailure, PlayBackendStartReport,
    PlayBackendStopReport,
};
