use std::sync::Arc;

use crate::ui::host::EditorPluginStatusReport;
use crate::ui::layouts::windows::workbench_host_window::ModulePluginsPaneViewData;

/// Derived retained-host presentation keyed by the immutable manager-owned status report.
///
/// This cache is not a plugin catalog authority. A new host status `Arc` naturally makes the
/// previous presentation unreachable, so the pane rebuilds exactly once for a new generation.
#[derive(Default)]
pub(in crate::ui::retained_host::app) struct ModulePluginPaneProjectionCache {
    cached: Option<CachedModulePluginPane>,
}

struct CachedModulePluginPane {
    status_report: Arc<EditorPluginStatusReport>,
    pane: ModulePluginsPaneViewData,
}

impl ModulePluginPaneProjectionCache {
    pub(in crate::ui::retained_host::app) fn get_or_build(
        &mut self,
        status_report: &Arc<EditorPluginStatusReport>,
        build: impl FnOnce(&EditorPluginStatusReport) -> ModulePluginsPaneViewData,
    ) -> ModulePluginsPaneViewData {
        if let Some(pane) = self.cached(status_report) {
            return pane;
        }

        let pane = build(status_report.as_ref());
        self.store(Arc::clone(status_report), pane.clone());
        pane
    }

    fn cached(
        &self,
        status_report: &Arc<EditorPluginStatusReport>,
    ) -> Option<ModulePluginsPaneViewData> {
        self.cached
            .as_ref()
            .filter(|cached| Arc::ptr_eq(&cached.status_report, status_report))
            .map(|cached| cached.pane.clone())
    }

    fn store(
        &mut self,
        status_report: Arc<EditorPluginStatusReport>,
        pane: ModulePluginsPaneViewData,
    ) {
        self.cached = Some(CachedModulePluginPane {
            status_report,
            pane,
        });
    }
}

#[cfg(test)]
#[path = "tests/cache.rs"]
mod tests;
