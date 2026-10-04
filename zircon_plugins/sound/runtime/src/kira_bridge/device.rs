//! CPAL device identity and exact format admission used by both the picker and Kira activation.
use kira::backend::cpal::{
    cpal::{
        self,
        traits::{DeviceTrait, HostTrait},
    },
    CpalBackendSettings,
};
use zircon_runtime::core::framework::audio::AudioChannelLayout;
use zircon_runtime::core::framework::sound::{
    SoundBackendCapability, SoundError, SoundOutputDeviceDescriptor, SoundOutputDeviceId,
    SoundOutputDeviceInfo,
};

use crate::SoundConfig;

pub(crate) const KIRA_CPAL_BACKEND: &str = "kira-cpal";

pub(crate) fn available_backends() -> Vec<SoundBackendCapability> {
    vec![SoundBackendCapability {
        backend: KIRA_CPAL_BACKEND.to_string(),
        display_name: "Kira CPAL Output".to_string(),
        realtime_capable: true,
        deterministic: false,
        min_sample_rate_hz: 8_000,
        max_sample_rate_hz: 384_000,
        min_channel_count: 1,
        max_channel_count: 2,
        supported_channel_layouts: vec![AudioChannelLayout::mono(), AudioChannelLayout::stereo()],
        min_block_size_frames: 1,
        max_block_size_frames: u16::MAX as usize,
        notes: vec![
            "Kira owns the audio thread and CPAL stream".to_string(),
            "v1 output is stereo; multichannel source downmix lands in Sound M4".to_string(),
        ],
    }]
}

pub(crate) fn available_devices(config: &SoundConfig) -> Vec<SoundOutputDeviceInfo> {
    let host = cpal::default_host();
    let default_id = host
        .default_output_device()
        .and_then(|device| stable_device_id(&device).ok());
    match host.output_devices() {
        Ok(devices) => devices
            .map(|device| device_info_for_device(device, config, default_id.as_deref()))
            .collect(),
        Err(error) => vec![SoundOutputDeviceInfo {
            descriptor: default_descriptor(config),
            is_default: true,
            available: false,
            diagnostic: Some(format!("Kira/CPAL device enumeration failed: {error}")),
        }],
    }
}

/// Check the exact stream request against the ranges CPAL reports for a device.
/// Kira's CPAL backend renders `f32`, so a non-f32 range cannot satisfy this request.
pub(crate) fn admit_exact_output_config(
    supported: impl IntoIterator<Item = cpal::SupportedStreamConfigRange>,
    requested: &cpal::StreamConfig,
) -> Result<(), SoundError> {
    let requested_buffer = match requested.buffer_size {
        cpal::BufferSize::Fixed(frames) => frames,
        cpal::BufferSize::Default => {
            return Err(SoundError::BackendUnavailable {
                detail: "Kira/CPAL exact admission requires a fixed output buffer".to_string(),
            })
        }
    };
    let admitted = supported.into_iter().any(|range| {
        range.channels() == requested.channels
            && range.min_sample_rate() <= requested.sample_rate
            && requested.sample_rate <= range.max_sample_rate()
            && range.sample_format() == cpal::SampleFormat::F32
            && matches!(
                range.buffer_size(),
                cpal::SupportedBufferSize::Range { min, max }
                    if *min <= requested_buffer && requested_buffer <= *max
            )
    });
    if admitted {
        Ok(())
    } else {
        Err(SoundError::BackendUnavailable {
            detail: format!(
                "Kira/CPAL device does not support exact f32 output {} Hz, {} channels, {} frames",
                requested.sample_rate, requested.channels, requested_buffer
            ),
        })
    }
}

pub(crate) fn device_identity_matches(requested: &str, actual: &str) -> bool {
    requested == actual
}

pub(super) fn backend_settings(
    descriptor: &SoundOutputDeviceDescriptor,
) -> Result<CpalBackendSettings, SoundError> {
    if descriptor.channel_count > 2 {
        return Err(SoundError::UnsupportedAdvancedFeature(
            "Kira v1 output supports mono or stereo only".to_string(),
        ));
    }
    if descriptor.block_size_frames > u32::MAX as usize {
        return Err(SoundError::InvalidParameter(
            "output block size exceeds CPAL's u32 frame limit".to_string(),
        ));
    }
    let requested = cpal::StreamConfig {
        channels: descriptor.channel_count,
        sample_rate: descriptor.sample_rate_hz,
        buffer_size: cpal::BufferSize::Fixed(descriptor.block_size_frames as u32),
    };
    let host = cpal::default_host();
    let (selected, custom_device) = if descriptor.id == SoundOutputDeviceId::default_system() {
        let device =
            host.default_output_device()
                .ok_or_else(|| SoundError::BackendUnavailable {
                    detail: "Kira/CPAL has no default output device".to_string(),
                })?;
        (device, false)
    } else {
        let wanted = descriptor
            .id
            .as_str()
            .strip_prefix(&format!("{KIRA_CPAL_BACKEND}:"))
            .ok_or_else(|| SoundError::BackendUnavailable {
                detail: format!(
                    "output device id `{}` is not a Kira/CPAL identity",
                    descriptor.id.as_str()
                ),
            })?;
        let selected = host
            .output_devices()
            .map_err(|error| SoundError::BackendUnavailable {
                detail: format!("Kira/CPAL device enumeration failed: {error}"),
            })?
            .find(|device| {
                stable_device_id(device)
                    .is_ok_and(|actual| device_identity_matches(wanted, &actual))
            })
            .ok_or_else(|| SoundError::BackendUnavailable {
                detail: format!("selected output device identity `{wanted}` is unavailable"),
            })?;
        (selected, true)
    };
    let supported =
        selected
            .supported_output_configs()
            .map_err(|error| SoundError::BackendUnavailable {
                detail: format!("Kira/CPAL supported output format query failed: {error}"),
            })?;
    admit_exact_output_config(supported, &requested)?;
    Ok(CpalBackendSettings {
        device: custom_device.then_some(selected),
        config: Some(requested),
    })
}

pub(crate) fn admit_output_descriptor(
    descriptor: &SoundOutputDeviceDescriptor,
) -> Result<(), SoundError> {
    let _ = backend_settings(descriptor)?;
    Ok(())
}

fn stable_device_id(device: &cpal::Device) -> Result<String, SoundError> {
    device
        .id()
        .map(|id| id.to_string())
        .map_err(|error| SoundError::BackendUnavailable {
            detail: format!("Kira/CPAL device identity query failed: {error}"),
        })
}

fn device_info_for_device(
    device: cpal::Device,
    config: &SoundConfig,
    default_id: Option<&str>,
) -> SoundOutputDeviceInfo {
    let display_name = device.to_string();
    let identity = stable_device_id(&device);
    let (id, is_default, identity_error) = match identity {
        Ok(identity) => (
            SoundOutputDeviceId::new(format!("{KIRA_CPAL_BACKEND}:{identity}")),
            default_id == Some(identity.as_str()),
            None,
        ),
        Err(error) => (
            SoundOutputDeviceId::new(format!("{KIRA_CPAL_BACKEND}:unavailable:{display_name}")),
            false,
            Some(error.to_string()),
        ),
    };
    let descriptor = SoundOutputDeviceDescriptor {
        id,
        backend: KIRA_CPAL_BACKEND.to_string(),
        display_name,
        sample_rate_hz: config.sample_rate_hz,
        channel_count: config.channel_count,
        channel_layout: config.channel_layout.clone(),
        block_size_frames: config.block_size_frames,
        latency_blocks: 2,
    };
    if let Some(error) = identity_error {
        return SoundOutputDeviceInfo {
            descriptor,
            is_default,
            available: false,
            diagnostic: Some(error),
        };
    }
    let requested = match stream_config_for_descriptor(&descriptor) {
        Ok(requested) => requested,
        Err(error) => {
            return SoundOutputDeviceInfo {
                descriptor,
                is_default,
                available: false,
                diagnostic: Some(error.to_string()),
            }
        }
    };
    let admission = device
        .supported_output_configs()
        .map_err(|error| SoundError::BackendUnavailable {
            detail: format!("Kira/CPAL supported output format query failed: {error}"),
        })
        .and_then(|supported| admit_exact_output_config(supported, &requested));
    SoundOutputDeviceInfo {
        descriptor,
        is_default,
        available: admission.is_ok(),
        diagnostic: admission.err().map(|error| error.to_string()),
    }
}

fn stream_config_for_descriptor(
    descriptor: &SoundOutputDeviceDescriptor,
) -> Result<cpal::StreamConfig, SoundError> {
    if descriptor.block_size_frames > u32::MAX as usize {
        return Err(SoundError::InvalidParameter(
            "output block size exceeds CPAL's u32 frame limit".to_string(),
        ));
    }
    Ok(cpal::StreamConfig {
        channels: descriptor.channel_count,
        sample_rate: descriptor.sample_rate_hz,
        buffer_size: cpal::BufferSize::Fixed(descriptor.block_size_frames as u32),
    })
}

fn default_descriptor(config: &SoundConfig) -> SoundOutputDeviceDescriptor {
    SoundOutputDeviceDescriptor {
        id: SoundOutputDeviceId::default_system(),
        backend: KIRA_CPAL_BACKEND.to_string(),
        display_name: "Default Output".to_string(),
        sample_rate_hz: config.sample_rate_hz,
        channel_count: config.channel_count,
        channel_layout: config.channel_layout.clone(),
        block_size_frames: config.block_size_frames,
        latency_blocks: 2,
    }
}

#[cfg(test)]
pub(crate) fn device_info_for_test(config: &SoundConfig) -> SoundOutputDeviceInfo {
    // This helper retains the catalog unit fixture; production enumeration never manufactures
    // an identity from a display name.
    SoundOutputDeviceInfo {
        descriptor: SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new(format!("{KIRA_CPAL_BACKEND}:test-fixture")),
            backend: KIRA_CPAL_BACKEND.to_string(),
            display_name: "Test Output".to_string(),
            sample_rate_hz: config.sample_rate_hz,
            channel_count: config.channel_count,
            channel_layout: config.channel_layout.clone(),
            block_size_frames: config.block_size_frames,
            latency_blocks: 2,
        },
        is_default: true,
        available: config.channel_count <= 2,
        diagnostic: (config.channel_count > 2).then(|| {
            format!(
                "{}-channel sound output is unavailable in Kira v1",
                config.channel_count
            )
        }),
    }
}
