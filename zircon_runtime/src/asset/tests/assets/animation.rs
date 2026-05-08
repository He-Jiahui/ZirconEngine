use crate::core::framework::animation::AnimationTrackPath;

use crate::asset::tests::support::{
    sample_animation_clip_asset, sample_animation_graph_asset, sample_animation_sequence_asset,
    sample_animation_skeleton_asset, sample_animation_state_machine_asset,
};
use crate::asset::{
    AnimationChannelAsset, AnimationClipAsset, AnimationEventTrackAsset, AnimationGraphAsset,
    AnimationGraphNodeAsset, AnimationSequenceAsset, AnimationSequenceTrackAsset,
    AnimationSkeletonAsset, AnimationStateMachineAsset,
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

#[test]
fn animation_assets_report_direct_references() {
    let clip = sample_animation_clip_asset();
    let graph = sample_animation_graph_asset();
    let state_machine = sample_animation_state_machine_asset();

    assert_eq!(
        reference_locators(clip.direct_references()),
        vec!["res://animation/hero.skeleton.zranim".to_string()]
    );
    assert_eq!(
        reference_locators(graph.direct_references()),
        vec!["res://animation/hero.clip.zranim".to_string()]
    );
    assert_eq!(
        reference_locators(state_machine.direct_references()),
        vec!["res://animation/hero.graph.zranim".to_string()]
    );

    let graph_roundtrip = AnimationGraphAsset::from_bytes(&graph.to_bytes().unwrap()).unwrap();
    let machine_roundtrip =
        AnimationStateMachineAsset::from_bytes(&state_machine.to_bytes().unwrap()).unwrap();
    assert_eq!(
        reference_locators(graph_roundtrip.direct_references()),
        vec!["res://animation/hero.clip.zranim".to_string()]
    );
    assert_eq!(
        reference_locators(machine_roundtrip.direct_references()),
        vec!["res://animation/hero.graph.zranim".to_string()]
    );
}

#[test]
fn animation_graph_assets_roundtrip_additive_mask_and_event_metadata() {
    let mut graph = sample_animation_graph_asset();
    graph.nodes.extend([
        AnimationGraphNodeAsset::Additive {
            id: "additive".to_string(),
            base: "idle".to_string(),
            additive: "idle".to_string(),
            weight_parameter: Some("speed".to_string()),
        },
        AnimationGraphNodeAsset::Mask {
            id: "upper_body".to_string(),
            input: "additive".to_string(),
            target_ids: vec!["Root/Spine".to_string(), "Root/Arm".to_string()],
        },
    ]);
    let roundtrip = AnimationGraphAsset::from_bytes(&graph.to_bytes().unwrap()).unwrap();

    assert_eq!(roundtrip, graph);
}

#[test]
fn animation_clip_assets_roundtrip_target_ids_and_event_tracks() {
    let mut clip = sample_animation_clip_asset();
    clip.tracks[0].target_id = Some("Root/Hand".to_string());
    clip.event_tracks.push(AnimationEventTrackAsset {
        target_id: Some("Root/Hand".to_string()),
        event: "footstep".to_string(),
        time_seconds: 0.5,
        payload: Some("stone".to_string()),
    });
    let roundtrip = AnimationClipAsset::from_bytes(&clip.to_bytes().unwrap()).unwrap();

    assert_eq!(roundtrip, clip);
}

#[test]
fn animation_assets_decode_legacy_stream_bytes_with_old_payload_shapes() {
    let clip_bytes = legacy_stream_bytes(
        LegacyAnimationAssetKind::Clip,
        &legacy_clip_from_sample(sample_animation_clip_asset()),
    );
    let clip = AnimationClipAsset::from_bytes(&clip_bytes).unwrap();
    assert_eq!(clip.tracks[0].target_id, None);
    assert!(clip.event_tracks.is_empty());

    let sequence_bytes = legacy_stream_bytes(
        LegacyAnimationAssetKind::Sequence,
        &legacy_sequence_from_sample(sample_animation_sequence_asset()),
    );
    let sequence = AnimationSequenceAsset::from_bytes(&sequence_bytes).unwrap();
    assert_eq!(sequence.bindings[0].target_id, None);

    let graph_bytes = legacy_stream_bytes(
        LegacyAnimationAssetKind::Graph,
        &legacy_graph_from_sample(sample_animation_graph_asset()),
    );
    let graph = AnimationGraphAsset::from_bytes(&graph_bytes).unwrap();
    assert!(graph.nodes.iter().all(|node| matches!(
        node,
        AnimationGraphNodeAsset::Clip { .. }
            | AnimationGraphNodeAsset::Blend { .. }
            | AnimationGraphNodeAsset::Output { .. }
    )));
}

fn reference_locators(references: Vec<crate::asset::AssetReference>) -> Vec<String> {
    references
        .into_iter()
        .map(|reference| reference.locator.to_string())
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum LegacyAnimationAssetKind {
    Skeleton,
    Clip,
    Sequence,
    Graph,
    StateMachine,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct LegacyAnimationBinaryHeader {
    magic: [u8; 8],
    version: u32,
    kind: LegacyAnimationAssetKind,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct LegacyAnimationAssetReferenceBinary {
    uuid: String,
    locator: String,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct LegacyAnimationClipAsset {
    name: Option<String>,
    skeleton: LegacyAnimationAssetReferenceBinary,
    duration_seconds: f32,
    tracks: Vec<LegacyAnimationClipBoneTrackAsset>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct LegacyAnimationClipBoneTrackAsset {
    bone_name: String,
    translation: AnimationChannelAsset,
    rotation: AnimationChannelAsset,
    scale: AnimationChannelAsset,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct LegacyAnimationSequenceAsset {
    name: Option<String>,
    duration_seconds: f32,
    frames_per_second: f32,
    bindings: Vec<LegacyAnimationSequenceBindingAsset>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct LegacyAnimationSequenceBindingAsset {
    entity_path: crate::core::framework::scene::EntityPath,
    tracks: Vec<AnimationSequenceTrackAsset>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct LegacyAnimationGraphAsset {
    name: Option<String>,
    parameters: Vec<crate::asset::AnimationGraphParameterAsset>,
    nodes: Vec<LegacyAnimationGraphNodeBinary>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct LegacyAnimationGraphNodeBinary {
    tag: u8,
    id: String,
    clip: Option<LegacyAnimationAssetReferenceBinary>,
    playback_speed: f32,
    looping: bool,
    inputs: Vec<String>,
    weight_parameter: Option<String>,
    source: String,
}

fn legacy_stream_bytes<T>(kind: LegacyAnimationAssetKind, payload: &T) -> Vec<u8>
where
    T: serde::Serialize,
{
    let mut bytes = bincode::serialize(&LegacyAnimationBinaryHeader {
        magic: *b"ZRANIM01",
        version: 1,
        kind,
    })
    .unwrap();
    bytes.extend(bincode::serialize(payload).unwrap());
    bytes
}

fn legacy_clip_from_sample(clip: AnimationClipAsset) -> LegacyAnimationClipAsset {
    LegacyAnimationClipAsset {
        name: clip.name,
        skeleton: legacy_reference(&clip.skeleton),
        duration_seconds: clip.duration_seconds,
        tracks: clip
            .tracks
            .into_iter()
            .map(|track| LegacyAnimationClipBoneTrackAsset {
                bone_name: track.bone_name,
                translation: track.translation,
                rotation: track.rotation,
                scale: track.scale,
            })
            .collect(),
    }
}

fn legacy_sequence_from_sample(sequence: AnimationSequenceAsset) -> LegacyAnimationSequenceAsset {
    LegacyAnimationSequenceAsset {
        name: sequence.name,
        duration_seconds: sequence.duration_seconds,
        frames_per_second: sequence.frames_per_second,
        bindings: sequence
            .bindings
            .into_iter()
            .map(|binding| LegacyAnimationSequenceBindingAsset {
                entity_path: binding.entity_path,
                tracks: binding.tracks,
            })
            .collect(),
    }
}

fn legacy_graph_from_sample(graph: AnimationGraphAsset) -> LegacyAnimationGraphAsset {
    LegacyAnimationGraphAsset {
        name: graph.name,
        parameters: graph.parameters,
        nodes: graph
            .nodes
            .into_iter()
            .filter_map(|node| match node {
                AnimationGraphNodeAsset::Clip {
                    id,
                    clip,
                    playback_speed,
                    looping,
                } => Some(LegacyAnimationGraphNodeBinary {
                    tag: 0,
                    id,
                    clip: Some(legacy_reference(&clip)),
                    playback_speed,
                    looping,
                    inputs: Vec::new(),
                    weight_parameter: None,
                    source: String::new(),
                }),
                AnimationGraphNodeAsset::Blend {
                    id,
                    inputs,
                    weight_parameter,
                } => Some(LegacyAnimationGraphNodeBinary {
                    tag: 1,
                    id,
                    clip: None,
                    playback_speed: 1.0,
                    looping: false,
                    inputs,
                    weight_parameter,
                    source: String::new(),
                }),
                AnimationGraphNodeAsset::Output { source } => {
                    Some(LegacyAnimationGraphNodeBinary {
                        tag: 2,
                        id: String::new(),
                        clip: None,
                        playback_speed: 1.0,
                        looping: false,
                        inputs: Vec::new(),
                        weight_parameter: None,
                        source,
                    })
                }
                AnimationGraphNodeAsset::Additive { .. } | AnimationGraphNodeAsset::Mask { .. } => {
                    None
                }
            })
            .collect(),
    }
}

fn legacy_reference(
    reference: &crate::asset::AssetReference,
) -> LegacyAnimationAssetReferenceBinary {
    LegacyAnimationAssetReferenceBinary {
        uuid: reference.uuid.to_string(),
        locator: reference.locator.to_string(),
    }
}
