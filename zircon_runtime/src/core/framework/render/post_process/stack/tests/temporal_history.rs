use crate::core::framework::render::{
    AntiAliasSettings, PostProcessEffectKind, PostProcessGraphResourceNames,
    PostProcessStackDescriptor, RenderMotionBlurSettings, RenderPostProcessEffectStackSettings,
};
#[test]
fn taa_resolve_declares_history_velocity_and_output_transfer_input() {
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &Default::default(),
        &Default::default(),
        &RenderPostProcessEffectStackSettings::default(),
        true,
        true,
        &AntiAliasSettings::taa(),
    );

    assert!(
        stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS.to_string())
    );
    assert!(
        stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::TAA_REACTIVE_MASK.to_string())
    );
    assert!(
        stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::SCENE_VELOCITY.to_string())
    );
    let taa_resolve = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::TaaResolve)
        .expect("TAA should enable a temporal resolve node");
    assert!(
        taa_resolve
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCENE_COLOR.to_string())
    );
    assert!(
        taa_resolve
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCENE_DEPTH.to_string())
    );
    assert!(
        taa_resolve
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCENE_VELOCITY.to_string())
    );
    assert!(
        taa_resolve
            .required_inputs
            .contains(&PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS.to_string())
    );
    assert!(
        taa_resolve
            .required_inputs
            .contains(&PostProcessGraphResourceNames::TAA_REACTIVE_MASK.to_string())
    );
    assert!(
        taa_resolve
            .produced_outputs
            .contains(&PostProcessGraphResourceNames::TAA_OUTPUT.to_string())
    );
    assert!(
        taa_resolve
            .produced_outputs
            .contains(&PostProcessGraphResourceNames::TAA_HISTORY_CURRENT.to_string())
    );

    let output_transfer = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::OutputTransfer)
        .expect("TAA stack should keep final composite");
    assert!(
        output_transfer
            .required_inputs
            .contains(&PostProcessGraphResourceNames::TAA_OUTPUT.to_string())
    );
    assert!(
        !output_transfer
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCENE_COLOR.to_string())
    );
    assert!(
        output_transfer
            .after
            .contains(&PostProcessEffectKind::TaaResolve)
    );

    let graph = stack.validated_graph();
    let taa_index = graph
        .nodes
        .iter()
        .position(|node| node.kind == PostProcessEffectKind::TaaResolve)
        .expect("validated graph should keep the TAA resolve node");
    let final_index = graph
        .nodes
        .iter()
        .position(|node| node.kind == PostProcessEffectKind::OutputTransfer)
        .expect("validated graph should keep final composite");
    assert!(taa_index < final_index);
}

#[test]
fn without_history_resources_disables_taa_and_restores_scene_color_input() {
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &Default::default(),
        &Default::default(),
        &RenderPostProcessEffectStackSettings::default(),
        true,
        true,
        &AntiAliasSettings::taa(),
    )
    .without_history_resources();
    let graph = stack.validated_graph();

    assert!(
        !graph
            .nodes
            .iter()
            .any(|node| node.kind == PostProcessEffectKind::TaaResolve)
    );
    assert!(
        !stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS.to_string())
    );
    assert!(
        !stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::SCENE_VELOCITY.to_string())
    );
    assert!(!graph.nodes.iter().any(|node| {
        node.required_inputs
            .contains(&PostProcessGraphResourceNames::TAA_REACTIVE_MASK.to_string())
    }));

    let output_transfer = graph
        .nodes
        .iter()
        .find(|node| node.kind == PostProcessEffectKind::OutputTransfer)
        .expect("history-stripped stack should keep final composite");
    assert!(
        output_transfer
            .required_inputs
            .contains(&PostProcessGraphResourceNames::SCENE_COLOR.to_string())
    );
    assert!(
        !output_transfer
            .required_inputs
            .contains(&PostProcessGraphResourceNames::TAA_OUTPUT.to_string())
    );
}

#[test]
fn without_history_resources_keeps_scene_velocity_for_motion_blur() {
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &Default::default(),
        &Default::default(),
        &RenderPostProcessEffectStackSettings {
            motion_blur: RenderMotionBlurSettings {
                shutter_angle: 0.5,
                samples: 2,
            },
            ..Default::default()
        },
        true,
        true,
        &AntiAliasSettings::taa(),
    )
    .without_history_resources();

    assert!(
        stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::SCENE_VELOCITY.to_string())
    );
    assert!(
        stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX.to_string())
    );
}

#[test]
fn without_history_resources_keeps_scene_velocity_for_hybrid_gi_rejection() {
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &Default::default(),
        &Default::default(),
        &RenderPostProcessEffectStackSettings::default(),
        true,
        true,
        &AntiAliasSettings::taa(),
    )
    .with_hybrid_gi_lighting_input()
    .without_history_resources();

    assert!(
        stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::HYBRID_GI_LIGHTING.to_string())
    );
    assert!(
        stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::SCENE_VELOCITY.to_string())
    );
}

#[test]
fn hybrid_gi_lighting_input_declares_scene_velocity_for_motion_rejection() {
    let stack = PostProcessStackDescriptor::default().with_hybrid_gi_lighting_input();

    assert!(
        stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::HYBRID_GI_LIGHTING.to_string())
    );
    assert!(
        stack
            .initial_resources
            .contains(&PostProcessGraphResourceNames::SCENE_VELOCITY.to_string())
    );
}
