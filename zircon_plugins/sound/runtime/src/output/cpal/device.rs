use zircon_runtime::core::framework::sound::SoundOutputDeviceInfo;
#[cfg(feature = "cpal-backend")]
use zircon_runtime::core::framework::sound::{SoundOutputDeviceDescriptor, SoundOutputDeviceId};

use crate::SoundConfig;

#[cfg(feature = "cpal-backend")]
use super::selection::select_stream_config;
#[cfg(feature = "cpal-backend")]
use super::{CPAL_BACKEND, CPAL_DEFAULT_OUTPUT_DEVICE_ID, CPAL_OUTPUT_DEVICE_ID_PREFIX};

#[cfg(feature = "cpal-backend")]
pub(crate) fn cpal_output_devices(config: &SoundConfig) -> Vec<SoundOutputDeviceInfo> {
    use cpal::traits::HostTrait;

    let host = cpal::default_host();
    let mut devices = Vec::new();
    match host.default_output_device() {
        Some(device) => devices.push(output_device_info(
            SoundOutputDeviceId::new(CPAL_DEFAULT_OUTPUT_DEVICE_ID),
            &device,
            true,
            config,
        )),
        None => devices.push(unavailable_default_output_device(config)),
    }

    match host.output_devices() {
        Ok(output_devices) => {
            for (index, device) in output_devices.enumerate() {
                devices.push(output_device_info(
                    SoundOutputDeviceId::new(format!("{CPAL_OUTPUT_DEVICE_ID_PREFIX}{index}")),
                    &device,
                    false,
                    config,
                ));
            }
        }
        Err(error) => {
            if let Some(default) = devices.first_mut() {
                default.available = false;
                default.diagnostic =
                    Some(format!("cpal output device enumeration failed: {error}"));
            }
        }
    }
    devices
}

#[cfg(not(feature = "cpal-backend"))]
pub(crate) fn cpal_output_devices(_config: &SoundConfig) -> Vec<SoundOutputDeviceInfo> {
    Vec::new()
}

#[cfg(feature = "cpal-backend")]
fn output_device_info(
    id: SoundOutputDeviceId,
    device: &cpal::Device,
    is_default: bool,
    config: &SoundConfig,
) -> SoundOutputDeviceInfo {
    use cpal::traits::DeviceTrait;

    let display_name = device
        .name()
        .unwrap_or_else(|_| "CPAL Output Device".to_string());
    let mut descriptor = SoundOutputDeviceDescriptor {
        id,
        backend: CPAL_BACKEND.to_string(),
        display_name,
        sample_rate_hz: config.sample_rate_hz,
        channel_count: config.channel_count,
        block_size_frames: config.block_size_frames,
        latency_blocks: zircon_runtime::core::framework::sound::DEFAULT_SOUND_OUTPUT_LATENCY_BLOCKS,
    };
    if let Ok(default_config) = device.default_output_config() {
        descriptor.sample_rate_hz = default_config.sample_rate().0;
        descriptor.channel_count = default_config.channels();
    }
    let diagnostic = select_stream_config(device, &descriptor)
        .err()
        .map(|error| error.to_string());
    SoundOutputDeviceInfo {
        descriptor,
        is_default,
        available: diagnostic.is_none(),
        diagnostic,
    }
}

#[cfg(feature = "cpal-backend")]
fn unavailable_default_output_device(config: &SoundConfig) -> SoundOutputDeviceInfo {
    SoundOutputDeviceInfo {
        descriptor: SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new(CPAL_DEFAULT_OUTPUT_DEVICE_ID),
            backend: CPAL_BACKEND.to_string(),
            display_name: "CPAL Default Output".to_string(),
            sample_rate_hz: config.sample_rate_hz,
            channel_count: config.channel_count,
            block_size_frames: config.block_size_frames,
            latency_blocks:
                zircon_runtime::core::framework::sound::DEFAULT_SOUND_OUTPUT_LATENCY_BLOCKS,
        },
        is_default: true,
        available: false,
        diagnostic: Some("cpal default output device is not available".to_string()),
    }
}
