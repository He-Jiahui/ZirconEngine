use zircon_runtime::core::framework::sound::{SoundError, SoundOutputDeviceDescriptor};

use super::error::backend_unavailable;
use super::{CPAL_DEFAULT_OUTPUT_DEVICE_ID, CPAL_OUTPUT_DEVICE_ID_PREFIX};

pub(in crate::output::cpal) fn select_output_device(
    host: &cpal::Host,
    descriptor: &SoundOutputDeviceDescriptor,
) -> Result<cpal::Device, SoundError> {
    use cpal::traits::HostTrait;

    match cpal_picker_device_index(descriptor.id.as_str())? {
        Some(index) => host
            .output_devices()
            .map_err(|error| {
                backend_unavailable(format!("cpal output device enumeration failed: {error}"))
            })?
            .nth(index)
            .ok_or_else(|| {
                backend_unavailable(format!(
                    "cpal output device `{}` is not available",
                    descriptor.id.as_str()
                ))
            }),
        None => host.default_output_device().ok_or_else(|| {
            backend_unavailable("cpal default output device is not available".to_string())
        }),
    }
}

pub(in crate::output::cpal) fn cpal_picker_device_index(
    device_id: &str,
) -> Result<Option<usize>, SoundError> {
    if device_id == CPAL_DEFAULT_OUTPUT_DEVICE_ID {
        return Ok(None);
    }
    let Some(raw_index) = device_id.strip_prefix(CPAL_OUTPUT_DEVICE_ID_PREFIX) else {
        return Ok(None);
    };
    raw_index.parse::<usize>().map(Some).map_err(|_| {
        backend_unavailable(format!(
            "cpal output device id `{device_id}` has an invalid picker index"
        ))
    })
}

pub(in crate::output::cpal) fn select_stream_config(
    device: &cpal::Device,
    descriptor: &SoundOutputDeviceDescriptor,
) -> Result<cpal::StreamConfig, SoundError> {
    use cpal::traits::DeviceTrait;

    let supported_configs = device.supported_output_configs().map_err(|error| {
        backend_unavailable(format!("cpal output config query failed: {error}"))
    })?;
    for config in supported_configs {
        if config.sample_format() != cpal::SampleFormat::F32 {
            continue;
        }
        if config.channels() != descriptor.channel_count {
            continue;
        }
        let sample_rate = descriptor.sample_rate_hz;
        if sample_rate < config.min_sample_rate().0 || sample_rate > config.max_sample_rate().0 {
            continue;
        }
        return Ok(config
            .with_sample_rate(cpal::SampleRate(sample_rate))
            .config());
    }
    Err(backend_unavailable(format!(
        "cpal selected output device does not support f32 {} Hz / {} channel output",
        descriptor.sample_rate_hz, descriptor.channel_count
    )))
}

#[cfg(test)]
mod tests {
    use super::super::{CPAL_DEFAULT_OUTPUT_DEVICE_ID, CPAL_OUTPUT_DEVICE_ID_PREFIX};
    use super::cpal_picker_device_index;

    #[test]
    fn output_device_cpal_picker_ids_parse_default_and_indexed_devices() {
        assert_eq!(
            cpal_picker_device_index(CPAL_DEFAULT_OUTPUT_DEVICE_ID).unwrap(),
            None
        );
        assert_eq!(
            cpal_picker_device_index(&format!("{CPAL_OUTPUT_DEVICE_ID_PREFIX}7")).unwrap(),
            Some(7)
        );
        assert_eq!(
            cpal_picker_device_index("sound.output.cpal.manual").unwrap(),
            None
        );
        assert!(cpal_picker_device_index(&format!("{CPAL_OUTPUT_DEVICE_ID_PREFIX}bad")).is_err());
    }
}
