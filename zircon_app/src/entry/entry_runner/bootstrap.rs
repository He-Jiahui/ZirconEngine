use zircon_runtime::core::{CoreError, CoreHandle};

use crate::entry::{BuiltinEngineEntry, EngineEntry, EntryConfig};

use super::EntryRunner;

impl EntryRunner {
    pub fn bootstrap(config: EntryConfig) -> Result<CoreHandle, CoreError> {
        BuiltinEngineEntry::for_profile(config.profile).bootstrap()
    }
}
