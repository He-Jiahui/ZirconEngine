use kira::backend::cpal::cpal::{
    BufferSize, SampleFormat, StreamConfig, SupportedBufferSize, SupportedStreamConfigRange,
};

use crate::kira_bridge::{
    admit_exact_output_config, admit_output_config_for_policy, device_identity_matches,
    output_buffer_policy, OutputBufferPolicy,
};

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

fn wasapi_period_range() -> SupportedStreamConfigRange {
    SupportedStreamConfigRange::new(
        2,
        48_000,
        48_000,
        SupportedBufferSize::Range { min: 480, max: 480 },
        SampleFormat::F32,
    )
}

#[test]
fn wasapi_shared_period_admits_distinct_fixed_ring_request_without_rewriting_it() {
    let requested = request(48_000, 2, 256);
    let supported = wasapi_period_range();
    assert!(admit_output_config_for_policy(
        OutputBufferPolicy::WasapiSharedPeriod,
        [supported],
        &requested,
    )
    .is_ok());
    assert_eq!(requested.buffer_size, BufferSize::Fixed(256));
    assert_eq!(
        *supported.buffer_size(),
        SupportedBufferSize::Range { min: 480, max: 480 }
    );
    #[cfg(windows)]
    assert_eq!(
        output_buffer_policy(kira::backend::cpal::cpal::HostId::Wasapi),
        OutputBufferPolicy::WasapiSharedPeriod
    );
}

#[test]
fn other_hosts_keep_strict_reported_buffer_range_admission() {
    assert!(admit_output_config_for_policy(
        OutputBufferPolicy::ReportedRange,
        [wasapi_period_range()],
        &request(48_000, 2, 256),
    )
    .is_err());
    assert!(admit_output_config_for_policy(
        OutputBufferPolicy::ReportedRange,
        [wasapi_period_range()],
        &request(48_000, 2, 480),
    )
    .is_ok());
    for host in kira::backend::cpal::cpal::ALL_HOSTS {
        #[cfg(windows)]
        if *host == kira::backend::cpal::cpal::HostId::Wasapi {
            continue;
        }
        assert_eq!(
            output_buffer_policy(*host),
            OutputBufferPolicy::ReportedRange
        );
    }
}

#[test]
fn shared_period_policy_rejects_unknown_formats_and_invalid_fixed_requests() {
    for policy in [
        OutputBufferPolicy::ReportedRange,
        OutputBufferPolicy::WasapiSharedPeriod,
    ] {
        for requested in [
            request(192_000, 2, 256),
            request(48_000, 1, 256),
            request(0, 2, 256),
            request(48_000, 0, 256),
            request(48_000, 2, 0),
            StreamConfig {
                buffer_size: BufferSize::Default,
                ..request(48_000, 2, 256)
            },
        ] {
            assert!(
                admit_output_config_for_policy(policy, [wasapi_period_range()], &requested)
                    .is_err()
            );
        }
        for (buffer_size, sample_format) in [
            (SupportedBufferSize::Unknown, SampleFormat::F32),
            (
                SupportedBufferSize::Range { min: 480, max: 480 },
                SampleFormat::I16,
            ),
            (
                SupportedBufferSize::Range { min: 0, max: 0 },
                SampleFormat::F32,
            ),
            (
                SupportedBufferSize::Range { min: 480, max: 128 },
                SampleFormat::F32,
            ),
        ] {
            let range =
                SupportedStreamConfigRange::new(2, 48_000, 48_000, buffer_size, sample_format);
            assert!(
                admit_output_config_for_policy(policy, [range], &request(48_000, 2, 256)).is_err()
            );
        }
        assert!(admit_output_config_for_policy(policy, [], &request(48_000, 2, 256)).is_err());
    }
}
