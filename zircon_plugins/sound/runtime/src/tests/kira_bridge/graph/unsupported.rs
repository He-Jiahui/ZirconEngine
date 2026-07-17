use zircon_runtime::core::framework::sound::{
    SoundEffectDescriptor, SoundEffectId, SoundEffectKind, SoundGainEffect, SoundTrackId,
    SoundTrackSend,
};

use crate::kira_bridge::compile_graph;

use super::support::graph_with_music_track;

#[test]
fn m1_graph_rejects_pre_effect_sends_before_kira_allocation() {
    let mut graph = graph_with_music_track();
    graph.tracks[1].sends.push(SoundTrackSend {
        target: SoundTrackId::master(),
        gain: 1.0,
        pre_effects: true,
    });

    assert!(compile_graph(&graph)
        .unwrap_err()
        .to_string()
        .contains("pre-effect sends are enabled by Sound M2"));
}

#[test]
fn m1_graph_rejects_effects_before_kira_effect_mapping_is_delivered() {
    let mut graph = graph_with_music_track();
    graph.tracks[1].effects.push(SoundEffectDescriptor::new(
        SoundEffectId::new(8),
        "Music Gain",
        SoundEffectKind::Gain(SoundGainEffect { gain: 0.5 }),
    ));

    assert!(compile_graph(&graph)
        .unwrap_err()
        .to_string()
        .contains("sound effects are enabled by Sound M2"));
}
