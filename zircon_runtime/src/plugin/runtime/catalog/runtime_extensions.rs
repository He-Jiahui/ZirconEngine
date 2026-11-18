use std::sync::Arc;

use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::ProjectPluginManifest;
use crate::core::CoreHandle;

use super::extension_report::{runtime_extension_report, RuntimeExtensionCatalogReport};
use super::RuntimePluginCatalog;

impl RuntimePluginCatalog {
    pub fn runtime_extensions(&self) -> RuntimeExtensionCatalogReport {
        runtime_extension_report(&self.registrations, &[], None)
    }

    /// Merges unfiltered reports using host modules already registered in this Core.
    /// Only the dependencies reached from runtime manifest rows enter the sort;
    /// this does not freeze the Core, activate modules, or validate the whole graph.
    /// Runtime manifest names cannot duplicate any registered host name.
    ///
    /// Use `from_registration_reports` for host-dependent plugins: constructors
    /// that pre-order plugins without a host remain strict. The caller serializes
    /// assembly and must register the contributions in the same Core before activation.
    pub fn runtime_extensions_for_registered_host(
        &self,
        host: &CoreHandle,
    ) -> RuntimeExtensionCatalogReport {
        let host_descriptors = if self.registrations.is_empty() {
            Vec::new()
        } else {
            host.registered_module_descriptors()
        };
        runtime_extension_report(
            &self.registrations,
            &host_descriptors,
            self.module_order_error(),
        )
    }

    pub fn runtime_extensions_for_project(
        &self,
        manifest: &ProjectPluginManifest,
        target: RuntimeTargetMode,
    ) -> Arc<RuntimeExtensionCatalogReport> {
        Arc::clone(
            &self
                .compiled_project_plan(manifest, target)
                .extension_report,
        )
    }
}

#[cfg(test)]
#[path = "tests/runtime_extensions.rs"]
mod tests;
