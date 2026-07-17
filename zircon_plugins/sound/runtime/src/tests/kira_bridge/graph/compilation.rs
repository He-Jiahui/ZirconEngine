use zircon_runtime::core::framework::sound::{SoundTrackDescriptor, SoundTrackId, SoundTrackSend};

use crate::kira_bridge::compile_graph;

use super::support::graph_with_music_track;

#[test]
fn graph_compiles_to_expected_track_tree() {
    let mut graph = graph_with_music_track();
    graph.tracks[1].sends.push(SoundTrackSend {
        target: SoundTrackId::master(),
        gain: 0.5,
        pre_effects: false,
    });

    let compiled = compile_graph(&graph).unwrap();

    assert_eq!(compiled.tracks().len(), 2);
    assert_eq!(compiled.tracks()[1].parent, Some(SoundTrackId::master()));
    assert_eq!(compiled.tracks()[1].sends.len(), 1);
    assert_eq!(compiled.send_targets(), &[SoundTrackId::master()]);
}

#[test]
fn graph_validation_rejects_missing_parent_before_kira_allocation() {
    let mut graph = graph_with_music_track();
    graph.tracks[1].parent = Some(SoundTrackId::new(99));

    assert!(compile_graph(&graph)
        .unwrap_err()
        .to_string()
        .contains("unknown track"));
}

#[test]
fn graph_validation_rejects_parent_cycle_before_kira_allocation() {
    let mut graph = graph_with_music_track();
    let mut bus = SoundTrackDescriptor::child(SoundTrackId::new(3), "Bus");
    bus.parent = Some(SoundTrackId::new(2));
    graph.tracks.push(bus);
    graph.tracks[1].parent = Some(SoundTrackId::new(3));

    assert!(compile_graph(&graph)
        .unwrap_err()
        .to_string()
        .contains("cycle"));
}
