use std::sync::Arc;

use zircon_core::ModuleDescriptor;
use zircon_module::EngineModule;

use super::super::entry_profile::EntryProfile;
use super::{editor_modules::editor_modules, runtime_modules::runtime_modules};

#[derive(Clone, Debug)]
pub struct BuiltinEntryModuleSet {
    modules: Vec<Arc<dyn EngineModule>>,
}

impl BuiltinEntryModuleSet {
    pub fn for_profile(profile: EntryProfile) -> Self {
        let mut modules = runtime_modules();
        if matches!(profile, EntryProfile::Editor) {
            modules.extend(editor_modules());
        }

        Self { modules }
    }

    pub fn modules(&self) -> &[Arc<dyn EngineModule>] {
        &self.modules
    }

    pub fn descriptors(&self) -> Vec<ModuleDescriptor> {
        self.modules
            .iter()
            .map(|module| module.descriptor())
            .collect()
    }
}
