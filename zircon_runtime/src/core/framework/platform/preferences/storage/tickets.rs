use std::fmt;
use std::sync::Arc;
use std::time::Instant;

use super::terminal::{
    PreferenceMutationCancelError, PreferenceMutationTerminal, PreferenceTicketWaitResult,
};

/// 与一次代际绑定的终态观察入口；新写入取代同键视图后，旧票据仍可观察其自身结果。
pub trait PreferenceMutationTicket: Send + Sync + fmt::Debug + 'static {
    fn generation(&self) -> u64;

    fn terminal(&self) -> Option<PreferenceMutationTerminal>;

    fn wait_until(&self, deadline: Instant) -> PreferenceTicketWaitResult;
}

/// 仅能在工作启动前取消，并须持有本次提交返回的取消权限；栅栏钉住时不可取消。
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

/// 前序已受理操作的完成栅栏；检查终态才能确认整体持久化结果。
pub trait PreferenceFlushTicket: Send + Sync + fmt::Debug + 'static {
    fn epoch(&self) -> u64;

    fn terminal(&self) -> Option<PreferenceMutationTerminal>;

    fn wait_until(&self, deadline: Instant) -> PreferenceTicketWaitResult;
}
