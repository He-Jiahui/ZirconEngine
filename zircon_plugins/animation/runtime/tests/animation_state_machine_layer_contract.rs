use zircon_plugin_animation_runtime::{
    CompiledStateMachineLayers, PoseLayerBlendMode, StateMachineLayerCompileError,
};
use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::animation::{
    AnimationStateAsset, AnimationStateMachineAsset, AnimationStateMachineLayerAsset,
    AnimationStateMachineLayerBlendModeAsset,
};

fn reference(name: &str) -> AssetReference {
    AssetReference::from_locator(
        AssetUri::parse(&format!("res://animation/{name}.machine.zranim")).unwrap(),
    )
}

fn base() -> AnimationStateMachineAsset {
    AnimationStateMachineAsset {
        name: Some("Base".into()),
        entry_state: "Idle".into(),
        states: vec![AnimationStateAsset::graph_ref(
            "Idle",
            reference("idle-graph"),
        )],
        transitions: Vec::new(),
        layers: Vec::new(),
    }
}

fn layered() -> AnimationStateMachineAsset {
    AnimationStateMachineAsset {
        layers: vec![
            AnimationStateMachineLayerAsset {
                name: "upper".into(),
                state_machine: reference("upper"),
                weight: 0.75,
                blend_mode: AnimationStateMachineLayerBlendModeAsset::Override,
                mask_weights: vec![0.0, 1.0],
            },
            AnimationStateMachineLayerAsset {
                name: "recoil".into(),
                state_machine: reference("recoil"),
                weight: 0.25,
                blend_mode: AnimationStateMachineLayerBlendModeAsset::Additive,
                mask_weights: Vec::new(),
            },
        ],
        ..base()
    }
}

#[test]
fn layered_machine_compiles_dense_mask_and_blend_modes() {
    let source = layered();

    let compiled = CompiledStateMachineLayers::compile(&source).unwrap();

    assert!(compiled.base().layers.is_empty());
    assert_eq!(compiled.layers().len(), 2);
    assert_eq!(compiled.layers()[0].name(), "upper");
    assert_eq!(
        compiled.layers()[0].machine().locator.to_string(),
        "res://animation/upper.machine.zranim"
    );
    assert_eq!(compiled.layers()[0].weight(), 0.75);
    assert_eq!(
        compiled.layers()[0].blend_mode(),
        PoseLayerBlendMode::Override
    );
    assert_eq!(compiled.layers()[0].mask().unwrap().as_slice(), &[0.0, 1.0]);
    assert_eq!(
        compiled.layers()[1].blend_mode(),
        PoseLayerBlendMode::Additive
    );
    assert!(compiled.layers()[1].mask().is_none());
}

#[test]
fn layered_machine_rejects_invalid_weight_and_mask() {
    let invalid_weight = AnimationStateMachineAsset {
        layers: vec![AnimationStateMachineLayerAsset {
            name: "bad".into(),
            state_machine: reference("bad"),
            weight: 1.5,
            blend_mode: Default::default(),
            mask_weights: Vec::new(),
        }],
        ..base()
    };
    assert!(matches!(
        CompiledStateMachineLayers::compile(&invalid_weight),
        Err(StateMachineLayerCompileError::InvalidWeight { .. })
    ));

    let invalid_mask = AnimationStateMachineAsset {
        layers: vec![AnimationStateMachineLayerAsset {
            name: "bad".into(),
            state_machine: reference("bad"),
            weight: 1.0,
            blend_mode: Default::default(),
            mask_weights: vec![f32::NAN],
        }],
        ..base()
    };
    assert!(matches!(
        CompiledStateMachineLayers::compile(&invalid_mask),
        Err(StateMachineLayerCompileError::InvalidMask { .. })
    ));
}

#[test]
fn layered_machine_reports_base_and_layer_references_in_order() {
    let source = AnimationStateMachineAsset {
        layers: vec![AnimationStateMachineLayerAsset {
            name: "upper".into(),
            state_machine: reference("upper"),
            weight: 1.0,
            blend_mode: Default::default(),
            mask_weights: Vec::new(),
        }],
        ..base()
    };

    assert_eq!(
        source
            .direct_references()
            .into_iter()
            .map(|reference| reference.locator.to_string())
            .collect::<Vec<_>>(),
        vec![
            "res://animation/idle-graph.machine.zranim",
            "res://animation/upper.machine.zranim",
        ]
    );
}

#[test]
fn layered_machine_binary_roundtrip_preserves_layers() {
    let source = layered();

    let decoded = AnimationStateMachineAsset::from_bytes(&source.to_bytes().unwrap()).unwrap();

    assert_eq!(decoded, source);
}
