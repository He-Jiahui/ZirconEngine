use std::sync::Arc;

use crate::core::CoreError;

use super::derived_projection::RuntimePluginCatalogProjection;
use super::RuntimePluginCatalog;

pub(super) fn collect_catalog_diagnostics(
    module_order_error: Option<&CoreError>,
    registrations: &[crate::plugin::RuntimePluginRegistrationReport],
    feature_registrations: &[crate::plugin::RuntimePluginFeatureRegistrationReport],
    projection: &RuntimePluginCatalogProjection,
) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if let Some(error) = module_order_error {
        diagnostics.push(format!(
            "runtime plugin module descriptor ordering failed: {error}"
        ));
    }
    for registration in registrations {
        diagnostics.extend(registration.diagnostics.iter().cloned());
    }
    for registration in feature_registrations {
        diagnostics.extend(registration.diagnostics.iter().cloned());
    }
    diagnostics.extend(projection.feature_definition_diagnostics().iter().cloned());
    diagnostics.extend(projection.bridge_dependency_diagnostics().iter().cloned());
    diagnostics
}

impl RuntimePluginCatalog {
    pub(super) fn reject_module_order(&mut self, error: CoreError) {
        self.module_order_error = Some(Arc::new(error));
        self.rebuild_diagnostics();
    }

    pub(super) fn rebuild_diagnostics(&mut self) {
        self.diagnostics = collect_catalog_diagnostics(
            self.module_order_error.as_deref(),
            &self.registrations,
            &self.feature_registrations,
            self.projection.as_ref(),
        );
    }
}
