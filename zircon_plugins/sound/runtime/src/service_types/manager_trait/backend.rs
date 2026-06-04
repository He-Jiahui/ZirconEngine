use zircon_runtime::core::framework::sound::{SoundBackendManager, SoundBackendStatus};

use super::super::DefaultSoundManager;

impl SoundBackendManager for DefaultSoundManager {
    fn backend_name(&self) -> String {
        self.backend_name_impl()
    }

    fn backend_status(&self) -> SoundBackendStatus {
        self.backend_status_impl()
    }
}
