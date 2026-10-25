use std::{iter::FusedIterator, num::NonZeroUsize, ops::Range};

use zircon_runtime::asset::SoundAsset;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SoundWaveEnvelopeBin {
    /// Original source frame range, with an exclusive end, never display seconds or FPS.
    pub start_frame: usize,
    pub end_frame: usize,
    pub minimum: f32,
    pub maximum: f32,
}

#[derive(Clone, Debug)]
pub struct SoundWaveEnvelope<'a> {
    asset: &'a SoundAsset,
    channel: usize,
    bin_count: usize,
    bin_index: usize,
    next_frame: usize,
    base_frames: usize,
    remainder: usize,
}

impl<'a> SoundWaveEnvelope<'a> {
    pub(super) fn new(
        asset: &'a SoundAsset,
        frames: Range<usize>,
        channel: usize,
        requested_bins: NonZeroUsize,
    ) -> Self {
        // The only caller validates this range against the already-validated source.
        let frame_count = frames.end - frames.start;
        let bin_count = frame_count.min(requested_bins.get());
        let (base_frames, remainder) = if bin_count == 0 {
            (0, 0)
        } else {
            (frame_count / bin_count, frame_count % bin_count)
        };
        Self {
            asset,
            channel,
            bin_count,
            bin_index: 0,
            next_frame: frames.start,
            base_frames,
            remainder,
        }
    }
}

impl Iterator for SoundWaveEnvelope<'_> {
    type Item = SoundWaveEnvelopeBin;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bin_index == self.bin_count {
            return None;
        }
        let start_frame = self.next_frame;
        // Quotient/remainder partitions cover every source frame exactly once. Each bin is
        // nonempty, and every end/channel product is bounded by the validated sample slice.
        // Unlike frame_count * bin_index / bin_count, this cannot overflow for a huge budget.
        let end_frame =
            start_frame + self.base_frames + usize::from(self.bin_index < self.remainder);
        let channels = usize::from(self.asset.channel_count);
        let samples = &self.asset.samples[start_frame * channels..end_frame * channels];
        let mut minimum = samples[self.channel];
        let mut maximum = minimum;
        for frame in samples.chunks_exact(channels) {
            minimum = minimum.min(frame[self.channel]);
            maximum = maximum.max(frame[self.channel]);
        }
        self.bin_index += 1;
        self.next_frame = end_frame;
        Some(SoundWaveEnvelopeBin {
            start_frame,
            end_frame,
            minimum,
            maximum,
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.bin_count - self.bin_index;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for SoundWaveEnvelope<'_> {}
impl FusedIterator for SoundWaveEnvelope<'_> {}
