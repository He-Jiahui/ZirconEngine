use crate::core::diagnostics::{DiagnosticPath, DiagnosticStore, DiagnosticStoreSnapshot};

use super::CoreHandle;

impl CoreHandle {
    pub fn diagnostic_store(&self) -> DiagnosticStore {
        self.inner.diagnostics.lock().unwrap().clone()
    }

    pub fn diagnostic_store_snapshot(&self) -> DiagnosticStoreSnapshot {
        self.inner.diagnostics.lock().unwrap().snapshot()
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
        self.inner.diagnostics.lock().unwrap().record(
            path,
            frame_index,
            value,
            unit,
            subsystem_tags,
        );
    }
}
