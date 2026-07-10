use std::sync::Arc;

use zircon_plugin_animation_runtime::{
    AnimationGraphCompileError, CompiledAnimationGraph, SkeletonTargetTable,
};
use zircon_runtime::asset::{
    AnimationGraphAsset, AnimationGraphNodeAsset, AnimationGraphParameterAsset,
    AnimationSkeletonAsset, AnimationSkeletonBoneAsset, AssetReference, AssetUri,
};
use zircon_runtime::core::framework::animation::{AnimationParameterMap, AnimationParameterValue};

#[test]
fn compiled_graph_keeps_dense_nodes_parameters_and_mask_rows_after_source_mutation() {
    let skeleton = skeleton(&[("Root", None), ("Spine", Some(0)), ("Hand", Some(1))]);
    let targets = Arc::new(SkeletonTargetTable::compile(&skeleton).unwrap());
    let mut graph = AnimationGraphAsset {
        name: Some("masked".into()),
        parameters: vec![AnimationGraphParameterAsset {
            name: "weight".into(),
            default_value: AnimationParameterValue::Scalar(0.25),
        }],
        nodes: vec![
            clip_node("idle", "res://animations/idle.zanim"),
            clip_node("run", "res://animations/run.zanim"),
            AnimationGraphNodeAsset::Blend {
                id: "locomotion".into(),
                inputs: vec!["idle".into(), "run".into()],
                weight_parameter: Some("weight".into()),
            },
            AnimationGraphNodeAsset::Mask {
                id: "upper".into(),
                input: "locomotion".into(),
                target_ids: vec!["Root/Spine/Hand".into()],
            },
            AnimationGraphNodeAsset::Output {
                source: "upper".into(),
            },
        ],
    };
    let compiled = CompiledAnimationGraph::compile(&graph, targets).unwrap();

    graph.parameters[0].name = "renamed".into();
    if let AnimationGraphNodeAsset::Blend { inputs, .. } = &mut graph.nodes[2] {
        inputs[0] = "missing".into();
    }
    if let AnimationGraphNodeAsset::Mask { target_ids, .. } = &mut graph.nodes[3] {
        target_ids[0] = "Missing/Bone".into();
    }

    let evaluation = compiled.evaluate(&AnimationParameterMap::from([(
        "weight".into(),
        AnimationParameterValue::Scalar(0.8),
    )]));
    assert_eq!(compiled.node_count(), 4);
    assert_eq!(compiled.parameter_count(), 1);
    assert_eq!(evaluation.clips().len(), 2);
    assert!((evaluation.clips()[0].weight() - 0.2).abs() < 0.0001);
    assert!((evaluation.clips()[1].weight() - 0.8).abs() < 0.0001);
    assert_eq!(evaluation.clips()[0].target_mask(), &[false, false, true]);
}

#[test]
fn compiled_graph_rejects_missing_edges_cycles_and_ambiguous_mask_leaves() {
    let targets = Arc::new(
        SkeletonTargetTable::compile(&skeleton(&[
            ("Root", None),
            ("Left", Some(0)),
            ("Hand", Some(1)),
            ("Right", Some(0)),
            ("Hand", Some(3)),
        ]))
        .unwrap(),
    );
    let missing = AnimationGraphAsset {
        name: None,
        parameters: vec![],
        nodes: vec![AnimationGraphNodeAsset::Output {
            source: "missing".into(),
        }],
    };
    assert!(matches!(
        CompiledAnimationGraph::compile(&missing, Arc::clone(&targets)),
        Err(AnimationGraphCompileError::MissingNode { .. })
    ));

    let cycle = AnimationGraphAsset {
        name: None,
        parameters: vec![],
        nodes: vec![
            AnimationGraphNodeAsset::Blend {
                id: "a".into(),
                inputs: vec!["b".into()],
                weight_parameter: None,
            },
            AnimationGraphNodeAsset::Blend {
                id: "b".into(),
                inputs: vec!["a".into()],
                weight_parameter: None,
            },
            AnimationGraphNodeAsset::Output { source: "a".into() },
        ],
    };
    assert!(matches!(
        CompiledAnimationGraph::compile(&cycle, Arc::clone(&targets)),
        Err(AnimationGraphCompileError::Cycle { .. })
    ));

    let ambiguous = AnimationGraphAsset {
        name: None,
        parameters: vec![],
        nodes: vec![
            clip_node("clip", "res://animations/clip.zanim"),
            AnimationGraphNodeAsset::Mask {
                id: "mask".into(),
                input: "clip".into(),
                target_ids: vec!["Hand".into()],
            },
            AnimationGraphNodeAsset::Output {
                source: "mask".into(),
            },
        ],
    };
    assert!(matches!(
        CompiledAnimationGraph::compile(&ambiguous, targets),
        Err(AnimationGraphCompileError::AmbiguousMaskTarget { .. })
    ));
}

fn clip_node(id: &str, uri: &str) -> AnimationGraphNodeAsset {
    AnimationGraphNodeAsset::Clip {
        id: id.into(),
        clip: AssetReference::from_locator(AssetUri::parse(uri).unwrap()),
        playback_speed: 1.0,
        looping: true,
    }
}

fn skeleton(bones: &[(&str, Option<u32>)]) -> AnimationSkeletonAsset {
    AnimationSkeletonAsset {
        name: None,
        bones: bones
            .iter()
            .map(|(name, parent_index)| AnimationSkeletonBoneAsset {
                name: (*name).into(),
                parent_index: *parent_index,
                local_translation: [0.0, 0.0, 0.0],
                local_rotation: [0.0, 0.0, 0.0, 1.0],
                local_scale: [1.0, 1.0, 1.0],
            })
            .collect(),
    }
}
