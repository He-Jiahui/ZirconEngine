use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 21;
const LAYOUT_CHECKS_PER_SAMPLE: usize = 8_192;

#[test]
#[ignore = "release-only performance contract"]
fn benchmark_sound_wav_channel_layout_mask_projection() {
    let mut legacy_raw = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_raw = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_raw.push(measure_layout_projection(
                legacy_channel_layout_from_wav_mask,
            ));
            optimized_raw.push(measure_layout_projection(channel_layout_from_wav_mask));
        } else {
            optimized_raw.push(measure_layout_projection(channel_layout_from_wav_mask));
            legacy_raw.push(measure_layout_projection(
                legacy_channel_layout_from_wav_mask,
            ));
        }
    }

    let legacy_p95_ns = nearest_rank(&legacy_raw, 95);
    let optimized_p95_ns = nearest_rank(&optimized_raw, 95);
    let improvement_percent = legacy_p95_ns
        .saturating_sub(optimized_p95_ns)
        .saturating_mul(100)
        / legacy_p95_ns.max(1);
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(50),
        "direct WAV channel-layout selection must improve P95 by at least 50%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
    println!(
        "PERF_RESULT task=plugins07_direct_wav_channel_layout sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank layout_checks_per_sample={LAYOUT_CHECKS_PER_SAMPLE} benchmark_channel_mask=0000063f benchmark_channel_count=8 legacy_layout_allocations_per_decode=18 optimized_layout_allocations_per_decode=2 legacy_transient_allocations_per_decode=16 optimized_transient_allocations_per_decode=0 threshold_percent=50 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} improvement_percent={improvement_percent} legacy_raw_ns={} optimized_raw_ns={}",
        raw_samples(&legacy_raw),
        raw_samples(&optimized_raw)
    );
}

fn measure_layout_projection(
    project: fn(u32, u16) -> SoundAssetResult<AudioChannelLayout>,
) -> u128 {
    let started = Instant::now();
    for _ in 0..LAYOUT_CHECKS_PER_SAMPLE {
        black_box(project(black_box(SURROUND_7_1_SPEAKER_MASK), 8).unwrap());
    }
    started.elapsed().as_nanos()
}

fn legacy_channel_layout_from_wav_mask(
    channel_mask: u32,
    channel_count: u16,
) -> SoundAssetResult<AudioChannelLayout> {
    if channel_mask.count_ones() != channel_count as u32 {
        return Err(SoundAssetError::ChannelMaskCountMismatch {
            channel_mask,
            channel_count,
        });
    }
    let unsupported = channel_mask & !SUPPORTED_WAV_SPEAKER_MASK;
    if unsupported != 0 {
        return Err(SoundAssetError::UnsupportedSpeakerMaskBits {
            channel_mask,
            unsupported,
        });
    }

    let mut speakers = Vec::with_capacity(channel_count as usize);
    for (bit, speaker) in [
        (SPEAKER_FRONT_LEFT, AudioSpeakerChannel::FrontLeft),
        (SPEAKER_FRONT_RIGHT, AudioSpeakerChannel::FrontRight),
        (SPEAKER_FRONT_CENTER, AudioSpeakerChannel::FrontCenter),
        (SPEAKER_LOW_FREQUENCY, AudioSpeakerChannel::LowFrequency),
        (SPEAKER_BACK_LEFT, AudioSpeakerChannel::BackLeft),
        (SPEAKER_BACK_RIGHT, AudioSpeakerChannel::BackRight),
        (SPEAKER_SIDE_LEFT, AudioSpeakerChannel::SideLeft),
        (SPEAKER_SIDE_RIGHT, AudioSpeakerChannel::SideRight),
    ] {
        if channel_mask & bit != 0 {
            speakers.push(speaker);
        }
    }
    Ok(legacy_layout_from_speakers(
        channel_count,
        speakers,
        format!("wav_extensible_{channel_mask:08x}"),
    ))
}

fn legacy_layout_from_speakers(
    channel_count: u16,
    speakers: Vec<AudioSpeakerChannel>,
    fallback_name: String,
) -> AudioChannelLayout {
    [
        AudioChannelLayout::mono(),
        AudioChannelLayout::stereo(),
        AudioChannelLayout::quad(),
        AudioChannelLayout::surround_5_0(),
        AudioChannelLayout::surround_5_1(),
        AudioChannelLayout::surround_5_1_side(),
        AudioChannelLayout::surround_7_0(),
        AudioChannelLayout::surround_7_1(),
    ]
    .into_iter()
    .find(|layout| layout.channel_count == channel_count && layout.speakers == speakers)
    .unwrap_or(AudioChannelLayout {
        name: fallback_name,
        channel_count,
        speakers,
    })
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn raw_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
