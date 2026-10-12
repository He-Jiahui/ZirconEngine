use crate::core::framework::window::{
    WindowCommandTerminal, WindowEffectiveState, WindowObservedState,
};

use super::WindowCommandFailure;

/// Native completion data returned by the platform thread. An applied native
/// operation must supply its actual effective state; non-applied results
/// cannot accidentally publish a requested state as an observed fact.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostWindowCommandCompletion {
    observed: WindowObservedState,
    effective: Option<WindowEffectiveState>,
    terminal: WindowCommandTerminal<WindowCommandFailure>,
}

impl HostWindowCommandCompletion {
    pub(crate) fn applied(observed: WindowObservedState, effective: WindowEffectiveState) -> Self {
        Self {
            observed,
            effective: Some(effective),
            terminal: WindowCommandTerminal::Applied,
        }
    }

    pub(crate) fn rejected(observed: WindowObservedState, reason: WindowCommandFailure) -> Self {
        Self {
            observed,
            effective: None,
            terminal: WindowCommandTerminal::Rejected { reason },
        }
    }

    pub(crate) fn failed(observed: WindowObservedState, reason: WindowCommandFailure) -> Self {
        Self {
            observed,
            effective: None,
            terminal: WindowCommandTerminal::Failed { reason },
        }
    }

    pub(crate) fn canceled(observed: WindowObservedState) -> Self {
        Self {
            observed,
            effective: None,
            terminal: WindowCommandTerminal::Canceled,
        }
    }

    pub(crate) const fn applies_effective_state(&self) -> bool {
        self.effective.is_some()
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        WindowObservedState,
        Option<WindowEffectiveState>,
        WindowCommandTerminal<WindowCommandFailure>,
    ) {
        (self.observed, self.effective, self.terminal)
    }
}
