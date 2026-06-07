use super::super::storage::ModuleContributionParserState;
use super::{manifest, required};

impl ModuleContributionParserState {
    pub(in super::super::super) fn push_current_module(&mut self) {
        let Some(name) = self.current_name.take() else {
            return;
        };
        self.modules.push(manifest::module_contribution_manifest(
            name,
            required::take_required_module_kind(&mut self.current_kind),
            required::take_required_module_crate_name(&mut self.current_crate_name),
            std::mem::take(&mut self.current_target_modes),
            std::mem::take(&mut self.current_capabilities),
        ));
    }
}
