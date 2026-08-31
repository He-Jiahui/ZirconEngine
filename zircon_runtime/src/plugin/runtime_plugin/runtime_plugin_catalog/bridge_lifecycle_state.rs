use std::sync::Arc;

use crate::core::framework::bridge::BridgeOwnerTransitionMode;
use crate::core::{RuntimeModuleLifecycleBlock, RuntimeModuleLifecycleObserver};
use crate::plugin::{BridgeTableDiagnosticsSummary, FrozenBridgeTable};

use super::{
    RuntimeExtensionCatalogReport, RuntimePluginBridgeLifecycleError,
    RuntimePluginBridgeLifecycleReport, RuntimePluginCatalog, RuntimePluginCatalogSnapshot,
};

#[derive(Clone, Debug)]
pub struct RuntimePluginBridgeLifecycleState {
    snapshot: Arc<RuntimePluginCatalogSnapshot>,
    extension_report: Arc<RuntimeExtensionCatalogReport>,
    bridge_table: FrozenBridgeTable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginBridgeLifecycleEvent {
    pub provider_package_id: String,
    pub mode: BridgeOwnerTransitionMode,
}

impl RuntimePluginBridgeLifecycleEvent {
    pub fn activate_provider(provider_package_id: impl Into<String>) -> Self {
        Self {
            provider_package_id: provider_package_id.into(),
            mode: BridgeOwnerTransitionMode::Activate,
        }
    }

    pub fn disable_provider(provider_package_id: impl Into<String>) -> Self {
        Self {
            provider_package_id: provider_package_id.into(),
            mode: BridgeOwnerTransitionMode::Disable,
        }
    }

    pub fn deactivate_provider(provider_package_id: impl Into<String>) -> Self {
        Self {
            provider_package_id: provider_package_id.into(),
            mode: BridgeOwnerTransitionMode::Deactivate,
        }
    }

    pub fn reload_provider(provider_package_id: impl Into<String>) -> Self {
        Self {
            provider_package_id: provider_package_id.into(),
            mode: BridgeOwnerTransitionMode::Reload,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimePluginBridgeLifecycleOutcome {
    Applied(RuntimePluginBridgeLifecycleReport),
    Blocked(RuntimePluginBridgeLifecycleError),
}

impl RuntimePluginBridgeLifecycleOutcome {
    pub fn is_applied(&self) -> bool {
        matches!(self, Self::Applied(_))
    }

    pub fn diagnostic(&self) -> String {
        match self {
            Self::Applied(report) => report.diagnostic(),
            Self::Blocked(error) => error.diagnostic(),
        }
    }
}

impl RuntimePluginBridgeLifecycleState {
    pub fn from_catalog(catalog: RuntimePluginCatalog) -> Self {
        Self::from_snapshot(Arc::new(RuntimePluginCatalogSnapshot::from_catalog(
            catalog,
        )))
    }

    pub fn from_extension_report(
        catalog: RuntimePluginCatalog,
        extension_report: Arc<RuntimeExtensionCatalogReport>,
    ) -> Self {
        Self::from_snapshot_and_extension_report(
            Arc::new(RuntimePluginCatalogSnapshot::from_catalog(catalog)),
            extension_report,
        )
    }

    pub fn from_snapshot(snapshot: Arc<RuntimePluginCatalogSnapshot>) -> Self {
        let extension_report = Arc::new(snapshot.catalog().runtime_extensions());
        Self::from_snapshot_and_extension_report(snapshot, extension_report)
    }

    pub fn from_snapshot_and_extension_report(
        snapshot: Arc<RuntimePluginCatalogSnapshot>,
        extension_report: Arc<RuntimeExtensionCatalogReport>,
    ) -> Self {
        let bridge_table = extension_report.registry.frozen_bridge_table();
        Self {
            snapshot,
            extension_report,
            bridge_table,
        }
    }

    pub fn snapshot(&self) -> &Arc<RuntimePluginCatalogSnapshot> {
        &self.snapshot
    }

    pub fn catalog(&self) -> &RuntimePluginCatalog {
        self.snapshot.catalog()
    }

    pub fn extension_report(&self) -> &RuntimeExtensionCatalogReport {
        self.extension_report.as_ref()
    }

    pub fn bridge_table(&self) -> &FrozenBridgeTable {
        &self.bridge_table
    }

    pub fn diagnostics_summary(&self) -> BridgeTableDiagnosticsSummary {
        self.bridge_table.diagnostics_summary()
    }

    pub fn provider_package_id_for_runtime_module(
        &self,
        runtime_module_name: &str,
    ) -> Option<String> {
        self.catalog()
            .provider_package_id_for_runtime_module(runtime_module_name)
    }

    pub fn apply_provider_lifecycle_event(
        &self,
        event: RuntimePluginBridgeLifecycleEvent,
    ) -> RuntimePluginBridgeLifecycleOutcome {
        match event.mode {
            BridgeOwnerTransitionMode::Activate => RuntimePluginBridgeLifecycleOutcome::Applied(
                self.activate_provider_at_frame_boundary(&event.provider_package_id),
            ),
            BridgeOwnerTransitionMode::Disable => self
                .disable_provider_at_frame_boundary(&event.provider_package_id)
                .map(RuntimePluginBridgeLifecycleOutcome::Applied)
                .unwrap_or_else(RuntimePluginBridgeLifecycleOutcome::Blocked),
            BridgeOwnerTransitionMode::Deactivate => self
                .deactivate_provider_at_frame_boundary(&event.provider_package_id)
                .map(RuntimePluginBridgeLifecycleOutcome::Applied)
                .unwrap_or_else(RuntimePluginBridgeLifecycleOutcome::Blocked),
            BridgeOwnerTransitionMode::Reload => RuntimePluginBridgeLifecycleOutcome::Applied(
                self.reload_provider_at_frame_boundary(&event.provider_package_id),
            ),
        }
    }

    pub fn activate_provider_at_frame_boundary(
        &self,
        provider_package_id: &str,
    ) -> RuntimePluginBridgeLifecycleReport {
        self.catalog().activate_bridge_provider_at_frame_boundary(
            &self.extension_report.registry,
            &self.bridge_table,
            provider_package_id,
        )
    }

    pub fn disable_provider_at_frame_boundary(
        &self,
        provider_package_id: &str,
    ) -> Result<RuntimePluginBridgeLifecycleReport, RuntimePluginBridgeLifecycleError> {
        self.catalog().disable_bridge_provider_at_frame_boundary(
            &self.extension_report.registry,
            &self.bridge_table,
            provider_package_id,
        )
    }

    pub fn deactivate_provider_at_frame_boundary(
        &self,
        provider_package_id: &str,
    ) -> Result<RuntimePluginBridgeLifecycleReport, RuntimePluginBridgeLifecycleError> {
        self.catalog().deactivate_bridge_provider_at_frame_boundary(
            &self.extension_report.registry,
            &self.bridge_table,
            provider_package_id,
        )
    }

    pub fn reload_provider_at_frame_boundary(
        &self,
        provider_package_id: &str,
    ) -> RuntimePluginBridgeLifecycleReport {
        self.catalog().reload_bridge_provider_at_frame_boundary(
            &self.extension_report.registry,
            &self.extension_report.registry,
            &self.bridge_table,
            provider_package_id,
        )
    }
}

impl RuntimeModuleLifecycleObserver for RuntimePluginBridgeLifecycleState {
    fn runtime_module_activated(&self, module_name: &str) {
        if let Some(provider_package_id) = self.provider_package_id_for_runtime_module(module_name)
        {
            self.activate_provider_at_frame_boundary(&provider_package_id);
        }
    }

    fn runtime_module_deactivating(
        &self,
        module_name: &str,
    ) -> Result<(), RuntimeModuleLifecycleBlock> {
        let Some(provider_package_id) = self.provider_package_id_for_runtime_module(module_name)
        else {
            return Ok(());
        };
        self.deactivate_provider_at_frame_boundary(&provider_package_id)
            .map(|_| ())
            .map_err(|error| RuntimeModuleLifecycleBlock::new(error.diagnostic()))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[test]
    fn bridge_lifecycle_state_keeps_shared_catalog_and_extension_report_snapshots() {
        let catalog = RuntimePluginCatalog::from_descriptors([]);
        let extension_report = Arc::new(catalog.runtime_extensions());
        let snapshot = Arc::new(RuntimePluginCatalogSnapshot::from_catalog(catalog));

        let state = RuntimePluginBridgeLifecycleState::from_snapshot_and_extension_report(
            Arc::clone(&snapshot),
            Arc::clone(&extension_report),
        );
        let cloned = state.clone();

        assert!(Arc::ptr_eq(&snapshot, state.snapshot()));
        assert!(Arc::ptr_eq(&extension_report, &state.extension_report));
        assert!(Arc::ptr_eq(state.snapshot(), cloned.snapshot()));
        assert!(Arc::ptr_eq(
            &state.extension_report,
            &cloned.extension_report
        ));
    }
}
