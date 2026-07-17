use kira::backend::cpal::{
    cpal::{self, traits::HostTrait},
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
    let default_name = host
        .default_output_device()
        .map(|device| device.to_string());
    match host.output_devices() {
        Ok(devices) => devices
            .map(|device| device.to_string())
            .map(|name| {
                device_info_for_name(
                    &name,
                    config,
                    default_name.as_deref() == Some(name.as_str()),
                )
            })
            .collect(),
        Err(error) => vec![SoundOutputDeviceInfo {
            descriptor: descriptor_for_name("Default Output", config),
            is_default: true,
            available: false,
            diagnostic: Some(format!("Kira/CPAL device enumeration failed: {error}")),
        }],
    }
}

pub(super) fn backend_settings(
    descriptor: &SoundOutputDeviceDescriptor,
) -> Result<CpalBackendSettings, SoundError> {
    if descriptor.channel_count > 2 {
        return Err(SoundError::UnsupportedAdvancedFeature(
            "Kira v1 output supports mono or stereo only".to_string(),
        ));
    }
    let device = if descriptor.id == SoundOutputDeviceId::default_system() {
        None
    } else {
        let host = cpal::default_host();
        let wanted_id = descriptor.id.as_str();
        let wanted_name = descriptor.display_name.as_str();
        let selected = host
            .output_devices()
            .map_err(|error| SoundError::BackendUnavailable {
                detail: format!("Kira/CPAL device enumeration failed: {error}"),
            })?
            .find(|device| {
                let name = device.to_string();
                name == wanted_name || format!("{KIRA_CPAL_BACKEND}:{name}") == wanted_id
            })
            .ok_or_else(|| SoundError::BackendUnavailable {
                detail: format!("selected output device `{wanted_name}` is unavailable"),
            })?;
        Some(selected)
    };
    Ok(CpalBackendSettings {
        device,
        config: Some(cpal::StreamConfig {
            channels: descriptor.channel_count,
            sample_rate: descriptor.sample_rate_hz,
            buffer_size: cpal::BufferSize::Fixed(descriptor.block_size_frames as u32),
        }),
    })
}

fn descriptor_for_name(name: &str, config: &SoundConfig) -> SoundOutputDeviceDescriptor {
    SoundOutputDeviceDescriptor {
        id: SoundOutputDeviceId::new(format!("{KIRA_CPAL_BACKEND}:{name}")),
        backend: KIRA_CPAL_BACKEND.to_string(),
        display_name: name.to_string(),
        sample_rate_hz: config.sample_rate_hz,
        channel_count: config.channel_count,
        channel_layout: config.channel_layout.clone(),
        block_size_frames: config.block_size_frames,
        latency_blocks: 2,
    }
}

fn device_info_for_name(
    name: &str,
    config: &SoundConfig,
    is_default: bool,
) -> SoundOutputDeviceInfo {
    let multichannel_unsupported = config.channel_count > 2;
    SoundOutputDeviceInfo {
        descriptor: descriptor_for_name(name, config),
        is_default,
        available: !multichannel_unsupported,
        diagnostic: multichannel_unsupported.then(|| {
            format!(
                "{}-channel sound output is unavailable in Kira v1",
                config.channel_count
            )
        }),
    }
}

#[cfg(test)]
pub(crate) fn device_info_for_test(config: &SoundConfig) -> SoundOutputDeviceInfo {
    device_info_for_name("Test Output", config, true)
}
