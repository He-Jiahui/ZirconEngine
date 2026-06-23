use crate::core::framework::render::{
    AntiAliasSettings, PostProcessEffectKind, PostProcessGraphResourceNames,
    PostProcessStackDescriptor, RenderPostProcessEffectStackSettings,
    RenderScreenSpaceReflectionSettings,
};
#[test]
fn screen_space_reflection_declares_specular_occlusion_and_resolve_inputs() {
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &Default::default(),
        &Default::default(),
        &RenderPostProcessEffectStackSettings {
            screen_space_reflection: RenderScreenSpaceReflectionSettings {
                intensity: 0.5,
                max_steps: 32,
                ..Default::default()
            },
            ..Default::default()
        },
        false,
        false,
        &AntiAliasSettings::off(),
    );

    let specular_occlusion = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::ScreenSpaceReflectionSpecularOcclusion)
        .expect("SSR should enable the screen-space reflection specular occlusion node");

    assert!(specular_occlusion
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCENE_DEPTH.to_string()));
    assert!(specular_occlusion
        .required_inputs
        .contains(&PostProcessGraphResourceNames::GBUFFER_MATERIAL.to_string()));
    assert!(specular_occlusion
        .required_inputs
        .contains(&PostProcessGraphResourceNames::AMBIENT_OCCLUSION.to_string()));
    assert!(specular_occlusion.produced_outputs.contains(
        &PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION.to_string()
    ));

    assert!(stack
        .initial_resources
        .contains(&PostProcessGraphResourceNames::HZB_FURTHEST.to_string()));

    let reflection_pyramid = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramid)
        .expect("SSR should enable the screen-space reflection reflection pyramid node");

    assert!(reflection_pyramid
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCENE_COLOR.to_string()));
    assert!(reflection_pyramid.produced_outputs.contains(
        &PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID.to_string()
    ));

    let reflection_pyramid_coarse = stack
        .effects
        .iter()
        .find(|effect| {
            effect.kind == PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramidCoarse
        })
        .expect("SSR should enable the coarse screen-space reflection reflection pyramid node");

    assert!(reflection_pyramid_coarse.required_inputs.contains(
        &PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID.to_string()
    ));
    assert!(reflection_pyramid_coarse.produced_outputs.contains(
        &PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE
            .to_string()
    ));
    assert!(reflection_pyramid_coarse
        .after
        .contains(&PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramid));

    let resolve = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::ScreenSpaceReflectionResolve)
        .expect("SSR should enable the screen-space reflection resolve node");

    assert!(resolve
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCENE_COLOR.to_string()));
    assert!(resolve
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCENE_DEPTH.to_string()));
    assert!(resolve
        .required_inputs
        .contains(&PostProcessGraphResourceNames::HZB_FURTHEST.to_string()));
    assert!(resolve.required_inputs.contains(
        &PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID.to_string()
    ));
    assert!(resolve.required_inputs.contains(
        &PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE
            .to_string()
    ));
    assert!(resolve
        .required_inputs
        .contains(&PostProcessGraphResourceNames::GBUFFER_NORMAL.to_string()));
    assert!(resolve
        .required_inputs
        .contains(&PostProcessGraphResourceNames::GBUFFER_MATERIAL.to_string()));
    assert!(resolve.required_inputs.contains(
        &PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION.to_string()
    ));
    assert!(!resolve
        .required_inputs
        .contains(&PostProcessGraphResourceNames::AMBIENT_OCCLUSION.to_string()));
    assert!(resolve
        .required_inputs
        .contains(&PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX.to_string()));
    assert!(resolve
        .after
        .contains(&PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramid));
    assert!(resolve
        .after
        .contains(&PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramidCoarse));
    assert!(resolve
        .after
        .contains(&PostProcessEffectKind::ScreenSpaceReflectionSpecularOcclusion));

    let scene_composite = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::SceneComposite)
        .expect("SSR should feed the scene composite node");
    assert!(scene_composite
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY.to_string()));
    assert!(scene_composite
        .produced_outputs
        .contains(&PostProcessGraphResourceNames::SCENE_COMPOSITED.to_string()));
    assert!(scene_composite
        .after
        .contains(&PostProcessEffectKind::ScreenSpaceReflectionResolve));

    let effect_stack = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::Uber)
        .expect("SSR should keep an effect-stack color node");
    assert!(effect_stack
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCENE_COMPOSITED.to_string()));
    assert!(!effect_stack
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY.to_string()));
    assert!(!effect_stack
        .required_inputs
        .contains(&PostProcessGraphResourceNames::GBUFFER_NORMAL.to_string()));
    assert!(!effect_stack
        .produced_outputs
        .contains(&PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY.to_string()));
}

#[test]
fn screen_space_reflection_resolve_temporal_declares_history_and_motion_vector_inputs() {
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &Default::default(),
        &Default::default(),
        &RenderPostProcessEffectStackSettings {
            screen_space_reflection: RenderScreenSpaceReflectionSettings {
                intensity: 0.5,
                max_steps: 32,
                ..Default::default()
            },
            ..Default::default()
        },
        true,
        true,
        &AntiAliasSettings::off(),
    );

    assert!(stack.initial_resources.contains(
        &PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION.to_string()
    ));
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

    let resolve = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::ScreenSpaceReflectionResolve)
        .expect("temporal SSR should enable the screen-space reflection resolve node");

    assert!(resolve.required_inputs.contains(
        &PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCREEN_SPACE_REFLECTION.to_string()
    ));
    assert!(resolve
        .required_inputs
        .contains(&PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX.to_string()));
    assert!(resolve.required_inputs.contains(
        &PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION.to_string()
    ));
    assert!(resolve
        .required_inputs
        .contains(&PostProcessGraphResourceNames::HZB_FURTHEST.to_string()));
    assert!(resolve.required_inputs.contains(
        &PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID.to_string()
    ));
    assert!(resolve.required_inputs.contains(
        &PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE
            .to_string()
    ));
}

#[test]
fn screen_space_reflection_resolve_feeds_scene_composite_before_output_transfer() {
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &Default::default(),
        &Default::default(),
        &RenderPostProcessEffectStackSettings {
            screen_space_reflection: RenderScreenSpaceReflectionSettings {
                intensity: 0.5,
                max_steps: 32,
                ..Default::default()
            },
            ..Default::default()
        },
        false,
        false,
        &AntiAliasSettings::off(),
    );

    let resolve = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::ScreenSpaceReflectionResolve)
        .expect("SSR should enable the screen-space reflection resolve node");

    assert!(resolve
        .produced_outputs
        .contains(&PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY.to_string()));
    assert!(resolve
        .after
        .contains(&PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramid));
    assert!(resolve
        .after
        .contains(&PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramidCoarse));
    assert!(resolve
        .after
        .contains(&PostProcessEffectKind::ScreenSpaceReflectionSpecularOcclusion));

    let scene_composite = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::SceneComposite)
        .expect("SSR should feed the scene composite node");
    assert!(scene_composite
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY.to_string()));
    assert!(scene_composite
        .produced_outputs
        .contains(&PostProcessGraphResourceNames::SCENE_COMPOSITED.to_string()));
    assert!(scene_composite
        .after
        .contains(&PostProcessEffectKind::ScreenSpaceReflectionResolve));

    let output_transfer = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::OutputTransfer)
        .expect("SSR should keep final composite node");
    assert!(!output_transfer
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY.to_string()));
    assert!(output_transfer.after.contains(&PostProcessEffectKind::Uber));

    let graph = stack.validated_graph();
    let graph_resolve = graph
        .nodes
        .iter()
        .find(|node| node.kind == PostProcessEffectKind::ScreenSpaceReflectionResolve)
        .expect("validated graph should keep the SSR resolve node");
    assert!(graph_resolve
        .produced_outputs
        .contains(&PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY.to_string()));
    assert!(graph_resolve
        .after
        .contains(&PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramid));
    assert!(graph_resolve
        .after
        .contains(&PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramidCoarse));
    assert!(graph_resolve
        .after
        .contains(&PostProcessEffectKind::ScreenSpaceReflectionSpecularOcclusion));

    let graph_composite = graph
        .nodes
        .iter()
        .find(|node| node.kind == PostProcessEffectKind::SceneComposite)
        .expect("validated graph should keep the scene composite node");
    assert!(graph_composite
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY.to_string()));
    assert!(graph_composite
        .produced_outputs
        .contains(&PostProcessGraphResourceNames::SCENE_COMPOSITED.to_string()));
    assert!(graph_composite
        .after
        .contains(&PostProcessEffectKind::ScreenSpaceReflectionResolve));

    let graph_final = graph
        .nodes
        .iter()
        .find(|node| node.kind == PostProcessEffectKind::OutputTransfer)
        .expect("validated graph should keep the final composite node");
    assert!(!graph_final
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY.to_string()));
    assert!(graph_final.after.contains(&PostProcessEffectKind::Uber));
}
