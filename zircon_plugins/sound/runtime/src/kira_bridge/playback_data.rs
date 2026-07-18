use std::sync::Arc;

use kira::{
    sound::{
        static_sound::{StaticSoundData, StaticSoundSettings},
        PlaybackPosition,
    },
    Frame,
};
use zircon_runtime::asset::SoundAsset;
use zircon_runtime::core::framework::sound::{SoundError, SoundPlaybackSettings};

use super::graph_compile::linear_gain_to_decibels;

pub(crate) fn static_sound_data(
    cached: &StaticSoundData,
    playback: &SoundPlaybackSettings,
    start_frame: usize,
    end_frame: Option<usize>,
) -> StaticSoundData {
    let gain = if playback.muted { 0.0 } else { playback.gain };
    let mut settings = StaticSoundSettings::default()
        .start_position(PlaybackPosition::Samples(0))
        .volume(linear_gain_to_decibels(gain))
        .playback_rate(playback.speed as f64)
        .panning(playback.pan);
    if playback.looped {
        settings = settings.loop_region(..);
    }
    let frame_count = cached.frames.len();
    let start_frame = start_frame.min(frame_count);
    let end_frame = end_frame.unwrap_or(frame_count).min(frame_count);
    let mut data = cached.clone();
    data.settings = settings;
    data.slice = (start_frame != 0 || end_frame != frame_count).then_some((start_frame, end_frame));
    data
}

pub(crate) fn cached_static_sound_data(asset: &SoundAsset) -> Result<StaticSoundData, SoundError> {
    Ok(StaticSoundData {
        sample_rate: asset.sample_rate_hz,
        frames: Arc::from(stereo_frames(asset)?),
        settings: StaticSoundSettings::default(),
        slice: None,
    })
}

fn stereo_frames(asset: &SoundAsset) -> Result<Vec<Frame>, SoundError> {
    match asset.channel_count {
        1 => Ok(asset
            .samples
            .iter()
            .copied()
            .map(|sample| Frame::new(sample, sample))
            .collect()),
        2 => Ok(asset
            .samples
            .chunks_exact(2)
            .map(|frame| Frame::new(frame[0], frame[1]))
            .collect()),
        channel_count => Err(SoundError::UnsupportedAdvancedFeature(format!(
            "{channel_count}-channel source downmix is enabled by Sound M4"
        ))),
    }
}
