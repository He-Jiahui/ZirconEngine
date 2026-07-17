use std::sync::Arc;

use kira::{
    backend::mock::{MockBackend, MockBackendSettings},
    sound::static_sound::{StaticSoundData, StaticSoundSettings},
    AudioManagerSettings, Frame,
};
use zircon_runtime::core::framework::sound::{
    SoundMixerGraph, SoundSourceDescriptor, SoundSourceId, SoundSourceManager,
    SoundTrackDescriptor, SoundTrackId,
};

use crate::tests::test_clip_with_rate;
use crate::DefaultSoundManager;

pub(in crate::tests::kira_bridge) fn mock_settings(
    sample_rate: u32,
) -> AudioManagerSettings<MockBackend> {
    AudioManagerSettings {
        backend_settings: MockBackendSettings { sample_rate },
        ..AudioManagerSettings::default()
    }
}

pub(super) fn graph_with_music_track(sample_rate: u32) -> SoundMixerGraph {
    let mut graph = SoundMixerGraph::default_stereo(sample_rate);
    graph
        .tracks
        .push(SoundTrackDescriptor::child(SoundTrackId::new(2), "Music"));
    graph
}

pub(super) fn silent_stereo_clip(sample_rate: u32) -> StaticSoundData {
    StaticSoundData {
        sample_rate,
        frames: Arc::from([Frame::ZERO; 32]),
        settings: StaticSoundSettings::default(),
        slice: None,
    }
}

pub(super) fn inactive_source_fixture() -> (DefaultSoundManager, SoundSourceId) {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip_with_rate(
        "res://sound/source-controls-kira.wav",
        10,
        &[0.1, 0.2, 0.3, 0.4],
    ));
    let source = sound
        .create_source(SoundSourceDescriptor::clip(clip))
        .unwrap();
    (sound, source)
}
