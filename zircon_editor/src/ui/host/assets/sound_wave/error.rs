use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum SoundWaveSourceError {
    #[error("decoded sound has no sample rate")]
    ZeroSampleRate,
    #[error("decoded sound has no channels")]
    ZeroChannels,
    #[error(
        "decoded sound declares {channels} channels but its layout declares {layout_channels}"
    )]
    LayoutChannelMismatch { channels: u16, layout_channels: u16 },
    #[error("decoded sound layout violates the existing channel layout contract")]
    InvalidLayout,
    #[error("{samples} interleaved samples do not form complete {channels}-channel frames")]
    PartialFrame { samples: usize, channels: u16 },
    #[error("decoded sound sample {sample_index} is not finite")]
    NonfiniteSample { sample_index: usize },
    #[error("channel {channel} does not exist in the {channels}-channel source")]
    MissingChannel { channel: usize, channels: u16 },
    #[error("source frame range [{start}, {end}) is invalid for {frames} source frames")]
    InvalidFrameRange {
        start: usize,
        end: usize,
        frames: usize,
    },
}
