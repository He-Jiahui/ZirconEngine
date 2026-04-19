use std::fmt;
use std::sync::Arc;

use zircon_runtime::core::{CoreError, CoreHandle, CoreRuntime, ModuleDescriptor};
use zircon_runtime::engine_module::EngineModule;

use super::{builtin_modules::builtin_modules_for_profile, EntryProfile};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryRunMode {
    Editor,
    Runtime,
    Headless,
}

impl From<EntryProfile> for EntryRunMode {
    fn from(value: EntryProfile) -> Self {
        match value {
            EntryProfile::Editor => Self::Editor,
            EntryProfile::Runtime => Self::Runtime,
            EntryProfile::Headless => Self::Headless,
        }
    }
}

pub trait EngineEntry: Send + Sync + fmt::Debug {
    fn profile(&self) -> EntryProfile;
    fn run_mode(&self) -> EntryRunMode;
    fn modules(&self) -> &[Arc<dyn EngineModule>];

    fn module_descriptors(&self) -> Vec<ModuleDescriptor> {
        self.modules()
            .iter()
            .map(|module| module.descriptor())
            .collect()
    }

    fn bootstrap(&self) -> Result<CoreHandle, CoreError> {
        let runtime = CoreRuntime::new();
        let descriptors = self.module_descriptors();

        for descriptor in &descriptors {
            runtime.register_module(descriptor.clone())?;
        }
        for descriptor in &descriptors {
            runtime.activate_module(&descriptor.name)?;
        }

        Ok(runtime.handle())
    }
}

#[derive(Clone, Debug)]
pub struct BuiltinEngineEntry {
    profile: EntryProfile,
    modules: Vec<Arc<dyn EngineModule>>,
}

impl BuiltinEngineEntry {
    pub fn for_profile(profile: EntryProfile) -> Self {
        Self {
            profile,
            modules: builtin_modules_for_profile(profile),
        }
    }
}

impl EngineEntry for BuiltinEngineEntry {
    fn profile(&self) -> EntryProfile {
        self.profile
    }

    fn run_mode(&self) -> EntryRunMode {
        self.profile.into()
    }

    fn modules(&self) -> &[Arc<dyn EngineModule>] {
        &self.modules
    }
}
