use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::Instant;

use super::BoundedKeyedIoCancelAuthority;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoundedKeyedIoFailure {
    pub code: &'static str,
}

impl BoundedKeyedIoFailure {
    pub const fn new(code: &'static str) -> Self {
        Self { code }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoundedKeyedIoTerminal {
    Succeeded,
    Failed(BoundedKeyedIoFailure),
    DeadlineBeforeStart,
    CancelledBeforeStart,
    Superseded { successor: u64 },
    Shutdown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoundedKeyedIoWaitResult {
    Terminal(BoundedKeyedIoTerminal),
    ObserverTimedOut,
}

#[derive(Clone, Debug)]
pub struct BoundedKeyedIoTicket {
    id: u64,
    generation: u64,
    state: Arc<TicketState>,
}

#[derive(Debug)]
struct TicketState {
    inner: Mutex<TicketStateInner>,
    changed: Condvar,
}

#[derive(Clone, Copy, Debug, Default)]
struct TicketStateInner {
    started: bool,
    fence_pinned: bool,
    terminal: Option<BoundedKeyedIoTerminal>,
}

impl BoundedKeyedIoTicket {
    pub(crate) fn pending(id: u64, generation: u64, fence_pinned: bool) -> Self {
        Self {
            id,
            generation,
            state: Arc::new(TicketState {
                inner: Mutex::new(TicketStateInner {
                    fence_pinned,
                    ..TicketStateInner::default()
                }),
                changed: Condvar::new(),
            }),
        }
    }

    pub const fn id(&self) -> u64 {
        self.id
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub fn terminal(&self) -> Option<BoundedKeyedIoTerminal> {
        self.lock().terminal
    }

    pub fn wait_until(&self, deadline: Instant) -> BoundedKeyedIoWaitResult {
        let mut state = self.lock();
        loop {
            if let Some(terminal) = state.terminal {
                return BoundedKeyedIoWaitResult::Terminal(terminal);
            }
            let now = Instant::now();
            if now >= deadline {
                return BoundedKeyedIoWaitResult::ObserverTimedOut;
            }
            state = self
                .state
                .changed
                .wait_timeout(state, deadline.saturating_duration_since(now))
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .0;
        }
    }

    pub fn cancel_before_start(
        &self,
        authority: &BoundedKeyedIoCancelAuthority,
    ) -> Result<(), BoundedKeyedIoCancelError> {
        if authority.ticket_id() != self.id {
            return Err(BoundedKeyedIoCancelError::WrongAuthority);
        }
        let mut state = self.lock();
        if state.fence_pinned {
            return Err(BoundedKeyedIoCancelError::FencePinned);
        }
        if state.started {
            return Err(BoundedKeyedIoCancelError::AlreadyStarted);
        }
        if state.terminal.is_none() {
            state.terminal = Some(BoundedKeyedIoTerminal::CancelledBeforeStart);
            self.state.changed.notify_all();
        }
        Ok(())
    }

    pub(crate) fn mark_started(&self) -> bool {
        let mut state = self.lock();
        if state.terminal.is_some() {
            return false;
        }
        state.started = true;
        true
    }

    pub(crate) fn mark_terminal(&self, terminal: BoundedKeyedIoTerminal) {
        let mut state = self.lock();
        if state.terminal.is_some() {
            return;
        }
        state.terminal = Some(terminal);
        self.state.changed.notify_all();
    }

    pub(crate) fn fence_pinned(&self) -> bool {
        self.lock().fence_pinned
    }

    pub(crate) fn pin_to_fence(&self) {
        self.lock().fence_pinned = true;
    }

    fn lock(&self) -> MutexGuard<'_, TicketStateInner> {
        self.state
            .inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoundedKeyedIoCancelError {
    WrongAuthority,
    AlreadyStarted,
    FencePinned,
}
