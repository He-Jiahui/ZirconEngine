use zircon_runtime::core::framework::sound::{SoundBackendCapability, SoundOutputDeviceInfo};

use crate::SoundConfig;

use super::{cpal, software};

pub(crate) fn available_output_backends() -> Vec<SoundBackendCapability> {
    let mut backends = software::software_backend_capabilities();
    backends.extend(cpal::cpal_backend_capabilities());
    backends
}

pub(crate) fn available_output_devices(config: &SoundConfig) -> Vec<SoundOutputDeviceInfo> {
    let mut devices = software::software_output_devices(config);
    devices.extend(cpal::cpal_output_devices(config));
    devices
}
