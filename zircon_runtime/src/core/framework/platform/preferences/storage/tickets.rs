use std::fmt;
use std::sync::Arc;
use std::time::Instant;

use super::terminal::{
    PreferenceMutationCancelError, PreferenceMutationTerminal, PreferenceTicketWaitResult,
};

pub trait PreferenceMutationTicket: Send + Sync + fmt::Debug + 'static {
    fn generation(&self) -> u64;

    fn terminal(&self) -> Option<PreferenceMutationTerminal>;

    fn wait_until(&self, deadline: Instant) -> PreferenceTicketWaitResult;
}

pub trait PreferenceMutationCancellation: Send + Sync + fmt::Debug + 'static {
    fn cancel_before_start(&self) -> Result<(), PreferenceMutationCancelError>;
}

pub struct PreferenceMutationSubmission {
    ticket: Arc<dyn PreferenceMutationTicket>,
    cancellation: Arc<dyn PreferenceMutationCancellation>,
}

impl PreferenceMutationSubmission {
    pub(crate) fn new(
        ticket: Arc<dyn PreferenceMutationTicket>,
        cancellation: Arc<dyn PreferenceMutationCancellation>,
    ) -> Self {
        Self {
            ticket,
            cancellation,
        }
    }

    pub fn ticket(&self) -> Arc<dyn PreferenceMutationTicket> {
        Arc::clone(&self.ticket)
    }

    pub fn cancellation(&self) -> Arc<dyn PreferenceMutationCancellation> {
        Arc::clone(&self.cancellation)
    }
}

impl fmt::Debug for PreferenceMutationSubmission {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PreferenceMutationSubmission")
            .field("generation", &self.ticket.generation())
            .field("terminal", &self.ticket.terminal())
            .finish_non_exhaustive()
    }
}

pub trait PreferenceFlushTicket: Send + Sync + fmt::Debug + 'static {
    fn epoch(&self) -> u64;

    fn terminal(&self) -> Option<PreferenceMutationTerminal>;

    fn wait_until(&self, deadline: Instant) -> PreferenceTicketWaitResult;
}
