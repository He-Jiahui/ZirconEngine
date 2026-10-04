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
    fence_pins: usize,
    terminal: Option<BoundedKeyedIoTerminal>,
}

impl BoundedKeyedIoTicket {
    pub(crate) fn pending(id: u64, generation: u64, fence_pinned: bool) -> Self {
        Self {
            id,
            generation,
            state: Arc::new(TicketState {
                inner: Mutex::new(TicketStateInner {
                    fence_pins: usize::from(fence_pinned),
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

    pub(super) fn same_instance(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.state, &other.state)
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

    /// 取消权威绑定票据状态的 Arc 身份而非可复用数值 id；started 或 fence pin 状态会拒绝绕过前置义务的取消。
    pub fn cancel_before_start(
        &self,
        authority: &BoundedKeyedIoCancelAuthority,
    ) -> Result<(), BoundedKeyedIoCancelError> {
        if !authority.authorizes(self) {
            return Err(BoundedKeyedIoCancelError::WrongAuthority);
        }
        let mut state = self.lock();
        if state.terminal == Some(BoundedKeyedIoTerminal::CancelledBeforeStart) {
            return Ok(());
        }
        if state.terminal.is_some() {
            return Err(BoundedKeyedIoCancelError::AlreadyStarted);
        }
        if state.fence_pins != 0 {
            return Err(BoundedKeyedIoCancelError::FencePinned);
        }
        if state.started {
            return Err(BoundedKeyedIoCancelError::AlreadyStarted);
        }
        state.terminal = Some(BoundedKeyedIoTerminal::CancelledBeforeStart);
        self.state.changed.notify_all();
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

    pub(crate) fn mark_terminal(&self, terminal: BoundedKeyedIoTerminal) -> bool {
        let mut state = self.lock();
        if state.terminal.is_some() {
            return false;
        }
        state.terminal = Some(terminal);
        self.state.changed.notify_all();
        true
    }

    pub(crate) fn mark_terminal_before_start(&self, terminal: BoundedKeyedIoTerminal) -> bool {
        let mut state = self.lock();
        if state.started || state.terminal.is_some() {
            return false;
        }
        state.terminal = Some(terminal);
        self.state.changed.notify_all();
        true
    }

    pub(crate) fn fence_pinned(&self) -> bool {
        self.lock().fence_pins != 0
    }

    pub(crate) fn pin_to_fence(&self) {
        let mut state = self.lock();
        state.fence_pins = state.fence_pins.saturating_add(1);
    }

    pub(crate) fn unpin_from_fence(&self) {
        let mut state = self.lock();
        state.fence_pins = state.fence_pins.saturating_sub(1);
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

#[cfg(test)]
#[path = "ticket/tests/astra_authority_tests.rs"]
mod astra_authority_tests;
