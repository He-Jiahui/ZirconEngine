use std::sync::{Arc, Mutex};

use super::RuntimeLibraryError;

#[derive(Clone, Debug, Default)]
pub(in crate::entry) struct RuntimeSessionTeardownFailureState(
    Arc<Mutex<Option<RuntimeLibraryError>>>,
);

impl RuntimeSessionTeardownFailureState {
    pub(super) fn record(&self, failure: RuntimeLibraryError) {
        let mut recorded_failure = match self.0.lock() {
            Ok(recorded_failure) => recorded_failure,
            Err(poisoned) => poisoned.into_inner(),
        };
        if recorded_failure.is_none() {
            *recorded_failure = Some(failure);
        }
    }

    pub(in crate::entry) fn take(&self) -> Option<RuntimeLibraryError> {
        let mut recorded_failure = match self.0.lock() {
            Ok(recorded_failure) => recorded_failure,
            Err(poisoned) => poisoned.into_inner(),
        };
        recorded_failure.take()
    }
}

#[cfg(test)]
mod tests {
    use super::{RuntimeLibraryError, RuntimeSessionTeardownFailureState};

    #[test]
    fn runtime_session_teardown_failure_state_retains_the_first_error() {
        let state = RuntimeSessionTeardownFailureState::default();
        state.record(RuntimeLibraryError::new("surface unbind failed"));
        state.record(RuntimeLibraryError::new("session destroy failed"));

        assert_eq!(state.take().unwrap().to_string(), "surface unbind failed");
        assert!(state.take().is_none());
    }
}
