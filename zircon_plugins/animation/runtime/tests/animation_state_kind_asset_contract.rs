use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::animation::{
    AnimationBlendSpace1DAsset, AnimationBlendSpace1DSampleAsset, AnimationBlendSpace2DAsset,
    AnimationBlendSpace2DSampleAsset, AnimationStateAsset, AnimationStateKindAsset,
    AnimationStateMachineAsset,
};
use zircon_runtime::core::math::Vec2;

fn reference(locator: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(locator).expect("test animation locator"))
}

#[test]
fn state_kind_binary_roundtrip_preserves_variants_and_direct_references() {
    let machine = AnimationStateMachineAsset {
        name: Some("StateKinds".to_string()),
        entry_state: "Clip".to_string(),
        states: vec![
            AnimationStateAsset {
                name: "Clip".to_string(),
                kind: AnimationStateKindAsset::Clip {
                    clip: reference("res://animation/direct.clip.zranim"),
                },
            },
            AnimationStateAsset {
                name: "Blend1D".to_string(),
                kind: AnimationStateKindAsset::BlendSpace1D(AnimationBlendSpace1DAsset {
                    parameter: "speed".to_string(),
                    samples: vec![AnimationBlendSpace1DSampleAsset {
                        position: 0.0,
                        graph: reference("res://animation/idle.graph.zranim"),
                    }],
                }),
            },
            AnimationStateAsset {
                name: "Blend2D".to_string(),
                kind: AnimationStateKindAsset::BlendSpace2D(AnimationBlendSpace2DAsset {
                    parameter: "direction".to_string(),
                    samples: vec![AnimationBlendSpace2DSampleAsset {
                        position: Vec2::new(1.0, 0.0),
                        graph: reference("res://animation/run.graph.zranim"),
                    }],
                }),
            },
            AnimationStateAsset {
                name: "Nested".to_string(),
                kind: AnimationStateKindAsset::SubMachine {
                    state_machine: reference("res://animation/nested.machine.zranim"),
                },
            },
            AnimationStateAsset::graph_ref(
                "Graph",
                reference("res://animation/legacy.graph.zranim"),
            ),
        ],
        transitions: Vec::new(),
        layers: Vec::new(),
    };

    let roundtrip = AnimationStateMachineAsset::from_bytes(&machine.to_bytes().unwrap()).unwrap();

    assert_eq!(roundtrip, machine);
    assert_eq!(
        roundtrip
            .direct_references()
            .into_iter()
            .map(|reference| reference.locator.to_string())
            .collect::<Vec<_>>(),
        vec![
            "res://animation/direct.clip.zranim",
            "res://animation/idle.graph.zranim",
            "res://animation/run.graph.zranim",
            "res://animation/nested.machine.zranim",
            "res://animation/legacy.graph.zranim",
        ]
    );
}
