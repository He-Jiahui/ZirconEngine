use std::collections::HashMap;

use kira::backend::mock::MockBackend;
use zircon_runtime::core::framework::sound::{
    SoundClipId, SoundMixerGraph, SoundSourceDescriptor, SoundTrackId,
};

use crate::engine::{LoadedClip, SourceVoice};
use crate::kira_bridge::KiraEngine;
use crate::tests::test_clip_with_rate;

use super::super::lifecycle::support::mock_settings;

pub(super) struct MockSourceRuntime {
    pub(super) engine: KiraEngine<MockBackend>,
    pub(super) clips: HashMap<SoundClipId, LoadedClip>,
    pub(super) next_playback_id: u64,
    pub(super) voice: SourceVoice,
}

impl MockSourceRuntime {
    pub(super) fn inactive_clip() -> Self {
        let clip_id = SoundClipId::new(1);
        let clip = LoadedClip::new(test_clip_with_rate(
            "res://sound/source-runtime.wav",
            10,
            &[0.1, 0.2, 0.3, 0.4],
        ))
        .unwrap();
        Self {
            engine: KiraEngine::inactive(),
            clips: HashMap::from([(clip_id, clip)]),
            next_playback_id: 0,
            voice: SourceVoice::new(SoundSourceDescriptor::clip(clip_id)),
        }
    }

    pub(super) fn activate(&mut self) {
        self.engine.activate(mock_settings(10)).unwrap();
        self.engine
            .sync_graph(&SoundMixerGraph::default_stereo(10))
            .unwrap();
        assert!(self.engine.contains_track(SoundTrackId::master()));
    }
}
