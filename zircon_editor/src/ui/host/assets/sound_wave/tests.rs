//! Pure memory contracts; no project asset, decoder registration or playback is produced here.

#[path = "tests/visible_range.rs"]
mod visible_range;

use std::num::NonZeroUsize;

use zircon_runtime::asset::{AssetUri, SoundAsset};
use zircon_runtime::core::framework::audio::AudioChannelLayout;

use super::{SoundWaveEnvelopeBin, SoundWaveSourceAdapter, SoundWaveSourceError};

fn sound(samples: Vec<f32>, channels: u16) -> SoundAsset {
    SoundAsset {
        uri: AssetUri::parse("res://audio/source.wav").unwrap(),
        sample_rate_hz: 44_100,
        channel_count: channels,
        channel_layout: AudioChannelLayout::for_channel_count(channels),
        samples,
    }
}

fn budget(value: usize) -> NonZeroUsize {
    NonZeroUsize::new(value).unwrap()
}

#[test]
fn sound_wave_source_preserves_borrowed_identity_rate_and_surround_order() {
    let mut asset = sound(vec![0.0; 24], 8);
    asset.channel_layout = AudioChannelLayout::surround_7_1();
    let before = asset.clone();
    let source = SoundWaveSourceAdapter::read(&asset).unwrap();
    assert!(std::ptr::eq(source.uri(), &asset.uri));
    assert!(std::ptr::eq(source.channel_layout(), &asset.channel_layout));
    assert_eq!(source.sample_rate_hz(), 44_100);
    assert_eq!(source.channel_count(), 8);
    assert_eq!(source.frame_count(), 3);
    assert_eq!(asset, before);
}

#[test]
fn sound_wave_source_rejects_zero_format_values_instead_of_zero_duration_fallback() {
    let mut asset = sound(vec![0.0], 1);
    asset.sample_rate_hz = 0;
    assert_eq!(
        SoundWaveSourceAdapter::read(&asset).unwrap_err(),
        SoundWaveSourceError::ZeroSampleRate
    );
    asset.sample_rate_hz = 44_100;
    asset.channel_count = 0;
    assert_eq!(
        SoundWaveSourceAdapter::read(&asset).unwrap_err(),
        SoundWaveSourceError::ZeroChannels
    );
}

#[test]
fn sound_wave_source_reuses_full_layout_contract_without_reordering_samples() {
    let mut asset = sound(vec![0.0, 0.0], 2);
    asset.channel_layout = AudioChannelLayout::mono();
    assert_eq!(
        SoundWaveSourceAdapter::read(&asset).unwrap_err(),
        SoundWaveSourceError::LayoutChannelMismatch {
            channels: 2,
            layout_channels: 1
        }
    );
    asset.channel_layout = AudioChannelLayout::stereo();
    asset.channel_layout.speakers.reverse();
    assert_eq!(
        SoundWaveSourceAdapter::read(&asset).unwrap_err(),
        SoundWaveSourceError::InvalidLayout
    );
    asset.channel_layout = AudioChannelLayout::discrete(2);
    asset.channel_layout.name = "discrete_02".into();
    assert_eq!(
        SoundWaveSourceAdapter::read(&asset).unwrap_err(),
        SoundWaveSourceError::InvalidLayout
    );
}

#[test]
fn sound_wave_source_keeps_canonical_discrete_positions_unknown() {
    let mut asset = sound(vec![0.1, 0.2, 0.3], 3);
    asset.channel_layout = AudioChannelLayout::discrete(3);
    let source = SoundWaveSourceAdapter::read(&asset).unwrap();
    assert!(source.channel_layout().speakers.is_empty());
    assert_eq!(source.channel_layout().name, "discrete_3");
    let channel: Vec<_> = source.waveform(2, budget(1)).unwrap().collect();
    assert_eq!((channel[0].minimum, channel[0].maximum), (0.3, 0.3));
}

#[test]
fn sound_wave_source_rejects_partial_interleaved_frame_instead_of_truncating() {
    let asset = sound(vec![0.0, 0.0, 1.0], 2);
    assert_eq!(
        SoundWaveSourceAdapter::read(&asset).unwrap_err(),
        SoundWaveSourceError::PartialFrame {
            samples: 3,
            channels: 2
        }
    );
}

#[test]
fn sound_wave_source_rejects_nonfinite_samples_before_any_channel_projection() {
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let asset = sound(vec![0.0, 0.0, 0.0, bad], 2);
        assert_eq!(
            SoundWaveSourceAdapter::read(&asset).unwrap_err(),
            SoundWaveSourceError::NonfiniteSample { sample_index: 3 }
        );
    }
}

#[test]
fn sound_wave_source_empty_decoded_samples_have_no_fabricated_envelope_bins() {
    let asset = sound(vec![], 1);
    let source = SoundWaveSourceAdapter::read(&asset).unwrap();
    assert_eq!(source.frame_count(), 0);
    let mut bins = source.waveform(0, budget(usize::MAX)).unwrap();
    assert_eq!(bins.len(), 0);
    assert_eq!(bins.size_hint(), (0, Some(0)));
    assert_eq!(bins.next(), None);
    assert_eq!(bins.next(), None);
    assert_eq!(
        source.waveform(1, budget(1)).unwrap_err(),
        SoundWaveSourceError::MissingChannel {
            channel: 1,
            channels: 1
        }
    );
}

#[test]
fn sound_wave_source_uneven_bins_cover_every_original_frame_and_keep_peaks() {
    let asset = sound(vec![0.1, -0.8, 0.2, 1.5, -1.25, 0.4, 0.7], 1);
    let source = SoundWaveSourceAdapter::read(&asset).unwrap();
    let bins: Vec<_> = source.waveform(0, budget(3)).unwrap().collect();
    assert_eq!(
        bins,
        vec![
            SoundWaveEnvelopeBin {
                start_frame: 0,
                end_frame: 3,
                minimum: -0.8,
                maximum: 0.2
            },
            SoundWaveEnvelopeBin {
                start_frame: 3,
                end_frame: 5,
                minimum: -1.25,
                maximum: 1.5
            },
            SoundWaveEnvelopeBin {
                start_frame: 5,
                end_frame: 7,
                minimum: 0.4,
                maximum: 0.7
            }
        ]
    );
}

#[test]
fn sound_wave_source_phase_opposed_channels_are_not_downmixed_or_cancelled() {
    let asset = sound(vec![0.9, -0.9, 0.4, -0.4, 0.2, -0.2], 2);
    let source = SoundWaveSourceAdapter::read(&asset).unwrap();
    let left = source.waveform(0, budget(1)).unwrap().next().unwrap();
    let right = source.waveform(1, budget(1)).unwrap().next().unwrap();
    assert_eq!((left.minimum, left.maximum), (0.2, 0.9));
    assert_eq!((right.minimum, right.maximum), (-0.9, -0.2));
    assert_eq!((left.start_frame, left.end_frame), (0, 3));
    assert_eq!((right.start_frame, right.end_frame), (0, 3));
}

#[test]
fn sound_wave_source_huge_bin_budget_is_lazy_bounded_and_has_exact_remaining_length() {
    let asset = sound(vec![0.1, 0.2, 0.3], 1);
    let source = SoundWaveSourceAdapter::read(&asset).unwrap();
    let mut bins = source.waveform(0, budget(usize::MAX)).unwrap();
    assert_eq!(bins.len(), 3);
    for (index, expected) in [0.1, 0.2, 0.3].into_iter().enumerate() {
        assert_eq!(
            bins.next(),
            Some(SoundWaveEnvelopeBin {
                start_frame: index,
                end_frame: index + 1,
                minimum: expected,
                maximum: expected
            })
        );
        assert_eq!(bins.len(), 2 - index);
    }
    assert_eq!(bins.next(), None);
    assert_eq!(bins.next(), None);
    assert_eq!(
        source.waveform(usize::MAX, budget(1)).unwrap_err(),
        SoundWaveSourceError::MissingChannel {
            channel: usize::MAX,
            channels: 1
        }
    );
}
