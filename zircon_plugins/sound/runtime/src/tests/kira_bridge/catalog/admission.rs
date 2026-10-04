use kira::backend::cpal::cpal::{
    BufferSize, SampleFormat, StreamConfig, SupportedBufferSize, SupportedStreamConfigRange,
};

use crate::kira_bridge::{admit_exact_output_config, device_identity_matches};

fn stereo_f32_range() -> SupportedStreamConfigRange {
    SupportedStreamConfigRange::new(
        2,
        44_100,
        96_000,
        SupportedBufferSize::Range { min: 128, max: 512 },
        SampleFormat::F32,
    )
}

fn request(rate: u32, channels: u16, frames: u32) -> StreamConfig {
    StreamConfig {
        channels,
        sample_rate: rate,
        buffer_size: BufferSize::Fixed(frames),
    }
}

#[test]
fn exact_rate_channel_and_buffer_are_admitted_from_supported_ranges() {
    assert!(admit_exact_output_config([stereo_f32_range()], &request(48_000, 2, 256),).is_ok());
}

#[test]
fn unsupported_rate_channel_and_buffer_are_rejected() {
    let supported = [stereo_f32_range()];
    assert!(admit_exact_output_config(supported, &request(192_000, 2, 256)).is_err());
    assert!(admit_exact_output_config([stereo_f32_range()], &request(48_000, 1, 256)).is_err());
    assert!(admit_exact_output_config([stereo_f32_range()], &request(48_000, 2, 1024)).is_err());
}

#[test]
fn unknown_buffer_and_non_f32_ranges_are_rejected() {
    assert!(admit_exact_output_config(
        [SupportedStreamConfigRange::new(
            2,
            44_100,
            96_000,
            SupportedBufferSize::Unknown,
            SampleFormat::F32,
        )],
        &request(48_000, 2, 256),
    )
    .is_err());
    assert!(admit_exact_output_config(
        [SupportedStreamConfigRange::new(
            2,
            44_100,
            96_000,
            SupportedBufferSize::Range { min: 128, max: 512 },
            SampleFormat::I16,
        )],
        &request(48_000, 2, 256),
    )
    .is_err());
}

#[test]
fn explicit_identity_is_exact_and_never_falls_back_to_display_name() {
    assert!(device_identity_matches("host:device-1", "host:device-1"));
    assert!(!device_identity_matches(
        "host:device-1",
        "Friendly Speakers"
    ));
    assert!(!device_identity_matches("host:missing", "host:device-1"));
}
