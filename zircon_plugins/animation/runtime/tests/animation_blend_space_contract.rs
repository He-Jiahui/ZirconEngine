use std::f32::consts::TAU;

use zircon_plugin_animation_runtime::compile_animation_state_machine_runtime;
use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::animation::{
    AnimationBlendSpace1DAsset, AnimationBlendSpace1DSampleAsset, AnimationBlendSpace2DAsset,
    AnimationBlendSpace2DSampleAsset, AnimationParameterMap, AnimationParameterValue,
    AnimationStateAsset, AnimationStateKindAsset, AnimationStateMachineAsset,
};
use zircon_runtime::core::math::Vec2;

#[test]
fn blend_space_1d_interpolates_sorted_compiler_samples() {
    let source = machine(AnimationStateKindAsset::BlendSpace1D(
        AnimationBlendSpace1DAsset {
            parameter: "speed".into(),
            samples: vec![
                sample_1d(1.0, "run"),
                sample_1d(-1.0, "reverse"),
                sample_1d(0.0, "idle"),
            ],
        },
    ));
    let compiled = compile_animation_state_machine_runtime(&source).unwrap();

    assert_eq!(
        sample_1d_paths(&compiled, -2.0),
        vec![("reverse".to_string(), 1.0)]
    );
    assert_eq!(
        sample_1d_paths(&compiled, 0.25),
        vec![("idle".to_string(), 0.75), ("run".to_string(), 0.25)]
    );
    assert_eq!(
        sample_1d_paths(&compiled, 2.0),
        vec![("run".to_string(), 1.0)]
    );
}

#[test]
fn blend_space_2d_weights_and_hull_projection_remain_normalized() {
    let source = machine(AnimationStateKindAsset::BlendSpace2D(
        AnimationBlendSpace2DAsset {
            parameter: "direction".into(),
            samples: vec![
                sample_2d([1.0, 1.0], "north-east"),
                sample_2d([0.0, 0.0], "origin"),
                sample_2d([1.0, 0.0], "east"),
                sample_2d([0.0, 1.0], "north"),
            ],
        },
    ));
    let compiled = compile_animation_state_machine_runtime(&source).unwrap();

    for point in [[0.25, 0.25], [2.0, 2.0]] {
        let samples = sample_2d_weights(&compiled, point);
        assert!(!samples.is_empty());
        assert!(samples.iter().all(|weight| *weight >= 0.0));
        assert!((samples.iter().sum::<f32>() - 1.0).abs() <= 1.0e-6);
    }
}

#[test]
fn blend_space_2d_accepts_ninety_six_cocircular_samples() {
    const SAMPLE_COUNT: usize = 96;
    let samples = (0..SAMPLE_COUNT)
        .map(|index| {
            let angle = TAU * index as f32 / SAMPLE_COUNT as f32;
            sample_2d([angle.cos(), angle.sin()], &format!("direction-{index}"))
        })
        .collect();
    let source = machine(AnimationStateKindAsset::BlendSpace2D(
        AnimationBlendSpace2DAsset {
            parameter: "direction".into(),
            samples,
        },
    ));

    let compiled = compile_animation_state_machine_runtime(&source).unwrap();
    let weights = sample_2d_weights(&compiled, [0.0, 0.0]);

    assert!(!weights.is_empty());
    assert!((weights.iter().sum::<f32>() - 1.0).abs() <= 1.0e-5);
}

#[test]
fn blend_space_2d_sampling_is_independent_of_authored_coordinate_scale() {
    for scale in [1.0e-20, 1.0e30] {
        let source = machine(AnimationStateKindAsset::BlendSpace2D(
            AnimationBlendSpace2DAsset {
                parameter: "direction".into(),
                samples: vec![
                    sample_2d([0.0, 0.0], "origin"),
                    sample_2d([scale, 0.0], "east"),
                    sample_2d([0.0, scale], "north"),
                ],
            },
        ));
        let compiled = compile_animation_state_machine_runtime(&source).unwrap();

        let mut weights = sample_2d_weights(&compiled, [scale * 0.25, scale * 0.25]);
        weights.sort_by(f32::total_cmp);

        assert_eq!(weights.len(), 3, "scale={scale:e}");
        assert!((weights[0] - 0.25).abs() <= 1.0e-5, "scale={scale:e}");
        assert!((weights[1] - 0.25).abs() <= 1.0e-5, "scale={scale:e}");
        assert!((weights[2] - 0.5).abs() <= 1.0e-5, "scale={scale:e}");
    }
}

fn machine(kind: AnimationStateKindAsset) -> AnimationStateMachineAsset {
    AnimationStateMachineAsset {
        name: Some("BlendSpace".into()),
        entry_state: "Blend".into(),
        states: vec![AnimationStateAsset {
            name: "Blend".into(),
            kind,
        }],
        transitions: Vec::new(),
        layers: Vec::new(),
    }
}

fn sample_1d(position: f32, name: &str) -> AnimationBlendSpace1DSampleAsset {
    AnimationBlendSpace1DSampleAsset {
        position,
        graph: graph(name),
    }
}

fn sample_2d(position: [f32; 2], name: &str) -> AnimationBlendSpace2DSampleAsset {
    AnimationBlendSpace2DSampleAsset {
        position: Vec2::from_array(position),
        graph: graph(name),
    }
}

fn graph(name: &str) -> AssetReference {
    AssetReference::from_locator(
        AssetUri::parse(&format!("res://animation/{name}.zranim")).unwrap(),
    )
}

fn sample_1d_paths(
    compiled: &zircon_plugin_animation_runtime::CompiledAnimationStateMachine,
    value: f32,
) -> Vec<(String, f32)> {
    compiled
        .evaluate(
            None,
            &AnimationParameterMap::from([(
                "speed".into(),
                AnimationParameterValue::Scalar(value),
            )]),
        )
        .graph_samples()
        .map(|(graph, weight)| {
            let path = graph.locator.to_string();
            let name = path
                .strip_prefix("res://animation/")
                .and_then(|path| path.strip_suffix(".zranim"))
                .unwrap()
                .to_string();
            (name, weight)
        })
        .collect()
}

fn sample_2d_weights(
    compiled: &zircon_plugin_animation_runtime::CompiledAnimationStateMachine,
    value: [f32; 2],
) -> Vec<f32> {
    compiled
        .evaluate(
            None,
            &AnimationParameterMap::from([(
                "direction".into(),
                AnimationParameterValue::Vec2(value),
            )]),
        )
        .graph_samples()
        .map(|(_, weight)| weight)
        .collect()
}
