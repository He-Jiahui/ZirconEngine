use super::expected_uber_effect_stack_outputs;
use crate::core::framework::render::{
    AntiAliasSettings, PostProcessEffectKind, PostProcessGraphResourceNames,
    PostProcessStackDescriptor, RenderBlurSettings, RenderDepthOfFieldSettings,
    RenderMotionBlurSettings, RenderPostProcessEffectStackSettings, RenderVignetteSettings,
};
#[test]
fn enabled_effect_stack_declares_tonemapped_for_uber_descriptor() {
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &Default::default(),
        &Default::default(),
        &RenderPostProcessEffectStackSettings {
            vignette: RenderVignetteSettings {
                intensity: 0.25,
                ..Default::default()
            },
            ..Default::default()
        },
        false,
        false,
        &AntiAliasSettings::off(),
    );
    let uber = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::Uber)
        .expect("enabled effect stack should keep the uber pass");

    assert!(
        uber.produced_outputs
            .contains(&PostProcessGraphResourceNames::TONEMAPPED.to_string()),
        "uber writes TONEMAPPED in the built-in pass descriptor, so the stack must declare it"
    );
}

#[test]
fn effect_stack_depth_of_field_feeds_uber_from_dedicated_intermediate() {
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
    );

    let depth_of_field = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::DepthOfField)
        .expect("DoF should enable a dedicated depth-of-field node");
    assert!(depth_of_field
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCENE_COLOR.to_string()));
    assert!(depth_of_field
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCENE_DEPTH.to_string()));
    assert!(depth_of_field
        .produced_outputs
        .contains(&PostProcessGraphResourceNames::DEPTH_OF_FIELDED.to_string()));

    let effect_stack = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::Uber)
        .expect("DoF should enable the effect stack node");

    assert!(effect_stack
        .required_inputs
        .contains(&PostProcessGraphResourceNames::DEPTH_OF_FIELDED.to_string()));
    assert!(!effect_stack
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCENE_DEPTH.to_string()));
    assert_eq!(
        effect_stack.produced_outputs,
        expected_uber_effect_stack_outputs()
    );

    let graph = stack.validated_graph();
    let depth_of_field_index = graph
        .nodes
        .iter()
        .position(|node| node.kind == PostProcessEffectKind::DepthOfField)
        .expect("validated graph should keep the dedicated DoF node");
    let uber_index = graph
        .nodes
        .iter()
        .position(|node| node.kind == PostProcessEffectKind::Uber)
        .expect("validated graph should keep the DoF effect-stack node");
    assert!(depth_of_field_index < uber_index);
}

#[test]
fn effect_stack_blur_feeds_uber_from_dedicated_intermediate() {
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &Default::default(),
        &Default::default(),
        &RenderPostProcessEffectStackSettings {
            blur: RenderBlurSettings { radius: 3.0 },
            ..Default::default()
        },
        false,
        false,
        &AntiAliasSettings::off(),
    );

    let blur = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::Blur)
        .expect("blur should enable a dedicated blur node");
    assert!(blur
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCENE_COLOR.to_string()));
    assert!(blur
        .produced_outputs
        .contains(&PostProcessGraphResourceNames::BLURRED.to_string()));

    let effect_stack = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::Uber)
        .expect("blur should keep the effect stack node for remaining stack work");
    assert!(effect_stack
        .required_inputs
        .contains(&PostProcessGraphResourceNames::BLURRED.to_string()));
    assert!(!effect_stack
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCENE_COLOR.to_string()));
    assert_eq!(
        effect_stack.produced_outputs,
        expected_uber_effect_stack_outputs()
    );

    let graph = stack.validated_graph();
    let blur_index = graph
        .nodes
        .iter()
        .position(|node| node.kind == PostProcessEffectKind::Blur)
        .expect("validated graph should keep the dedicated blur node");
    let uber_index = graph
        .nodes
        .iter()
        .position(|node| node.kind == PostProcessEffectKind::Uber)
        .expect("validated graph should keep the blur-fed effect-stack node");
    assert!(blur_index < uber_index);
}

#[test]
fn effect_stack_motion_blur_declares_depth_and_reconstructed_motion_vector_inputs() {
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
        false,
        false,
        &AntiAliasSettings::off(),
    );

    assert!(stack
        .initial_resources
        .contains(&PostProcessGraphResourceNames::SCENE_VELOCITY.to_string()));
    assert!(stack
        .initial_resources
        .contains(&PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX.to_string()));
    assert!(stack
        .initial_resources
        .contains(&PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE.to_string()));
    assert!(stack
        .initial_resources
        .contains(&PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX.to_string()));
    let motion_blur = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::MotionBlur)
        .expect("motion blur should enable a dedicated motion blur node");
    assert!(motion_blur
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCENE_COLOR.to_string()));
    assert!(motion_blur
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCENE_DEPTH.to_string()));
    assert!(motion_blur
        .required_inputs
        .contains(&PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX.to_string()));
    assert!(motion_blur
        .produced_outputs
        .contains(&PostProcessGraphResourceNames::MOTION_BLURRED.to_string()));

    let effect_stack = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::Uber)
        .expect("motion blur should enable the effect stack node");

    assert!(effect_stack
        .required_inputs
        .contains(&PostProcessGraphResourceNames::MOTION_BLURRED.to_string()));
    assert!(!effect_stack
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCENE_DEPTH.to_string()));
    assert!(!effect_stack
        .required_inputs
        .contains(&PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX.to_string()));
    assert_eq!(
        effect_stack.produced_outputs,
        expected_uber_effect_stack_outputs()
    );

    let graph = stack.validated_graph();
    let motion_blur_index = graph
        .nodes
        .iter()
        .position(|node| node.kind == PostProcessEffectKind::MotionBlur)
        .expect("validated graph should keep the dedicated motion blur node");
    let uber_index = graph
        .nodes
        .iter()
        .position(|node| node.kind == PostProcessEffectKind::Uber)
        .expect("validated graph should keep the motion-blur-fed effect-stack node");
    assert!(motion_blur_index < uber_index);
}

#[test]
fn effect_stack_omits_depth_of_field_intermediate_outputs_when_dof_is_disabled() {
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &Default::default(),
        &Default::default(),
        &RenderPostProcessEffectStackSettings {
            vignette: RenderVignetteSettings {
                intensity: 0.25,
                ..Default::default()
            },
            ..Default::default()
        },
        false,
        false,
        &AntiAliasSettings::off(),
    );

    let effect_stack = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::Uber)
        .expect("vignette should enable the effect stack node");

    assert_eq!(
        effect_stack.produced_outputs,
        expected_uber_effect_stack_outputs()
    );
}
