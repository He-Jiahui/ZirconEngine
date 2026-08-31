mod scheduler;
mod snapshot;

pub(crate) use scheduler::{EventLoopDueSources, EventLoopScheduler};
pub(crate) use snapshot::{
    EventLoopHostWakeEvidence, EventLoopSchedulerSnapshot, EventLoopWakeDispatchEvidence,
};

#[cfg(test)]
mod tests;
