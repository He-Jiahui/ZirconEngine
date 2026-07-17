use std::sync::MutexGuard;

use crate::core::diagnostics::{DiagnosticPath, DiagnosticStore, DiagnosticStoreSnapshot};

use super::CoreHandle;

impl CoreHandle {
    pub fn diagnostic_store(&self) -> DiagnosticStore {
        self.lock_diagnostics().clone()
    }

    pub fn diagnostic_store_snapshot(&self) -> DiagnosticStoreSnapshot {
        self.lock_diagnostics().snapshot()
    }

    pub fn record_diagnostic<U, T>(
        &self,
        path: impl Into<DiagnosticPath>,
        frame_index: u64,
        value: f64,
        unit: Option<U>,
        subsystem_tags: impl IntoIterator<Item = T>,
    ) where
        U: Into<String>,
        T: Into<String>,
    {
        self.lock_diagnostics()
            .record(path, frame_index, value, unit, subsystem_tags);
    }

    pub(super) fn lock_diagnostics(&self) -> MutexGuard<'_, DiagnosticStore> {
        self.inner
            .diagnostics
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{self, AssertUnwindSafe};

    use crate::core::CoreRuntime;

    #[test]
    fn core_handle_diagnostic_accessors_recover_poisoned_store_lock() {
        let runtime = CoreRuntime::new();
        let handle = runtime.handle();

        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = handle.inner.diagnostics.lock().unwrap();
            panic!("poison core handle diagnostics store");
        }));

        handle.record_diagnostic("runtime.poisoned", 7, 42.0, Some("count"), ["runtime"]);

        let store = handle.diagnostic_store();
        let snapshot = store.snapshot();
        assert_eq!(snapshot.series.len(), 1);
        let series = &snapshot.series[0];
        assert_eq!(series.path.as_str(), "runtime.poisoned");
        assert_eq!(series.current, Some(42.0));
        assert_eq!(series.unit.as_deref(), Some("count"));
        assert_eq!(series.subsystem_tags, ["runtime"]);

        let snapshot = handle.diagnostic_store_snapshot();
        assert_eq!(snapshot.series.len(), 1);
    }
}
