use crate::core::framework::animation::AnimationTrackPath;

use crate::asset::tests::support::{
    sample_animation_clip_asset, sample_animation_graph_asset, sample_animation_sequence_asset,
    sample_animation_skeleton_asset, sample_animation_state_machine_asset,
};
use crate::asset::{
    AnimationClipAsset, AnimationGraphAsset, AnimationSequenceAsset, AnimationSkeletonAsset,
    AnimationStateMachineAsset,
};

#[test]
fn animation_binary_assets_roundtrip_and_sequence_exposes_shared_track_paths() {
    let skeleton = sample_animation_skeleton_asset();
    let clip = sample_animation_clip_asset();
    let sequence = sample_animation_sequence_asset();
    let graph = sample_animation_graph_asset();
    let state_machine = sample_animation_state_machine_asset();

    assert_eq!(
        AnimationSkeletonAsset::from_bytes(&skeleton.to_bytes().unwrap()).unwrap(),
        skeleton
    );
    assert_eq!(
        AnimationClipAsset::from_bytes(&clip.to_bytes().unwrap()).unwrap(),
        clip
    );
    assert_eq!(
        AnimationSequenceAsset::from_bytes(&sequence.to_bytes().unwrap()).unwrap(),
        sequence
    );
    assert_eq!(
        AnimationGraphAsset::from_bytes(&graph.to_bytes().unwrap()).unwrap(),
        graph
    );
    assert_eq!(
        AnimationStateMachineAsset::from_bytes(&state_machine.to_bytes().unwrap()).unwrap(),
        state_machine
    );

    assert_eq!(
        sequence.track_paths(),
        vec![
            AnimationTrackPath::parse("Root/Hero:Transform.translation").unwrap(),
            AnimationTrackPath::parse("Root/Hero:AnimationPlayer.weight").unwrap(),
        ]
    );
}

#[test]
fn animation_binary_assets_reject_kind_mismatch() {
    let graph_bytes = sample_animation_graph_asset().to_bytes().unwrap();
    let error = AnimationSequenceAsset::from_bytes(&graph_bytes).unwrap_err();

    assert!(error.contains("kind mismatch"), "unexpected error: {error}");
}
