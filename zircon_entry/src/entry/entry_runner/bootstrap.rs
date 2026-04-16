use zircon_core::{CoreError, CoreHandle, CoreRuntime};

use crate::entry::{BuiltinEntryModuleSet, EntryConfig};

use super::EntryRunner;

impl EntryRunner {
    pub fn bootstrap(config: EntryConfig) -> Result<CoreHandle, CoreError> {
        let runtime = CoreRuntime::new();
        let handle = runtime.handle();
        let modules = BuiltinEntryModuleSet::for_profile(config.profile);

        for descriptor in modules.descriptors() {
            runtime.register_module(descriptor.clone())?;
        }
        for descriptor in modules.descriptors() {
            runtime.activate_module(&descriptor.name)?;
        }

        Ok(handle)
    }
}
