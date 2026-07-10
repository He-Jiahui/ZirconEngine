use std::sync::Arc;

use crate::core::CoreError;

use super::{bridge_dependencies::validate_bridge_dependency_closure, RuntimePluginCatalog};

impl RuntimePluginCatalog {
    pub(super) fn reject_module_order(&mut self, error: CoreError) {
        self.module_order_error = Some(Arc::new(error));
        self.rebuild_diagnostics();
    }

    pub(super) fn rebuild_diagnostics(&mut self) {
        self.diagnostics.clear();
        if let Some(error) = &self.module_order_error {
            self.diagnostics.push(format!(
                "runtime plugin module descriptor ordering failed: {error}"
            ));
        }
        for registration in &self.registrations {
            self.diagnostics
                .extend(registration.diagnostics.iter().cloned());
        }
        for registration in &self.feature_registrations {
            self.diagnostics
                .extend(registration.diagnostics.iter().cloned());
        }
        validate_bridge_dependency_closure(&self.registrations, &mut self.diagnostics);
    }
}
