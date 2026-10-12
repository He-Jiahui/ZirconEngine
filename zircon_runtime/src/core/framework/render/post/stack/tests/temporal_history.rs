use crate::core::framework::render::{
    AntiAliasSettings, PostProcessEffectKind, PostProcessGraphResourceNames,
    PostProcessStackDescriptor, RenderDepthOfFieldSettings, RenderMotionBlurSettings,
    RenderPostProcessEffectStackSettings,
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

    assert!(stack
        .initial_resources
        .contains(&PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS.to_string()));
    assert!(stack
        .initial_resources
        .contains(&PostProcessGraphResourceNames::TAA_REACTIVE_MASK.to_string()));
    assert!(stack
        .initial_resources
        .contains(&PostProcessGraphResourceNames::SCENE_VELOCITY.to_string()));
    let taa_resolve = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::TaaResolve)
        .expect("TAA should enable a temporal resolve node");
    assert!(taa_resolve
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCENE_COLOR.to_string()));
    assert!(taa_resolve
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCENE_DEPTH.to_string()));
    assert!(taa_resolve
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCENE_VELOCITY.to_string()));
    assert!(taa_resolve
        .required_inputs
        .contains(&PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS.to_string()));
    assert!(taa_resolve
        .required_inputs
        .contains(&PostProcessGraphResourceNames::TAA_REACTIVE_MASK.to_string()));
    assert!(taa_resolve
        .produced_outputs
        .contains(&PostProcessGraphResourceNames::TAA_OUTPUT.to_string()));
    assert!(taa_resolve
        .produced_outputs
        .contains(&PostProcessGraphResourceNames::TAA_HISTORY_CURRENT.to_string()));

    let output_transfer = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::OutputTransfer)
        .expect("TAA stack should keep final composite");
    assert!(output_transfer
        .required_inputs
        .contains(&PostProcessGraphResourceNames::TAA_OUTPUT.to_string()));
    assert!(!output_transfer
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCENE_COLOR.to_string()));
    assert!(output_transfer
        .after
        .contains(&PostProcessEffectKind::TaaResolve));

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
fn depth_of_field_precedes_temporal_reconstruction_and_feeds_taa() {
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &Default::default(),
        &Default::default(),
        &RenderPostProcessEffectStackSettings {
            depth_of_field: RenderDepthOfFieldSettings {
                aperture: 0.75,
                max_blur_radius: 4.0,
                ..Default::default()
            },
            ..Default::default()
        },
        true,
        true,
        &AntiAliasSettings::taa(),
    );

    let depth_of_field = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::DepthOfField)
        .expect("depth of field should be present");
    assert!(depth_of_field
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCENE_COLOR.to_string()));
    assert!(!depth_of_field
        .required_inputs
        .contains(&PostProcessGraphResourceNames::TAA_OUTPUT.to_string()));

    let taa_resolve = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::TaaResolve)
        .expect("TAA should be present");
    assert!(taa_resolve
        .required_inputs
        .contains(&PostProcessGraphResourceNames::DEPTH_OF_FIELDED.to_string()));
    assert!(!taa_resolve
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCENE_COLOR.to_string()));
    assert!(taa_resolve
        .after
        .contains(&PostProcessEffectKind::DepthOfField));

    let graph = stack.validated_graph();
    let depth_of_field_index = graph
        .nodes
        .iter()
        .position(|node| node.kind == PostProcessEffectKind::DepthOfField)
        .expect("validated graph should keep depth of field");
    let taa_index = graph
        .nodes
        .iter()
        .position(|node| node.kind == PostProcessEffectKind::TaaResolve)
        .expect("validated graph should keep TAA");
    assert!(depth_of_field_index < taa_index);
}

#[test]
fn disabling_depth_of_field_reconnects_taa_to_scene_color() {
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &Default::default(),
        &Default::default(),
        &RenderPostProcessEffectStackSettings {
            depth_of_field: RenderDepthOfFieldSettings {
                aperture: 0.75,
                max_blur_radius: 4.0,
                ..Default::default()
            },
            ..Default::default()
        },
        true,
        true,
        &AntiAliasSettings::taa(),
    )
    .with_effect_disabled(PostProcessEffectKind::DepthOfField);

    let taa_resolve = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::TaaResolve)
        .expect("TAA should remain enabled");
    assert!(taa_resolve
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCENE_COLOR.to_string()));
    assert!(!taa_resolve
        .required_inputs
        .contains(&PostProcessGraphResourceNames::DEPTH_OF_FIELDED.to_string()));
    assert!(!taa_resolve
        .after
        .contains(&PostProcessEffectKind::DepthOfField));
}

#[test]
fn disabling_depth_of_field_without_taa_reconnects_output_to_scene_color() {
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &Default::default(),
        &Default::default(),
        &RenderPostProcessEffectStackSettings {
            depth_of_field: RenderDepthOfFieldSettings {
                aperture: 0.75,
                max_blur_radius: 4.0,
                ..Default::default()
            },
            ..Default::default()
        },
        false,
        false,
        &AntiAliasSettings::off(),
    )
    .with_effect_disabled(PostProcessEffectKind::DepthOfField);

    let output_transfer = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::OutputTransfer)
        .expect("output transfer should remain enabled");
    assert!(output_transfer
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCENE_COLOR.to_string()));
    assert!(!output_transfer
        .required_inputs
        .contains(&PostProcessGraphResourceNames::DEPTH_OF_FIELDED.to_string()));
}

#[test]
fn disabling_taa_preserves_enabled_depth_of_field_as_scene_color_source() {
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &Default::default(),
        &Default::default(),
        &RenderPostProcessEffectStackSettings {
            depth_of_field: RenderDepthOfFieldSettings {
                aperture: 0.75,
                max_blur_radius: 4.0,
                ..Default::default()
            },
            ..Default::default()
        },
        true,
        true,
        &AntiAliasSettings::taa(),
    )
    .with_effect_disabled(PostProcessEffectKind::TaaResolve);

    let output_transfer = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::OutputTransfer)
        .expect("output transfer should remain enabled");
    assert!(output_transfer
        .required_inputs
        .contains(&PostProcessGraphResourceNames::DEPTH_OF_FIELDED.to_string()));
    assert!(!output_transfer
        .required_inputs
        .contains(&PostProcessGraphResourceNames::TAA_OUTPUT.to_string()));
}

#[test]
fn unavailable_temporal_history_preserves_depth_of_field_as_scene_color_source() {
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &Default::default(),
        &Default::default(),
        &RenderPostProcessEffectStackSettings {
            depth_of_field: RenderDepthOfFieldSettings {
                aperture: 0.75,
                max_blur_radius: 4.0,
                ..Default::default()
            },
            ..Default::default()
        },
        true,
        false,
        &AntiAliasSettings::taa(),
    );

    let output_transfer = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::OutputTransfer)
        .expect("output transfer should remain enabled");
    assert!(output_transfer
        .required_inputs
        .contains(&PostProcessGraphResourceNames::DEPTH_OF_FIELDED.to_string()));
    assert!(!output_transfer
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCENE_COLOR.to_string()));
}

#[test]
fn unavailable_history_disables_taa_and_restores_scene_color_input() {
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &Default::default(),
        &Default::default(),
        &RenderPostProcessEffectStackSettings::default(),
        true,
        false,
        &AntiAliasSettings::taa(),
    );
    let graph = stack.validated_graph();

    assert!(!graph
        .nodes
        .iter()
        .any(|node| node.kind == PostProcessEffectKind::TaaResolve));
    assert!(!stack
        .initial_resources
        .contains(&PostProcessGraphResourceNames::TAA_HISTORY_PREVIOUS.to_string()));
    assert!(!stack
        .initial_resources
        .contains(&PostProcessGraphResourceNames::SCENE_VELOCITY.to_string()));
    assert!(!graph.nodes.iter().any(|node| {
        node.required_inputs
            .contains(&PostProcessGraphResourceNames::TAA_REACTIVE_MASK.to_string())
    }));

    let output_transfer = graph
        .nodes
        .iter()
        .find(|node| node.kind == PostProcessEffectKind::OutputTransfer)
        .expect("history-stripped stack should keep final composite");
    assert!(output_transfer
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCENE_COLOR.to_string()));
    assert!(!output_transfer
        .required_inputs
        .contains(&PostProcessGraphResourceNames::TAA_OUTPUT.to_string()));
}

#[test]
fn unavailable_history_keeps_scene_velocity_for_motion_blur() {
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
        false,
        &AntiAliasSettings::taa(),
    );

    assert!(stack
        .initial_resources
        .contains(&PostProcessGraphResourceNames::SCENE_VELOCITY.to_string()));
    assert!(stack
        .initial_resources
        .contains(&PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX.to_string()));
}

#[test]
fn unavailable_history_keeps_scene_velocity_for_hybrid_gi_rejection() {
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &Default::default(),
        &Default::default(),
        &RenderPostProcessEffectStackSettings::default(),
        true,
        false,
        &AntiAliasSettings::taa(),
    )
    .with_hybrid_gi_lighting_input();

    assert!(stack
        .initial_resources
        .contains(&PostProcessGraphResourceNames::HYBRID_GI_LIGHTING.to_string()));
    assert!(stack
        .initial_resources
        .contains(&PostProcessGraphResourceNames::SCENE_VELOCITY.to_string()));
}

#[test]
fn hybrid_gi_lighting_input_declares_scene_velocity_for_motion_rejection() {
    let stack = PostProcessStackDescriptor::default().with_hybrid_gi_lighting_input();

    assert!(stack
        .initial_resources
        .contains(&PostProcessGraphResourceNames::HYBRID_GI_LIGHTING.to_string()));
    assert!(stack
        .initial_resources
        .contains(&PostProcessGraphResourceNames::SCENE_VELOCITY.to_string()));
}
