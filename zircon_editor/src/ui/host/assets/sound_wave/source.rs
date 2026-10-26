use std::{num::NonZeroUsize, ops::Range};

use zircon_runtime::asset::{AssetUri, SoundAsset};
use zircon_runtime::core::framework::audio::AudioChannelLayout;

use super::{SoundWaveEnvelope, SoundWaveSourceError};

#[derive(Clone, Copy, Debug)]
pub struct SoundWaveSource<'a> {
    asset: &'a SoundAsset,
    frame_count: usize,
}

impl<'a> SoundWaveSource<'a> {
    pub fn uri(&self) -> &'a AssetUri {
        &self.asset.uri
    }

    pub fn sample_rate_hz(&self) -> u32 {
        self.asset.sample_rate_hz
    }

    pub fn channel_count(&self) -> u16 {
        self.asset.channel_count
    }

    /// Preserve the original speaker order, including canonical discrete layouts whose
    /// positions remain unknown. A display adapter must not infer or rewrite that order.
    pub fn channel_layout(&self) -> &'a AudioChannelLayout {
        &self.asset.channel_layout
    }

    /// Together with sample_rate_hz this is an exact duration ratio, not a playback clock.
    pub fn frame_count(&self) -> usize {
        self.frame_count
    }

    /// Produces lazy per-channel extrema from the original decoded samples. The caller
    /// chooses its visible bin budget; no envelope buffer, downmix, CurveKey or clock is made.
    pub fn waveform(
        &self,
        channel: usize,
        requested_bins: NonZeroUsize,
    ) -> Result<SoundWaveEnvelope<'a>, SoundWaveSourceError> {
        self.waveform_frames(channel, 0..self.frame_count, requested_bins)
    }

    /// Projects one visible, half-open source-frame interval without changing the source.
    /// Bin coordinates retain the original frame origin, including after zoom or pan.
    /// Empty intervals produce no bins; invalid intervals are rejected rather than clamped.
    /// The caller supplies source frames, never an unqualified playback cursor or FPS value.
    pub fn waveform_frames(
        &self,
        channel: usize,
        frames: Range<usize>,
        requested_bins: NonZeroUsize,
    ) -> Result<SoundWaveEnvelope<'a>, SoundWaveSourceError> {
        if channel >= usize::from(self.asset.channel_count) {
            return Err(SoundWaveSourceError::MissingChannel {
                channel,
                channels: self.asset.channel_count,
            });
        }
        if frames.start > frames.end || frames.end > self.frame_count {
            return Err(SoundWaveSourceError::InvalidFrameRange {
                start: frames.start,
                end: frames.end,
                frames: self.frame_count,
            });
        }
        Ok(SoundWaveEnvelope::new(
            self.asset,
            frames,
            channel,
            requested_bins,
        ))
    }
}

pub struct SoundWaveSourceAdapter;

impl SoundWaveSourceAdapter {
    /// Reads one immutable, already-decoded SoundAsset supplied by the original asset owner.
    /// This does not import a URI, prove catalog/publication identity, resolve a SoundClipId,
    /// establish device readiness or enable playback/recording. The host retains those gates.
    pub fn read(asset: &SoundAsset) -> Result<SoundWaveSource<'_>, SoundWaveSourceError> {
        if asset.sample_rate_hz == 0 {
            return Err(SoundWaveSourceError::ZeroSampleRate);
        }
        if asset.channel_count == 0 {
            return Err(SoundWaveSourceError::ZeroChannels);
        }
        if !asset
            .channel_layout
            .matches_channel_count(asset.channel_count)
        {
            return Err(SoundWaveSourceError::LayoutChannelMismatch {
                channels: asset.channel_count,
                layout_channels: asset.channel_layout.channel_count,
            });
        }
        if !asset.channel_layout.is_valid_contract_layout() {
            return Err(SoundWaveSourceError::InvalidLayout);
        }
        let channels = usize::from(asset.channel_count);
        if asset.samples.len() % channels != 0 {
            return Err(SoundWaveSourceError::PartialFrame {
                samples: asset.samples.len(),
                channels: asset.channel_count,
            });
        }
        if let Some(sample_index) = asset.samples.iter().position(|sample| !sample.is_finite()) {
            return Err(SoundWaveSourceError::NonfiniteSample { sample_index });
        }
        Ok(SoundWaveSource {
            asset,
            frame_count: asset.samples.len() / channels,
        })
    }
}
