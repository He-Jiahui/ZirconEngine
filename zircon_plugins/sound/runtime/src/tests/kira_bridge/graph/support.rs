use zircon_runtime::core::framework::sound::{
    SoundCompressorEffect, SoundEffectDescriptor, SoundEffectId, SoundEffectKind, SoundMixerGraph,
    SoundMixerGraphManager, SoundSidechainInput, SoundTrackDescriptor, SoundTrackId,
    SoundTrackSend,
};

use crate::DefaultSoundManager;

pub(super) fn graph_with_music_track() -> SoundMixerGraph {
    let mut graph = SoundMixerGraph::default_stereo(48_000);
    graph
        .tracks
        .push(SoundTrackDescriptor::child(SoundTrackId::new(2), "Music"));
    graph
}

pub(super) fn authored_send_graph(pre_effects: bool) -> SoundMixerGraph {
    let sound = DefaultSoundManager::default();
    let music = SoundTrackId::new(2);
    let aux = SoundTrackId::new(3);
    sound
        .add_or_update_track(SoundTrackDescriptor::child(music, "Music"))
        .unwrap();
    sound
        .add_or_update_track(SoundTrackDescriptor::child(aux, "Aux"))
        .unwrap();
    sound
        .add_or_update_track_send(
            music,
            SoundTrackSend {
                target: aux,
                gain: 0.5,
                pre_effects,
            },
        )
        .unwrap();
    sound.mixer_snapshot().unwrap().graph
}

pub(super) fn authored_sidechain_graph() -> SoundMixerGraph {
    let sound = DefaultSoundManager::default();
    let target = SoundTrackId::new(2);
    let key = SoundTrackId::new(3);
    sound
        .add_or_update_track(SoundTrackDescriptor::child(target, "Target"))
        .unwrap();
    sound
        .add_or_update_track(SoundTrackDescriptor::child(key, "Key"))
        .unwrap();
    sound
        .add_or_update_effect(
            target,
            SoundEffectDescriptor::new(
                SoundEffectId::new(77),
                "Sidechain Compressor",
                SoundEffectKind::Compressor(SoundCompressorEffect {
                    threshold_db: -18.0,
                    ratio: 8.0,
                    attack_ms: 1.0,
                    release_ms: 50.0,
                    makeup_gain_db: 0.0,
                    sidechain: Some(SoundSidechainInput {
                        track: key,
                        pre_effects: true,
                    }),
                }),
            ),
        )
        .unwrap();
    sound.mixer_snapshot().unwrap().graph
}
