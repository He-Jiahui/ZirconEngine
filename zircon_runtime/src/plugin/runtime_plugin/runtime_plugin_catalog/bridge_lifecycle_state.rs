use crate::plugin::{BridgeOwnerTransitionMode, BridgeTableDiagnosticsSummary, FrozenBridgeTable};

use super::{
    RuntimeExtensionCatalogReport, RuntimePluginBridgeLifecycleError,
    RuntimePluginBridgeLifecycleReport, RuntimePluginCatalog,
};

#[derive(Clone, Debug)]
pub struct RuntimePluginBridgeLifecycleState {
    catalog: RuntimePluginCatalog,
    extension_report: RuntimeExtensionCatalogReport,
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
        let extension_report = catalog.runtime_extensions();
        Self::from_extension_report(catalog, extension_report)
    }

    pub fn from_extension_report(
        catalog: RuntimePluginCatalog,
        extension_report: RuntimeExtensionCatalogReport,
    ) -> Self {
        let bridge_table = extension_report.registry.frozen_bridge_table();
        Self {
            catalog,
            extension_report,
            bridge_table,
        }
    }

    pub fn catalog(&self) -> &RuntimePluginCatalog {
        &self.catalog
    }

    pub fn extension_report(&self) -> &RuntimeExtensionCatalogReport {
        &self.extension_report
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
        self.catalog
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
        self.catalog.activate_bridge_provider_at_frame_boundary(
            &self.extension_report.registry,
            &self.bridge_table,
            provider_package_id,
        )
    }

    pub fn disable_provider_at_frame_boundary(
        &self,
        provider_package_id: &str,
    ) -> Result<RuntimePluginBridgeLifecycleReport, RuntimePluginBridgeLifecycleError> {
        self.catalog.disable_bridge_provider_at_frame_boundary(
            &self.extension_report.registry,
            &self.bridge_table,
            provider_package_id,
        )
    }

    pub fn deactivate_provider_at_frame_boundary(
        &self,
        provider_package_id: &str,
    ) -> Result<RuntimePluginBridgeLifecycleReport, RuntimePluginBridgeLifecycleError> {
        self.catalog.deactivate_bridge_provider_at_frame_boundary(
            &self.extension_report.registry,
            &self.bridge_table,
            provider_package_id,
        )
    }

    pub fn reload_provider_at_frame_boundary(
        &self,
        provider_package_id: &str,
    ) -> RuntimePluginBridgeLifecycleReport {
        self.catalog.reload_bridge_provider_at_frame_boundary(
            &self.extension_report.registry,
            &self.extension_report.registry,
            &self.bridge_table,
            provider_package_id,
        )
    }
}
