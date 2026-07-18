use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

use kira::{
    backend::Backend,
    sound::static_sound::StaticSoundHandle,
    track::{SendTrackHandle, TrackHandle},
    AudioManager,
};
use zircon_runtime::core::framework::sound::{SoundMixerGraph, SoundPlaybackId, SoundTrackId};

mod graph;
mod lifecycle;
mod playback;

const DEFAULT_LOGICAL_RESOURCE_CAPACITY: usize = 128;

pub(crate) type DefaultKiraEngine = KiraEngine<kira::DefaultBackend>;

/// The only runtime owner of Kira managers, tracks, sends, and playback handles.
pub(crate) struct KiraEngine<B: Backend> {
    pub(super) manager: Option<AudioManager<B>>,
    pub(super) tracks: HashMap<SoundTrackId, TrackHandle>,
    pub(super) send_tracks: HashMap<SoundTrackId, SendTrackHandle>,
    pub(super) playbacks: HashMap<SoundPlaybackId, StaticSoundHandle>,
    pub(super) graph: Option<SoundMixerGraph>,
    pub(super) logical_track_capacity: usize,
    pub(super) logical_voice_capacity: usize,
    pub(super) physical_sub_track_capacity: usize,
    pub(super) physical_send_track_capacity: usize,
    pub(super) physical_voice_capacity: usize,
    pub(super) global_volume_gain: f32,
}

impl<B: Backend> Debug for KiraEngine<B> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("KiraEngine")
            .field("active", &self.is_active())
            .field("track_count", &self.tracks.len())
            .field("send_track_count", &self.send_tracks.len())
            .field("playback_count", &self.playbacks.len())
            .finish()
    }
}

impl<B: Backend> KiraEngine<B> {
    pub(crate) fn inactive() -> Self {
        Self {
            manager: None,
            tracks: HashMap::new(),
            send_tracks: HashMap::new(),
            playbacks: HashMap::new(),
            graph: None,
            logical_track_capacity: DEFAULT_LOGICAL_RESOURCE_CAPACITY,
            logical_voice_capacity: DEFAULT_LOGICAL_RESOURCE_CAPACITY,
            physical_sub_track_capacity: DEFAULT_LOGICAL_RESOURCE_CAPACITY,
            physical_send_track_capacity: DEFAULT_LOGICAL_RESOURCE_CAPACITY,
            physical_voice_capacity: DEFAULT_LOGICAL_RESOURCE_CAPACITY,
            global_volume_gain: 1.0,
        }
    }
}
