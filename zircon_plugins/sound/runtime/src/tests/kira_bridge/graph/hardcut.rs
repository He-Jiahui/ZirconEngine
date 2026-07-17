use zircon_runtime::core::framework::sound::{SoundEffectKind, SoundTrackId};

use crate::kira_bridge::compile_graph;

use super::support::{authored_send_graph, authored_sidechain_graph};

#[test]
fn post_effect_track_send_compiles_to_kira_send_contract() {
    let graph = authored_send_graph(false);
    let compiled = compile_graph(&graph).unwrap();
    let music = compiled
        .tracks()
        .iter()
        .find(|track| track.id == SoundTrackId::new(2))
        .unwrap();

    assert_eq!(music.sends.len(), 1);
    assert_eq!(music.sends[0].target, SoundTrackId::new(3));
    assert!(!music.sends[0].pre_effects);
    assert_eq!(compiled.send_targets(), &[SoundTrackId::new(3)]);
}

#[test]
fn inactive_graph_stores_pre_effect_send_before_kira_reports_m2_surface() {
    let graph = authored_send_graph(true);
    let music = graph
        .tracks
        .iter()
        .find(|track| track.id == SoundTrackId::new(2))
        .unwrap();

    assert!(music.sends[0].pre_effects);
    assert!(compile_graph(&graph)
        .unwrap_err()
        .to_string()
        .contains("pre-effect sends are enabled by Sound M2"));
}

#[test]
fn inactive_graph_stores_sidechain_effect_before_kira_reports_m2_surface() {
    let graph = authored_sidechain_graph();
    let target = graph
        .tracks
        .iter()
        .find(|track| track.id == SoundTrackId::new(2))
        .unwrap();

    assert!(matches!(
        &target.effects[0].kind,
        SoundEffectKind::Compressor(effect)
            if effect.sidechain.as_ref().is_some_and(|sidechain| {
                sidechain.track == SoundTrackId::new(3) && sidechain.pre_effects
            })
    ));
    assert!(compile_graph(&graph)
        .unwrap_err()
        .to_string()
        .contains("sound effects are enabled by Sound M2"));
}
