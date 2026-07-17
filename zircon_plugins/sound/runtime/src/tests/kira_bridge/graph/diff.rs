use kira::backend::mock::MockBackend;
use zircon_runtime::core::framework::sound::{SoundTrackDescriptor, SoundTrackId};

use crate::kira_bridge::{diff_graphs, GraphSyncAction, KiraEngine, PARAMETER_TWEEN_DURATION};

use super::super::lifecycle::support::mock_settings;
use super::support::graph_with_music_track;

#[test]
fn graph_edit_applies_minimal_diff() {
    let before = graph_with_music_track();
    let mut after = before.clone();
    after
        .tracks
        .push(SoundTrackDescriptor::child(SoundTrackId::new(3), "SFX"));

    let diff = diff_graphs(&before, &after).unwrap();

    assert_eq!(diff.actions().len(), 1);
    assert!(matches!(
        diff.actions()[0],
        GraphSyncAction::AddTrack { ref track } if track.id == SoundTrackId::new(3)
    ));

    let mut engine = KiraEngine::<MockBackend>::inactive();
    engine.activate(mock_settings(48_000)).unwrap();
    engine.sync_graph(&before).unwrap();
    engine.sync_graph(&after).unwrap();
    assert_eq!(engine.track_count(), 3);
    assert!(engine.contains_track(SoundTrackId::new(3)));
}

#[test]
fn param_change_is_tweened() {
    let before = graph_with_music_track();
    let mut after = before.clone();
    after.tracks[1].controls.gain = 0.5;

    let diff = diff_graphs(&before, &after).unwrap();
    let GraphSyncAction::SetTrackVolume { tween, .. } = diff.actions()[0] else {
        panic!("gain-only graph edit must be a volume update");
    };

    assert_eq!(tween.duration, PARAMETER_TWEEN_DURATION);
}

#[test]
fn send_gain_change_updates_the_existing_kira_route_without_rebuilding_the_track() {
    let mut before = graph_with_music_track();
    before.tracks[1]
        .sends
        .push(zircon_runtime::core::framework::sound::SoundTrackSend {
            target: SoundTrackId::master(),
            gain: 0.25,
            pre_effects: false,
        });
    let mut after = before.clone();
    after.tracks[1].sends[0].gain = 1.0;

    let diff = diff_graphs(&before, &after).unwrap();

    assert!(diff.actions().iter().any(|action| matches!(
        action,
        GraphSyncAction::SetTrackSendVolume {
            track,
            target,
            linear_gain,
            ..
        } if *track == SoundTrackId::new(2)
            && *target == SoundTrackId::master()
            && (*linear_gain - 1.0).abs() < 1.0e-6
    )));
    assert!(!diff.actions().iter().any(|action| matches!(
        action,
        GraphSyncAction::RebuildSubtree { .. } | GraphSyncAction::RebuildGraph
    )));
}

#[test]
fn parent_move_with_new_descendant_rebuilds_subtree_once() {
    let mut before = graph_with_music_track();
    before
        .tracks
        .push(SoundTrackDescriptor::child(SoundTrackId::new(3), "Bus"));
    let mut after = before.clone();
    after.tracks[1].parent = Some(SoundTrackId::new(3));
    let mut child = SoundTrackDescriptor::child(SoundTrackId::new(4), "Stem");
    child.parent = Some(SoundTrackId::new(2));
    after.tracks.push(child);

    let diff = diff_graphs(&before, &after).unwrap();
    assert_eq!(diff.actions().len(), 1);
    assert!(matches!(
        diff.actions()[0],
        GraphSyncAction::RebuildSubtree { root } if root == SoundTrackId::new(3)
    ));

    let mut engine = KiraEngine::<MockBackend>::inactive();
    engine.activate(mock_settings(48_000)).unwrap();
    engine.sync_graph(&before).unwrap();
    engine.sync_graph(&after).unwrap();
    assert_eq!(engine.track_count(), 4);
    assert!(engine.contains_track(SoundTrackId::new(4)));
}

#[test]
fn reversing_a_parent_chain_rebuilds_the_new_top_level_subtree() {
    let mut before = graph_with_music_track();
    let mut child = SoundTrackDescriptor::child(SoundTrackId::new(3), "Child");
    child.parent = Some(SoundTrackId::new(2));
    before.tracks.push(child);
    let mut after = before.clone();
    after.tracks[1].parent = Some(SoundTrackId::new(3));
    after.tracks[2].parent = Some(SoundTrackId::master());

    let diff = diff_graphs(&before, &after).unwrap();

    assert!(diff.actions().iter().any(|action| matches!(
        action,
        GraphSyncAction::RebuildSubtree { root } if *root == SoundTrackId::new(3)
    )));

    let mut engine = KiraEngine::<MockBackend>::inactive();
    engine.activate(mock_settings(48_000)).unwrap();
    engine.sync_graph(&before).unwrap();
    engine.sync_graph(&after).unwrap();
    assert_eq!(engine.installed_graph_for_test(), Some(&after));
    assert_eq!(engine.track_count(), 3);
    assert!(engine.contains_track(SoundTrackId::new(2)));
    assert!(engine.contains_track(SoundTrackId::new(3)));
}
