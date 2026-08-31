use std::fmt;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::Instant;

use super::contract::{
    NativePluginDiscoveryRefreshError, NativePluginDiscoveryRoot, NativePluginDiscoverySnapshot,
};

type TerminalObserver = Box<dyn FnOnce(NativePluginDiscoveryRefreshTerminal) + Send + 'static>;

#[derive(Clone, Debug)]
pub enum NativePluginDiscoveryRefreshTerminal {
    Published(Arc<NativePluginDiscoverySnapshot>),
    Superseded { generation: u64 },
    Cancelled,
    DeadlineExceeded,
    Shutdown,
    Rejected { reason: Arc<str> },
    Failed(Arc<NativePluginDiscoveryRefreshError>),
}

#[derive(Clone)]
pub struct NativePluginDiscoveryRefreshTicket {
    inner: Arc<TicketInner>,
}

#[derive(Debug)]
pub(super) struct NativePluginDiscoveryRefreshCancellation {
    cancelled: Arc<AtomicBool>,
    deadline: Instant,
}

impl Clone for NativePluginDiscoveryRefreshCancellation {
    fn clone(&self) -> Self {
        Self {
            cancelled: Arc::clone(&self.cancelled),
            deadline: self.deadline,
        }
    }
}

impl NativePluginDiscoveryRefreshCancellation {
    pub(super) fn new(deadline: Instant) -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
            deadline,
        }
    }

    pub(super) fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    pub(super) fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire) || Instant::now() >= self.deadline
    }

    pub(super) fn is_explicitly_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }

    pub(super) fn check_active(&self) -> Result<(), NativePluginDiscoveryRefreshError> {
        if self.cancelled.load(Ordering::Acquire) {
            return Err(NativePluginDiscoveryRefreshError::cancelled());
        }
        if Instant::now() >= self.deadline {
            return Err(NativePluginDiscoveryRefreshError::deadline_exceeded());
        }
        Ok(())
    }
}

struct TicketInner {
    root: NativePluginDiscoveryRoot,
    generation: u64,
    cancellation: NativePluginDiscoveryRefreshCancellation,
    max_observers: usize,
    state: Mutex<TicketState>,
    terminal_ready: Condvar,
    observer_panics: AtomicUsize,
}

struct TicketState {
    terminal: Option<NativePluginDiscoveryRefreshTerminal>,
    observers: Vec<TerminalObserver>,
    observer_count: usize,
}

pub(super) struct NativePluginDiscoveryRefreshTerminalDelivery {
    ticket: NativePluginDiscoveryRefreshTicket,
    terminal: NativePluginDiscoveryRefreshTerminal,
    observers: Vec<TerminalObserver>,
}

fn reserve_remaining_observer_budget_if_full(
    observers: &mut Vec<TerminalObserver>,
    max_observers: usize,
) {
    if !observers.is_empty() && observers.len() == observers.capacity() {
        let remaining_capacity = max_observers.saturating_sub(observers.len());
        observers.reserve_exact(remaining_capacity);
    }
}

impl NativePluginDiscoveryRefreshTicket {
    pub(super) fn new(
        root: NativePluginDiscoveryRoot,
        generation: u64,
        deadline: Instant,
        max_observers: usize,
    ) -> Self {
        Self {
            inner: Arc::new(TicketInner {
                root,
                generation,
                cancellation: NativePluginDiscoveryRefreshCancellation::new(deadline),
                max_observers,
                state: Mutex::new(TicketState {
                    terminal: None,
                    observers: Vec::new(),
                    observer_count: 0,
                }),
                terminal_ready: Condvar::new(),
                observer_panics: AtomicUsize::new(0),
            }),
        }
    }

    pub fn root(&self) -> &NativePluginDiscoveryRoot {
        &self.inner.root
    }

    pub fn generation(&self) -> u64 {
        self.inner.generation
    }

    pub fn cancel(&self) -> bool {
        self.inner.cancellation.cancel();
        self.finish(NativePluginDiscoveryRefreshTerminal::Cancelled)
    }

    pub fn is_complete(&self) -> bool {
        self.lock_state().terminal.is_some()
    }

    pub fn terminal(&self) -> Option<NativePluginDiscoveryRefreshTerminal> {
        self.lock_state().terminal.clone()
    }

    /// Waits without spinning until one terminal state has been committed or the ticket deadline
    /// makes the queued generation terminal.
    pub(crate) fn wait_terminal(&self) -> NativePluginDiscoveryRefreshTerminal {
        loop {
            let mut state = self.lock_state();
            if let Some(terminal) = state.terminal.clone() {
                return terminal;
            }

            let deadline = self.inner.cancellation.deadline;
            let now = Instant::now();
            if now >= deadline {
                drop(state);
                self.finish(NativePluginDiscoveryRefreshTerminal::DeadlineExceeded);
                continue;
            }

            let (next_state, wait_result) = self
                .inner
                .terminal_ready
                .wait_timeout(state, deadline.duration_since(now))
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state = next_state;
            if let Some(terminal) = state.terminal.clone() {
                return terminal;
            }
            drop(state);
            if wait_result.timed_out() {
                // A concurrent cancellation or completion may win this race. `finish` is
                // idempotent, and the next iteration returns whichever terminal was committed.
                self.finish(NativePluginDiscoveryRefreshTerminal::DeadlineExceeded);
            }
        }
    }

    /// Observers are bounded per ticket and invoked outside internal locks exactly once.
    pub fn on_terminal(
        &self,
        observer: impl FnOnce(NativePluginDiscoveryRefreshTerminal) + Send + 'static,
    ) -> bool {
        let observer: TerminalObserver = Box::new(observer);
        let terminal = {
            let mut state = self.lock_state();
            if state.observer_count >= self.inner.max_observers {
                return false;
            }
            state.observer_count += 1;
            if let Some(terminal) = state.terminal.clone() {
                terminal
            } else {
                reserve_remaining_observer_budget_if_full(
                    &mut state.observers,
                    self.inner.max_observers,
                );
                state.observers.push(observer);
                return true;
            }
        };
        self.run_observer(observer, terminal);
        true
    }

    pub fn terminal_observer_panic_count(&self) -> usize {
        self.inner.observer_panics.load(Ordering::Acquire)
    }

    pub(super) fn cancellation(&self) -> NativePluginDiscoveryRefreshCancellation {
        self.inner.cancellation.clone()
    }

    pub(super) fn finish(&self, terminal: NativePluginDiscoveryRefreshTerminal) -> bool {
        let Some(delivery) = self.reserve_terminal(terminal) else {
            return false;
        };
        delivery.deliver();
        true
    }

    /// Commits a terminal state without running observers. Callers that need a coupled state
    /// transition can publish their immutable result before delivering callbacks outside locks.
    pub(super) fn reserve_terminal(
        &self,
        terminal: NativePluginDiscoveryRefreshTerminal,
    ) -> Option<NativePluginDiscoveryRefreshTerminalDelivery> {
        let observers = {
            let mut state = self.lock_state();
            if state.terminal.is_some() {
                return None;
            }
            state.terminal = Some(terminal.clone());
            std::mem::take(&mut state.observers)
        };
        self.inner.terminal_ready.notify_all();
        Some(NativePluginDiscoveryRefreshTerminalDelivery {
            ticket: self.clone(),
            terminal,
            observers,
        })
    }

    fn lock_state(&self) -> MutexGuard<'_, TicketState> {
        self.inner
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn run_observer(
        &self,
        observer: TerminalObserver,
        terminal: NativePluginDiscoveryRefreshTerminal,
    ) {
        if catch_unwind(AssertUnwindSafe(|| observer(terminal))).is_err() {
            self.inner.observer_panics.fetch_add(1, Ordering::Release);
        }
    }
}

impl NativePluginDiscoveryRefreshTerminalDelivery {
    pub(super) fn deliver(self) {
        for observer in self.observers {
            self.ticket.run_observer(observer, self.terminal.clone());
        }
    }
}

impl fmt::Debug for NativePluginDiscoveryRefreshTicket {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NativePluginDiscoveryRefreshTicket")
            .field("root", self.root())
            .field("generation", &self.generation())
            .field("is_complete", &self.is_complete())
            .finish()
    }
}

#[cfg(test)]
mod capacity_tests {
    use super::*;

    const SATURATED_OBSERVER_BUDGET: usize = 32;

    fn observer() -> TerminalObserver {
        Box::new(|_| {})
    }

    #[test]
    fn observer_queue_reservation_preserves_small_fan_out_and_jumps_to_budget() {
        let mut observers = Vec::new();
        reserve_remaining_observer_budget_if_full(&mut observers, SATURATED_OBSERVER_BUDGET);
        assert_eq!(observers.capacity(), 0);

        observers.push(observer());
        while observers.len() < observers.capacity() {
            reserve_remaining_observer_budget_if_full(&mut observers, SATURATED_OBSERVER_BUDGET);
            observers.push(observer());
        }
        assert!(observers.capacity() < SATURATED_OBSERVER_BUDGET);

        reserve_remaining_observer_budget_if_full(&mut observers, SATURATED_OBSERVER_BUDGET);
        assert!(observers.capacity() >= SATURATED_OBSERVER_BUDGET);
    }
}
