use std::sync::Arc;

use crate::builtin::{
    RuntimeModuleLoadReport, runtime_modules_for_target_with_plugin_registration_reports,
};
use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::ProjectPluginManifest;
use crate::plugin::{
    RuntimeExtensionCatalogReport, RuntimePluginCatalog, RuntimePluginRegistrationReport,
};

use super::error::{RuntimeDynamicSessionError, RuntimeDynamicSessionResult};

pub(super) struct LinkedRuntimePluginPlan {
    modules: RuntimeModuleLoadReport,
    extensions: Arc<RuntimeExtensionCatalogReport>,
    package_ids: Vec<String>,
}

impl LinkedRuntimePluginPlan {
    pub(super) fn prepare(
        registrations: &[RuntimePluginRegistrationReport],
        project_manifest: Option<&ProjectPluginManifest>,
        target_mode: RuntimeTargetMode,
    ) -> RuntimeDynamicSessionResult<Self> {
        let mut effective_manifest = project_manifest.cloned().unwrap_or_default();
        for registration in registrations {
            if effective_manifest
                .selections
                .iter()
                .all(|selection| selection.id != registration.project_selection.id)
            {
                effective_manifest
                    .selections
                    .push(registration.project_selection.clone());
            }
        }

        let modules = runtime_modules_for_target_with_plugin_registration_reports(
            target_mode,
            Some(&effective_manifest),
            registrations.iter(),
        );
        let catalog = RuntimePluginCatalog::from_registration_reports(
            registrations.iter().cloned(),
            std::iter::empty(),
        );
        let selected_package_ids = effective_manifest
            .enabled_for_target(target_mode)
            .map(|selection| selection.id.clone())
            .collect::<std::collections::BTreeSet<_>>();
        let extension_report =
            catalog.runtime_extensions_for_project(&effective_manifest, target_mode);
        let mut fatal_diagnostics = modules.fatal_messages();
        fatal_diagnostics.extend(extension_report.fatal_diagnostics.iter().cloned());
        if !fatal_diagnostics.is_empty() {
            fatal_diagnostics.sort();
            fatal_diagnostics.dedup();
            return Err(RuntimeDynamicSessionError::ModuleDiscovery {
                message: fatal_diagnostics.join("; "),
            });
        }

        Ok(Self {
            modules,
            extensions: extension_report,
            package_ids: registrations
                .iter()
                .filter(|registration| {
                    selected_package_ids.contains(&registration.project_selection.id)
                })
                .map(|registration| registration.package_manifest.id.clone())
                .collect(),
        })
    }

    pub(super) fn contains_package(&self, package_id: &str) -> bool {
        self.package_ids.iter().any(|id| id == package_id)
    }

    pub(super) fn into_parts(
        self,
    ) -> (RuntimeModuleLoadReport, Arc<RuntimeExtensionCatalogReport>) {
        (self.modules, self.extensions)
    }
}
