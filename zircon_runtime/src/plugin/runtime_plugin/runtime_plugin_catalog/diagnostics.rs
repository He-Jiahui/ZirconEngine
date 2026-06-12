use super::{bridge_dependencies::validate_bridge_dependency_closure, RuntimePluginCatalog};

impl RuntimePluginCatalog {
    pub(super) fn rebuild_diagnostics(&mut self) {
        self.diagnostics.clear();
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
