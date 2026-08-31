use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use super::{ProductFailureLedger, ProductFailureReport, ProductHostPhase, ProductTerminalReason};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProductShutdownTransitionError {
    InvalidTransition {
        from: ProductHostPhase,
        to: ProductHostPhase,
    },
}

impl Display for ProductShutdownTransitionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTransition { from, to } => write!(
                formatter,
                "invalid product shutdown transition from {} to {}",
                from.as_str(),
                to.as_str()
            ),
        }
    }
}

impl Error for ProductShutdownTransitionError {}

/// Evidence for how one product-host shutdown phase was satisfied.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProductShutdownPhaseDisposition {
    Executed,
    NoOwner,
    LegacyCombined,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ProductShutdownTransition {
    from: ProductHostPhase,
    to: ProductHostPhase,
    disposition: ProductShutdownPhaseDisposition,
    phase_elapsed: Duration,
    total_elapsed: Duration,
}

impl ProductShutdownTransition {
    pub(crate) const fn from(&self) -> ProductHostPhase {
        self.from
    }

    pub(crate) const fn to(&self) -> ProductHostPhase {
        self.to
    }

    pub(crate) const fn disposition(&self) -> ProductShutdownPhaseDisposition {
        self.disposition
    }

    pub(crate) const fn phase_elapsed(&self) -> Duration {
        self.phase_elapsed
    }

    pub(crate) const fn total_elapsed(&self) -> Duration {
        self.total_elapsed
    }
}

#[derive(Debug)]
struct ProductShutdownState {
    phase: ProductHostPhase,
    terminal_reason: Option<ProductTerminalReason>,
    started_at: Instant,
    phase_started_at: Instant,
    transitions: Vec<ProductShutdownTransition>,
}

impl Default for ProductShutdownState {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            phase: ProductHostPhase::Composing,
            terminal_reason: None,
            started_at: now,
            phase_started_at: now,
            transitions: Vec::with_capacity(ProductHostPhase::COUNT - 1),
        }
    }
}

/// Cold-path authority for one product generation's terminal reason and shutdown phases.
#[derive(Clone, Debug, Default)]
pub(crate) struct ProductShutdownCoordinator {
    state: Arc<Mutex<ProductShutdownState>>,
    failures: ProductFailureLedger,
}

impl ProductShutdownCoordinator {
    pub(crate) fn mark_running(&self) -> Result<(), ProductShutdownTransitionError> {
        self.advance_to(ProductHostPhase::Running)
    }

    pub(crate) fn request_stop(
        &self,
        reason: ProductTerminalReason,
    ) -> Result<(), ProductShutdownTransitionError> {
        self.request_stop_with_disposition(reason, ProductShutdownPhaseDisposition::Executed)
    }

    pub(crate) fn request_stop_with_disposition(
        &self,
        reason: ProductTerminalReason,
        disposition: ProductShutdownPhaseDisposition,
    ) -> Result<(), ProductShutdownTransitionError> {
        let mut state = self.lock();
        if state.terminal_reason.is_none() {
            state.terminal_reason = Some(reason);
        }
        match state.phase {
            ProductHostPhase::Composing | ProductHostPhase::Running => {
                transition_to(&mut state, ProductHostPhase::Quiescing, disposition);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub(crate) fn advance_to(
        &self,
        phase: ProductHostPhase,
    ) -> Result<(), ProductShutdownTransitionError> {
        self.advance_to_with_disposition(phase, ProductShutdownPhaseDisposition::Executed)
    }

    pub(crate) fn advance_to_with_disposition(
        &self,
        phase: ProductHostPhase,
        disposition: ProductShutdownPhaseDisposition,
    ) -> Result<(), ProductShutdownTransitionError> {
        let mut state = self.lock();
        if state.phase == phase {
            return Ok(());
        }
        if state.phase == ProductHostPhase::Composing && phase == ProductHostPhase::Running {
            transition_to(&mut state, phase, disposition);
            return Ok(());
        }
        if state.terminal_reason.is_some() && state.phase.next_shutdown_phase() == Some(phase) {
            transition_to(&mut state, phase, disposition);
            return Ok(());
        }
        Err(ProductShutdownTransitionError::InvalidTransition {
            from: state.phase,
            to: phase,
        })
    }

    pub(crate) fn failure_ledger(&self) -> ProductFailureLedger {
        self.failures.clone()
    }

    pub(crate) fn snapshot(&self) -> ProductShutdownSnapshot {
        let (phase, terminal_reason, transitions) = {
            let state = self.lock();
            (
                state.phase,
                state.terminal_reason,
                state.transitions.clone(),
            )
        };
        ProductShutdownSnapshot {
            phase,
            terminal_reason,
            transitions,
            failures: self.failures.snapshot(),
        }
    }

    fn lock(&self) -> MutexGuard<'_, ProductShutdownState> {
        match self.state.lock() {
            Ok(state) => state,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}

fn transition_to(
    state: &mut ProductShutdownState,
    phase: ProductHostPhase,
    disposition: ProductShutdownPhaseDisposition,
) {
    let from = state.phase;
    let now = Instant::now();
    state.transitions.push(ProductShutdownTransition {
        from,
        to: phase,
        disposition,
        phase_elapsed: now.saturating_duration_since(state.phase_started_at),
        total_elapsed: now.saturating_duration_since(state.started_at),
    });
    state.phase = phase;
    state.phase_started_at = now;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProductShutdownSnapshot {
    phase: ProductHostPhase,
    terminal_reason: Option<ProductTerminalReason>,
    transitions: Vec<ProductShutdownTransition>,
    failures: ProductFailureReport,
}

impl ProductShutdownSnapshot {
    pub(crate) const fn phase(&self) -> ProductHostPhase {
        self.phase
    }

    pub(crate) const fn terminal_reason(&self) -> Option<ProductTerminalReason> {
        self.terminal_reason
    }

    pub(crate) fn transitions(&self) -> &[ProductShutdownTransition] {
        &self.transitions
    }

    pub(crate) const fn failures(&self) -> &ProductFailureReport {
        &self.failures
    }
}
