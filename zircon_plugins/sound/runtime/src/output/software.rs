use zircon_runtime::core::framework::sound::{
    SoundBackendCapability, SoundOutputDeviceDescriptor, SoundOutputDeviceInfo,
};

use crate::SoundConfig;

pub(crate) const SOFTWARE_NULL_BACKEND: &str = "software-null";

pub(crate) fn software_backend_capabilities() -> Vec<SoundBackendCapability> {
    vec![SoundBackendCapability {
        backend: SOFTWARE_NULL_BACKEND.to_string(),
        display_name: "Deterministic Software Null Output".to_string(),
        realtime_capable: false,
        deterministic: true,
        min_sample_rate_hz: 1,
        max_sample_rate_hz: 384_000,
        min_channel_count: 1,
        max_channel_count: 64,
        min_block_size_frames: 1,
        max_block_size_frames: 65_536,
        notes: vec![
            "headless backend for tests and editor preview".to_string(),
            "pulls blocks from the software mixer without opening an OS device".to_string(),
        ],
    }]
}

pub(crate) fn supports_software_backend(backend: &str) -> bool {
    backend == SOFTWARE_NULL_BACKEND || backend.starts_with("software-")
}

pub(crate) fn software_output_devices(config: &SoundConfig) -> Vec<SoundOutputDeviceInfo> {
    vec![SoundOutputDeviceInfo {
        descriptor: SoundOutputDeviceDescriptor::software(
            SOFTWARE_NULL_BACKEND,
            config.sample_rate_hz,
            config.channel_count,
            config.block_size_frames,
        ),
        is_default: true,
        available: true,
        diagnostic: None,
    }]
}
