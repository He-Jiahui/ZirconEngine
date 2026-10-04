use std::sync::MutexGuard;

use crate::core::diagnostics::{DiagnosticPath, DiagnosticStore, DiagnosticStoreSnapshot};

use super::CoreHandle;

impl CoreHandle {
    /// 返回诊断存储的独立副本；修改副本不会回写运行时记录。
    pub fn diagnostic_store(&self) -> DiagnosticStore {
        self.lock_diagnostics().clone()
    }

    pub fn diagnostic_store_snapshot(&self) -> DiagnosticStoreSnapshot {
        self.lock_diagnostics().snapshot()
    }

    /// 记录跨子系统帧指标；路径和单位在取得诊断锁前完成转换，缩短共享写锁占用。
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
        let path: DiagnosticPath = path.into();
        let unit: Option<String> = unit.map(Into::into);
        self.lock_diagnostics()
            .record(path, frame_index, value, unit, subsystem_tags);
    }

    pub(crate) fn update_diagnostic_store<R>(
        &self,
        update: impl FnOnce(&mut DiagnosticStore) -> R,
    ) -> R {
        let mut store = self.lock_diagnostics();
        update(&mut store)
    }

    pub(super) fn lock_diagnostics(&self) -> MutexGuard<'_, DiagnosticStore> {
        self.inner
            .diagnostics
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[cfg(test)]
#[path = "tests/diagnostics.rs"]
mod tests;
