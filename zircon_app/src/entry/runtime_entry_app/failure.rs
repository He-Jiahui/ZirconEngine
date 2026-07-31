use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    sync::{Arc, Mutex},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::entry) struct RuntimeEntryAppFailure {
    component: &'static str,
    requested: String,
    cause: String,
    recovery: &'static str,
}

impl RuntimeEntryAppFailure {
    pub(super) fn new(
        component: &'static str,
        requested: impl Display,
        cause: impl Display,
        recovery: &'static str,
    ) -> Self {
        Self {
            component,
            requested: requested.to_string(),
            cause: cause.to_string(),
            recovery,
        }
    }
}

impl Display for RuntimeEntryAppFailure {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "runtime startup diagnostic: component={} requested={} cause={} recovery={}",
            self.component, self.requested, self.cause, self.recovery
        )
    }
}

impl Error for RuntimeEntryAppFailure {}

#[derive(Clone, Debug, Default)]
pub(in crate::entry) struct RuntimeEntryAppFailureState(Arc<Mutex<Option<RuntimeEntryAppFailure>>>);

impl RuntimeEntryAppFailureState {
    pub(super) fn record(&self, failure: RuntimeEntryAppFailure) {
        let mut recorded_failure = match self.0.lock() {
            Ok(recorded_failure) => recorded_failure,
            Err(poisoned) => poisoned.into_inner(),
        };
        if recorded_failure.is_none() {
            *recorded_failure = Some(failure);
        }
    }

    pub(super) fn is_recorded(&self) -> bool {
        let recorded_failure = match self.0.lock() {
            Ok(recorded_failure) => recorded_failure,
            Err(poisoned) => poisoned.into_inner(),
        };
        recorded_failure.is_some()
    }

    pub(in crate::entry) fn take(&self) -> Option<RuntimeEntryAppFailure> {
        let mut recorded_failure = match self.0.lock() {
            Ok(recorded_failure) => recorded_failure,
            Err(poisoned) => poisoned.into_inner(),
        };
        recorded_failure.take()
    }
}

#[cfg(test)]
mod tests {
    use super::{RuntimeEntryAppFailure, RuntimeEntryAppFailureState};

    #[test]
    fn runtime_entry_failure_uses_actionable_startup_diagnostic_fields() {
        let failure = RuntimeEntryAppFailure::new(
            "runtime_surface_present",
            "viewport=1 size=1280x720",
            "frame capture failed: device lost",
            "verify the graphics adapter and restart zircon_runtime",
        );

        assert_eq!(
            failure.to_string(),
            "runtime startup diagnostic: component=runtime_surface_present requested=viewport=1 size=1280x720 cause=frame capture failed: device lost recovery=verify the graphics adapter and restart zircon_runtime"
        );
    }

    #[test]
    fn runtime_entry_failure_state_retains_the_first_fatal_callback_failure() {
        let state = RuntimeEntryAppFailureState::default();
        assert!(
            !state.is_recorded(),
            "a fresh runtime entry failure state must allow host initialization"
        );
        state.record(RuntimeEntryAppFailure::new(
            "runtime_window",
            "primary_window",
            "window creation failed",
            "verify the desktop session and retry",
        ));
        state.record(RuntimeEntryAppFailure::new(
            "runtime_frame_loop",
            "runtime_session",
            "frame tick failed",
            "restart zircon_runtime",
        ));
        assert!(
            state.is_recorded(),
            "a terminal callback failure must remain visible until EntryRunner collects it"
        );

        assert_eq!(
            state.take().unwrap().to_string(),
            "runtime startup diagnostic: component=runtime_window requested=primary_window cause=window creation failed recovery=verify the desktop session and retry"
        );
        assert!(!state.is_recorded());
    }
}
