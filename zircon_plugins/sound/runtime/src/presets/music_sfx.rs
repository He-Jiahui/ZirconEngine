use zircon_runtime::core::framework::sound::{
    SoundEffectDescriptor, SoundEffectId, SoundEffectKind, SoundGainEffect, SoundLimiterEffect,
    SoundMixerGraph, SoundTrackDescriptor, SoundTrackId,
};

use crate::SoundConfig;

use super::default::default_graph;

pub(crate) fn music_sfx_graph(config: &SoundConfig) -> SoundMixerGraph {
    let mut graph = default_graph(config);
    graph.tracks = vec![
        SoundTrackDescriptor::master(),
        SoundTrackDescriptor::child(SoundTrackId::new(2), "Music"),
        SoundTrackDescriptor::child(SoundTrackId::new(3), "SFX"),
        SoundTrackDescriptor::child(SoundTrackId::new(4), "Ambience"),
    ];
    graph.tracks[0].effects.push(SoundEffectDescriptor::new(
        SoundEffectId::new(1),
        "Master Limiter",
        SoundEffectKind::Limiter(SoundLimiterEffect { ceiling: 1.0 }),
    ));
    graph.tracks[1].effects.push(SoundEffectDescriptor::new(
        SoundEffectId::new(2),
        "Music Trim",
        SoundEffectKind::Gain(SoundGainEffect { gain: 0.9 }),
    ));
    graph
}
