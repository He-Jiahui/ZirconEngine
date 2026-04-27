use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use zircon_runtime::asset::{
    AssetUri, ProjectAssetManager, SoundAsset, PROJECT_ASSET_MANAGER_NAME,
};
use zircon_runtime::core::framework::sound::{
    SoundBackendState, SoundBackendStatus, SoundClipId, SoundClipInfo, SoundError, SoundMixBlock,
    SoundPlaybackId, SoundPlaybackSettings,
};
use zircon_runtime::core::CoreHandle;

use super::SoundConfig;

#[derive(Clone, Debug, Default)]
pub struct SoundDriver;

#[derive(Debug, Default)]
struct SoundState {
    next_clip_id: u64,
    next_playback_id: u64,
    clip_ids_by_locator: HashMap<String, SoundClipId>,
    clips: HashMap<SoundClipId, LoadedClip>,
    playbacks: HashMap<SoundPlaybackId, ActivePlayback>,
}

#[derive(Clone, Debug)]
struct LoadedClip {
    asset: SoundAsset,
}

#[derive(Clone, Debug)]
struct ActivePlayback {
    clip: SoundClipId,
    cursor_frame: usize,
    gain: f32,
    looped: bool,
}

#[derive(Clone, Debug)]
pub struct DefaultSoundManager {
    core: Option<CoreHandle>,
    config: Arc<Mutex<SoundConfig>>,
    state: Arc<Mutex<SoundState>>,
}

impl Default for DefaultSoundManager {
    fn default() -> Self {
        Self::new(None)
    }
}

impl DefaultSoundManager {
    pub fn new(core: Option<CoreHandle>) -> Self {
        Self {
            core,
            config: Arc::new(Mutex::new(SoundConfig::default())),
            state: Arc::new(Mutex::new(SoundState::default())),
        }
    }

    fn config(&self) -> SoundConfig {
        self.config
            .lock()
            .expect("sound config mutex poisoned")
            .clone()
    }

    fn project_asset_manager(&self) -> Result<Arc<ProjectAssetManager>, SoundError> {
        let core = self
            .core
            .as_ref()
            .ok_or_else(|| SoundError::BackendUnavailable {
                detail: "sound manager is not attached to a CoreRuntime".to_string(),
            })?;
        core.resolve_manager::<ProjectAssetManager>(PROJECT_ASSET_MANAGER_NAME)
            .map_err(|error| SoundError::BackendUnavailable {
                detail: error.to_string(),
            })
    }
}

impl zircon_runtime::core::framework::sound::SoundManager for DefaultSoundManager {
    fn backend_name(&self) -> String {
        let config = self.config();
        if config.enabled {
            config.backend
        } else {
            "disabled".to_string()
        }
    }

    fn backend_status(&self) -> SoundBackendStatus {
        let config = self.config();
        if !config.enabled {
            return SoundBackendStatus {
                requested_backend: config.backend,
                active_backend: None,
                state: SoundBackendState::Disabled,
                detail: Some("sound playback is disabled".to_string()),
                sample_rate_hz: config.sample_rate_hz,
                channel_count: config.channel_count,
            };
        }

        SoundBackendStatus {
            requested_backend: config.backend.clone(),
            active_backend: Some(config.backend),
            state: SoundBackendState::Ready,
            detail: None,
            sample_rate_hz: config.sample_rate_hz,
            channel_count: config.channel_count,
        }
    }

    fn load_clip(&self, locator: &str) -> Result<SoundClipId, SoundError> {
        let uri = AssetUri::parse(locator).map_err(|_| SoundError::InvalidLocator {
            locator: locator.to_string(),
        })?;
        let asset_manager = self.project_asset_manager()?;
        let asset_id =
            asset_manager
                .resolve_asset_id(&uri)
                .ok_or_else(|| SoundError::InvalidLocator {
                    locator: locator.to_string(),
                })?;
        let asset = asset_manager
            .load_sound_asset(asset_id)
            .map_err(|error| SoundError::Decode(error.to_string()))?;

        let mut state = self.state.lock().expect("sound state mutex poisoned");
        if let Some(existing) = state.clip_ids_by_locator.get(locator).copied() {
            return Ok(existing);
        }

        state.next_clip_id += 1;
        let clip_id = SoundClipId::new(state.next_clip_id);
        state
            .clip_ids_by_locator
            .insert(locator.to_string(), clip_id);
        state.clips.insert(clip_id, LoadedClip { asset });
        Ok(clip_id)
    }

    fn clip_info(&self, clip: SoundClipId) -> Result<SoundClipInfo, SoundError> {
        let state = self.state.lock().expect("sound state mutex poisoned");
        let clip = state
            .clips
            .get(&clip)
            .ok_or(SoundError::UnknownClip { clip })?;
        Ok(SoundClipInfo {
            locator: clip.asset.uri.to_string(),
            sample_rate_hz: clip.asset.sample_rate_hz,
            channel_count: clip.asset.channel_count,
            frame_count: clip.asset.frame_count(),
            duration_seconds: clip.asset.frame_count() as f32 / clip.asset.sample_rate_hz as f32,
        })
    }

    fn play_clip(
        &self,
        clip: SoundClipId,
        settings: SoundPlaybackSettings,
    ) -> Result<SoundPlaybackId, SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        if !state.clips.contains_key(&clip) {
            return Err(SoundError::UnknownClip { clip });
        }

        state.next_playback_id += 1;
        let playback_id = SoundPlaybackId::new(state.next_playback_id);
        state.playbacks.insert(
            playback_id,
            ActivePlayback {
                clip,
                cursor_frame: 0,
                gain: settings.gain,
                looped: settings.looped,
            },
        );
        Ok(playback_id)
    }

    fn stop_playback(&self, playback: SoundPlaybackId) -> Result<(), SoundError> {
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .playbacks
            .remove(&playback)
            .map(|_| ())
            .ok_or(SoundError::UnknownPlayback { playback })
    }

    fn render_mix(&self, frames: usize) -> Result<SoundMixBlock, SoundError> {
        if frames == 0 {
            return Err(SoundError::InvalidMixRequest { frames });
        }

        let config = self.config();
        if !config.enabled {
            return Err(SoundError::BackendUnavailable {
                detail: "sound playback is disabled".to_string(),
            });
        }

        let output_channels = config.channel_count.max(1);
        let mut mix = SoundMixBlock::silent(config.sample_rate_hz, output_channels, frames);
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let clips = state.clips.clone();
        let mut finished = Vec::new();

        for (playback_id, playback) in state.playbacks.iter_mut() {
            let Some(clip) = clips.get(&playback.clip) else {
                finished.push(*playback_id);
                continue;
            };

            if mix_playback(
                &mut mix.samples,
                output_channels,
                config.master_gain,
                &clip.asset,
                playback,
            ) {
                finished.push(*playback_id);
            }
        }

        for playback_id in finished {
            state.playbacks.remove(&playback_id);
        }
        for sample in &mut mix.samples {
            *sample = sample.clamp(-1.0, 1.0);
        }

        Ok(mix)
    }
}

fn mix_playback(
    destination: &mut [f32],
    output_channels: u16,
    master_gain: f32,
    clip: &SoundAsset,
    playback: &mut ActivePlayback,
) -> bool {
    let clip_channels = clip.channel_count as usize;
    let output_channels = output_channels as usize;
    let frame_count = clip.frame_count();
    if frame_count == 0 || clip_channels == 0 {
        return true;
    }

    for frame_index in 0..(destination.len() / output_channels) {
        if playback.cursor_frame >= frame_count {
            if playback.looped {
                playback.cursor_frame = 0;
            } else {
                return true;
            }
        }

        let clip_frame_offset = playback.cursor_frame * clip_channels;
        let clip_frame = &clip.samples[clip_frame_offset..clip_frame_offset + clip_channels];
        let output_offset = frame_index * output_channels;
        for channel in 0..output_channels {
            destination[output_offset + channel] +=
                sample_for_output_channel(clip_frame, channel, output_channels)
                    * playback.gain
                    * master_gain;
        }
        playback.cursor_frame += 1;
    }

    false
}

fn sample_for_output_channel(
    clip_frame: &[f32],
    output_channel: usize,
    output_channel_count: usize,
) -> f32 {
    if clip_frame.len() == 1 {
        return clip_frame[0];
    }
    if output_channel_count == 1 {
        return clip_frame.iter().copied().sum::<f32>() / clip_frame.len() as f32;
    }

    clip_frame
        .get(output_channel)
        .copied()
        .unwrap_or_else(|| *clip_frame.last().unwrap_or(&0.0))
}
