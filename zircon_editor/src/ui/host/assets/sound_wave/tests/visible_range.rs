//! Visible-source intervals only; no file, asset admission, decoder, view store or clock.

use super::{budget, sound};
use crate::ui::host::editor_asset_manager::{
    SoundWaveEnvelopeBin, SoundWaveSourceAdapter, SoundWaveSourceError,
};

#[test]
fn sound_wave_source_visible_range_keeps_original_origin_and_excludes_offscreen_peaks() {
    let asset = sound(vec![1000.0, 1.0, -2.0, 0.5, 4.0, -0.25, -1000.0], 1);
    let before = asset.clone();
    let source = SoundWaveSourceAdapter::read(&asset).unwrap();
    let bins: Vec<_> = source
        .waveform_frames(0, 1..6, budget(2))
        .unwrap()
        .collect();
    assert_eq!(
        bins,
        vec![
            SoundWaveEnvelopeBin {
                start_frame: 1,
                end_frame: 4,
                minimum: -2.0,
                maximum: 1.0,
            },
            SoundWaveEnvelopeBin {
                start_frame: 4,
                end_frame: 6,
                minimum: -0.25,
                maximum: 4.0,
            },
        ]
    );
    assert_eq!(asset, before);
}

#[test]
fn sound_wave_source_visible_range_preserves_interleaved_channels_and_phase() {
    let asset = sound(
        vec![-50.0, 50.0, 1.0, -1.0, 0.5, -0.5, 2.0, -2.0, 100.0, -100.0],
        2,
    );
    let source = SoundWaveSourceAdapter::read(&asset).unwrap();
    let left = source
        .waveform_frames(0, 1..4, budget(1))
        .unwrap()
        .next()
        .unwrap();
    let right = source
        .waveform_frames(1, 1..4, budget(1))
        .unwrap()
        .next()
        .unwrap();
    assert_eq!((left.start_frame, left.end_frame), (1, 4));
    assert_eq!((right.start_frame, right.end_frame), (1, 4));
    assert_eq!((left.minimum, left.maximum), (0.5, 2.0));
    assert_eq!((right.minimum, right.maximum), (-2.0, -0.5));
    assert!(std::ptr::eq(source.channel_layout(), &asset.channel_layout));
}

#[test]
fn sound_wave_source_visible_range_whole_source_matches_existing_entry() {
    let asset = sound(
        vec![0.5, -0.5, 2.0, -2.0, -1.0, 1.0, 4.0, -4.0, 0.25, -0.25],
        2,
    );
    let source = SoundWaveSourceAdapter::read(&asset).unwrap();
    for channel in 0..2 {
        for bins in [1, 2, 3, usize::MAX] {
            let whole: Vec<_> = source.waveform(channel, budget(bins)).unwrap().collect();
            let visible: Vec<_> = source
                .waveform_frames(channel, 0..source.frame_count(), budget(bins))
                .unwrap()
                .collect();
            assert_eq!(visible, whole);
        }
    }
}

#[test]
fn sound_wave_source_visible_range_empty_intervals_remain_empty_and_fused() {
    for asset in [sound(vec![], 2), sound(vec![0.0; 6], 2)] {
        let source = SoundWaveSourceAdapter::read(&asset).unwrap();
        for frame in 0..=source.frame_count() {
            let mut bins = source
                .waveform_frames(1, frame..frame, budget(usize::MAX))
                .unwrap();
            assert_eq!(bins.len(), 0);
            assert_eq!(bins.size_hint(), (0, Some(0)));
            assert_eq!(bins.next(), None);
            assert_eq!(bins.next(), None);
        }
    }
}

#[test]
fn sound_wave_source_visible_range_rejects_invalid_bounds_without_clamping_or_overflow() {
    let asset = sound(vec![0.0; 3], 1);
    let source = SoundWaveSourceAdapter::read(&asset).unwrap();
    for (start, end) in [
        (2, 1),
        (0, 4),
        (4, 4),
        (usize::MAX, usize::MAX),
        (0, usize::MAX),
        (usize::MAX, 0),
    ] {
        assert_eq!(
            source
                .waveform_frames(0, start..end, budget(1))
                .unwrap_err(),
            SoundWaveSourceError::InvalidFrameRange {
                start,
                end,
                frames: 3
            }
        );
    }
    // Channel validation keeps the existing entry's precedence, even for an invalid range.
    assert_eq!(
        source
            .waveform_frames(1, usize::MAX..usize::MAX, budget(1))
            .unwrap_err(),
        SoundWaveSourceError::MissingChannel {
            channel: 1,
            channels: 1
        }
    );
    let empty = sound(vec![], 1);
    let empty_source = SoundWaveSourceAdapter::read(&empty).unwrap();
    assert_eq!(
        empty_source
            .waveform_frames(0, 1..1, budget(1))
            .unwrap_err(),
        SoundWaveSourceError::InvalidFrameRange {
            start: 1,
            end: 1,
            frames: 0
        }
    );
}

#[test]
fn sound_wave_source_visible_range_huge_budget_and_clones_keep_the_original_tail() {
    let asset = sound(vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0], 1);
    let source = SoundWaveSourceAdapter::read(&asset).unwrap();
    let mut bins = source.waveform_frames(0, 3..6, budget(usize::MAX)).unwrap();
    assert_eq!(bins.len(), 3);
    assert_eq!(
        bins.next(),
        Some(SoundWaveEnvelopeBin {
            start_frame: 3,
            end_frame: 4,
            minimum: 3.0,
            maximum: 3.0
        })
    );
    assert_eq!(bins.len(), 2);
    assert_eq!(bins.size_hint(), (2, Some(2)));
    let clone = bins.clone();
    let tail: Vec<_> = bins.by_ref().collect();
    assert_eq!(clone.collect::<Vec<_>>(), tail);
    assert_eq!((tail[0].start_frame, tail[1].end_frame), (4, 6));
    assert_eq!((tail[1].minimum, tail[1].maximum), (5.0, 5.0));
    assert_eq!(bins.len(), 0);
    assert_eq!(bins.next(), None);
    assert_eq!(bins.next(), None);
}

#[test]
fn sound_wave_source_visible_range_all_small_windows_preserve_sample_coverage() {
    let asset = sound(
        (0..9)
            .flat_map(|frame| {
                (0..3).map(move |channel| ((frame * 7 + channel * 3) % 17) as f32 - 8.0)
            })
            .collect(),
        3,
    );
    let source = SoundWaveSourceAdapter::read(&asset).unwrap();
    for start in 0..=source.frame_count() {
        for end in start..=source.frame_count() {
            for channel in 0..3 {
                for bin_budget in 1..=11 {
                    let bins: Vec<_> = source
                        .waveform_frames(channel, start..end, budget(bin_budget))
                        .unwrap()
                        .collect();
                    assert!(bins.len() <= bin_budget);
                    let mut next_frame = start;
                    for bin in bins {
                        assert_eq!(bin.start_frame, next_frame);
                        assert!(bin.start_frame < bin.end_frame && bin.end_frame <= end);
                        let samples: Vec<_> = (bin.start_frame..bin.end_frame)
                            .map(|frame| asset.samples[frame * 3 + channel])
                            .collect();
                        assert_eq!(
                            bin.minimum,
                            samples.iter().copied().fold(f32::INFINITY, f32::min)
                        );
                        assert_eq!(
                            bin.maximum,
                            samples.iter().copied().fold(f32::NEG_INFINITY, f32::max)
                        );
                        next_frame = bin.end_frame;
                    }
                    assert_eq!(next_frame, end);
                }
            }
        }
    }
}
