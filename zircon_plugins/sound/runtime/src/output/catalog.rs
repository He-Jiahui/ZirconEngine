use zircon_runtime::core::framework::sound::{SoundBackendCapability, SoundOutputDeviceInfo};

use crate::kira_bridge;
use crate::SoundConfig;

pub(crate) fn available_output_backends() -> Vec<SoundBackendCapability> {
    kira_bridge::available_backends()
}

pub(crate) fn available_output_devices(config: &SoundConfig) -> Vec<SoundOutputDeviceInfo> {
    kira_bridge::available_devices(config)
}
