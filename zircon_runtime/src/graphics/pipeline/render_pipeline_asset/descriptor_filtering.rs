use std::collections::BTreeSet;

use crate::core::framework::render::{
    PostProcessEffectKind, PostProcessGraphResourceNames, PostProcessStackDescriptor,
};
use crate::graphics::feature::{
    BuiltinRenderFeature, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
    RenderFeatureResourceAccess, RenderFeatureResourceDescriptor, RenderFeatureResourceKind,
};
use crate::graphics::pipeline::declarations::{RenderPipelineCompileOptions, RendererFeatureAsset};
use crate::graphics::scene::anti_alias::fxaa::FXAA_EXECUTOR_ID;
use crate::graphics::scene::anti_alias::smaa::SMAA_EXECUTOR_ID;
use crate::graphics::{FrameHistoryBinding, FrameHistorySlot};

pub(super) fn feature_descriptor(feature: &RendererFeatureAsset) -> RenderFeatureDescriptor {
    feature.descriptor()
}

pub(super) fn feature_descriptor_for_options(
    feature: &RendererFeatureAsset,
    options: &RenderPipelineCompileOptions,
) -> RenderFeatureDescriptor {
    let mut descriptor = feature.descriptor();
    if feature.is_builtin(BuiltinRenderFeature::Hzb) && !options.enable_hzb_occlusion_culling {
        descriptor = filter_hzb_occlusion_descriptor(descriptor);
    }
    let Some(post_process_stack) = options.post_process_stack.as_ref() else {
        if feature.is_builtin(BuiltinRenderFeature::Temporal) {
            descriptor = filter_taa_resolve_descriptor(descriptor);
        }
        if feature.is_builtin(BuiltinRenderFeature::PostProcess) {
            descriptor = filter_no_stack_post_process_resources(descriptor);
        }
        if feature.is_builtin(BuiltinRenderFeature::AntiAlias) {
            descriptor = filter_no_stack_terminal_anti_alias_descriptor(descriptor);
        }
        if feature.builtin_feature().is_none() {
            descriptor = filter_no_stack_plugin_post_process_resources(descriptor);
        }
        return descriptor;
    };
    let Some(builtin_feature) = feature.builtin_feature() else {
        return filter_plugin_post_process_descriptor(descriptor, post_process_stack);
    };
    if !post_process_stack_filters_feature(builtin_feature) {
        return descriptor;
    }

    filter_post_process_descriptor(descriptor, builtin_feature, post_process_stack)
}

fn filter_hzb_occlusion_descriptor(
    mut descriptor: RenderFeatureDescriptor,
) -> RenderFeatureDescriptor {
    descriptor
        .stage_passes
        .retain(|pass| pass.executor_id.as_str() != "visibility.hzb-occlusion-cull");
    descriptor
}

fn filter_taa_resolve_descriptor(
    mut descriptor: RenderFeatureDescriptor,
) -> RenderFeatureDescriptor {
    descriptor.stage_passes.retain(|pass| {
        !matches!(
            pass.executor_id.as_str(),
            "temporal.taa-reactive-mask-clear"
                | "temporal.taa-reactive-mask-mesh"
                | "temporal.taa-resolve"
        )
    });
    descriptor
}

fn filter_no_stack_post_process_resources(
    mut descriptor: RenderFeatureDescriptor,
) -> RenderFeatureDescriptor {
    descriptor.stage_passes.retain(|pass| {
        !matches!(
            pass.executor_id.as_str(),
            "post.color-lut-bake" | "post.upscale"
        )
    });
    for pass in &mut descriptor.stage_passes {
        pass.resources.retain(|resource| {
            resource.name != PostProcessGraphResourceNames::TAA_OUTPUT
                && resource.name != PostProcessGraphResourceNames::TAA_REACTIVE_MASK
                && resource.name != PostProcessGraphResourceNames::COLOR_LUT
                && resource.name != PostProcessGraphResourceNames::UPSCALED
        });
    }
    descriptor
}

fn filter_no_stack_plugin_post_process_resources(
    mut descriptor: RenderFeatureDescriptor,
) -> RenderFeatureDescriptor {
    descriptor
        .stage_passes
        .retain(|pass| pass.executor_id.as_str() != SMAA_EXECUTOR_ID);
    descriptor
}

fn filter_no_stack_terminal_anti_alias_descriptor(
    mut descriptor: RenderFeatureDescriptor,
) -> RenderFeatureDescriptor {
    descriptor
        .stage_passes
        .retain(|pass| pass.executor_id.as_str() != SMAA_EXECUTOR_ID);
    descriptor
}

fn post_process_stack_filters_feature(feature: BuiltinRenderFeature) -> bool {
    matches!(
        feature,
        BuiltinRenderFeature::Bloom
            | BuiltinRenderFeature::ColorGrading
            | BuiltinRenderFeature::Temporal
            | BuiltinRenderFeature::AntiAlias
            | BuiltinRenderFeature::PostProcess
    )
}

fn filter_plugin_post_process_descriptor(
    mut descriptor: RenderFeatureDescriptor,
    stack: &PostProcessStackDescriptor,
) -> RenderFeatureDescriptor {
    descriptor.stage_passes = descriptor
        .stage_passes
        .into_iter()
        .filter_map(|pass| filter_plugin_post_process_pass(pass, stack))
        .collect();
    descriptor
}

fn filter_plugin_post_process_pass(
    pass: RenderFeaturePassDescriptor,
    stack: &PostProcessStackDescriptor,
) -> Option<RenderFeaturePassDescriptor> {
    if !plugin_post_process_pass_enabled(&pass, stack) {
        return None;
    }
    if post_process_pass_can_be_filtered(
        BuiltinRenderFeature::PostProcess,
        pass.executor_id.as_str(),
    ) {
        return filter_post_process_pass(pass, BuiltinRenderFeature::PostProcess, stack);
    }
    Some(pass)
}

fn plugin_post_process_pass_enabled(
    pass: &RenderFeaturePassDescriptor,
    stack: &PostProcessStackDescriptor,
) -> bool {
    pass.executor_id.as_str() != SMAA_EXECUTOR_ID
        || stack_effect_enabled(stack, PostProcessEffectKind::Smaa)
}

fn filter_post_process_descriptor(
    mut descriptor: RenderFeatureDescriptor,
    feature: BuiltinRenderFeature,
    stack: &PostProcessStackDescriptor,
) -> RenderFeatureDescriptor {
    descriptor.stage_passes = descriptor
        .stage_passes
        .into_iter()
        .filter_map(|pass| filter_post_process_pass(pass, feature, stack))
        .collect();
    sync_optional_history_bindings(&mut descriptor, feature, stack);
    descriptor
}

fn sync_optional_history_bindings(
    descriptor: &mut RenderFeatureDescriptor,
    feature: BuiltinRenderFeature,
    stack: &PostProcessStackDescriptor,
) {
    if feature != BuiltinRenderFeature::Temporal {
        return;
    }

    descriptor
        .history_bindings
        .retain(|binding| binding.slot != FrameHistorySlot::TaaSceneColor);
    if stack_effect_enabled(stack, PostProcessEffectKind::TaaResolve) {
        descriptor
            .history_bindings
            .push(FrameHistoryBinding::read_write(
                FrameHistorySlot::TaaSceneColor,
            ));
    }
}

fn filter_post_process_pass(
    mut pass: RenderFeaturePassDescriptor,
    feature: BuiltinRenderFeature,
    stack: &PostProcessStackDescriptor,
) -> Option<RenderFeaturePassDescriptor> {
    if !post_process_pass_can_be_filtered(feature, pass.executor_id.as_str()) {
        return Some(pass);
    }
    if !optional_post_process_pass_enabled(feature, pass.executor_id.as_str(), stack) {
        return None;
    }
    let active_resources = active_post_process_graph_resources(stack);
    pass.resources = pass
        .resources
        .into_iter()
        .filter(|resource| post_process_resource_is_active(resource, &active_resources))
        .collect();
    route_bloom_to_latest_scene_color_input(&mut pass, feature, stack);
    route_scene_composite_to_latest_scene_color_input(&mut pass, feature, stack);
    route_blur_to_latest_color_input(&mut pass, feature, stack);
    route_uber_to_latest_color_input(&mut pass, feature, stack);
    route_output_transfer_to_upscaled_input(&mut pass, feature, stack);
    route_output_transfer_to_terminal_anti_alias_input(&mut pass, feature, stack);
    (!pass.resources.is_empty()).then_some(pass)
}

fn post_process_pass_can_be_filtered(feature: BuiltinRenderFeature, executor_id: &str) -> bool {
    match (feature, executor_id) {
        (BuiltinRenderFeature::Temporal, "temporal.velocity-object")
        | (BuiltinRenderFeature::Temporal, "temporal.velocity-camera")
        | (BuiltinRenderFeature::Temporal, "temporal.taa-reactive-mask-clear")
        | (BuiltinRenderFeature::Temporal, "temporal.taa-reactive-mask-mesh")
        | (BuiltinRenderFeature::Temporal, "temporal.taa-resolve")
        | (BuiltinRenderFeature::PostProcess, "post.motion-vector-tile-max")
        | (BuiltinRenderFeature::PostProcess, "post.motion-vector-tile-max-coarse")
        | (BuiltinRenderFeature::PostProcess, "post.motion-vector-neighbor-max")
        | (BuiltinRenderFeature::PostProcess, "post.depth-of-field-prepare")
        | (BuiltinRenderFeature::PostProcess, "post.depth-of-field")
        | (BuiltinRenderFeature::PostProcess, "post.motion-blur")
        | (BuiltinRenderFeature::PostProcess, "post.exposure.histogram")
        | (BuiltinRenderFeature::PostProcess, "post.exposure.resolve")
        | (BuiltinRenderFeature::PostProcess, "post.screen-space-reflection-reflection-pyramid")
        | (
            BuiltinRenderFeature::PostProcess,
            "post.screen-space-reflection-reflection-pyramid-coarse",
        )
        | (BuiltinRenderFeature::PostProcess, "post.screen-space-reflection-specular-occlusion")
        | (BuiltinRenderFeature::PostProcess, "post.screen-space-reflection-resolve")
        | (BuiltinRenderFeature::PostProcess, "post.scene-composite")
        | (BuiltinRenderFeature::PostProcess, "post.blur")
        | (BuiltinRenderFeature::PostProcess, "post.color-lut-bake")
        | (BuiltinRenderFeature::PostProcess, "post.uber")
        | (BuiltinRenderFeature::PostProcess, "post.upscale")
        | (BuiltinRenderFeature::PostProcess, "post.output-transfer")
        | (BuiltinRenderFeature::Bloom, "post.bloom-extract" | "post.bloom")
        | (BuiltinRenderFeature::AntiAlias, _) => true,
        _ => false,
    }
}

fn optional_post_process_pass_enabled(
    feature: BuiltinRenderFeature,
    executor_id: &str,
    stack: &PostProcessStackDescriptor,
) -> bool {
    match (feature, executor_id) {
        (BuiltinRenderFeature::Bloom, "post.bloom-extract" | "post.bloom") => {
            stack_effect_enabled(stack, PostProcessEffectKind::Bloom)
        }
        (BuiltinRenderFeature::PostProcess, "post.color-lut-bake") => {
            stack_effect_enabled(stack, PostProcessEffectKind::ColorLutBake)
        }
        (BuiltinRenderFeature::PostProcess, "post.upscale") => {
            stack_effect_enabled(stack, PostProcessEffectKind::Upscale)
        }
        (BuiltinRenderFeature::AntiAlias, id) if id == FXAA_EXECUTOR_ID => {
            stack_effect_enabled(stack, PostProcessEffectKind::Fxaa)
        }
        (BuiltinRenderFeature::AntiAlias, id) if id == SMAA_EXECUTOR_ID => {
            stack_effect_enabled(stack, PostProcessEffectKind::Smaa)
        }
        (
            BuiltinRenderFeature::Temporal,
            "temporal.taa-reactive-mask-clear"
            | "temporal.taa-reactive-mask-mesh"
            | "temporal.taa-resolve",
        ) => stack_effect_enabled(stack, PostProcessEffectKind::TaaResolve),
        (BuiltinRenderFeature::Temporal, "temporal.velocity-object")
        | (BuiltinRenderFeature::Temporal, "temporal.velocity-camera") => {
            post_process_stack_uses_scene_velocity(stack)
        }
        (BuiltinRenderFeature::PostProcess, "post.motion-vector-tile-max")
        | (BuiltinRenderFeature::PostProcess, "post.motion-vector-tile-max-coarse")
        | (BuiltinRenderFeature::PostProcess, "post.motion-vector-neighbor-max") => {
            post_process_stack_uses_reconstructed_motion_vectors(stack)
        }
        (BuiltinRenderFeature::PostProcess, "post.depth-of-field-prepare") => {
            post_process_stack_uses_depth_of_field(stack)
        }
        (BuiltinRenderFeature::PostProcess, "post.depth-of-field") => {
            stack_effect_enabled(stack, PostProcessEffectKind::DepthOfField)
        }
        (BuiltinRenderFeature::PostProcess, "post.motion-blur") => {
            stack_effect_enabled(stack, PostProcessEffectKind::MotionBlur)
        }
        (BuiltinRenderFeature::PostProcess, "post.exposure.histogram") => {
            stack_effect_enabled(stack, PostProcessEffectKind::ExposureHistogram)
        }
        (BuiltinRenderFeature::PostProcess, "post.exposure.resolve") => {
            stack_effect_enabled(stack, PostProcessEffectKind::ExposureResolve)
        }
        (BuiltinRenderFeature::PostProcess, "post.screen-space-reflection-reflection-pyramid") => {
            stack_effect_enabled(
                stack,
                PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramid,
            )
        }
        (
            BuiltinRenderFeature::PostProcess,
            "post.screen-space-reflection-reflection-pyramid-coarse",
        ) => stack_effect_enabled(
            stack,
            PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramidCoarse,
        ),
        (BuiltinRenderFeature::PostProcess, "post.screen-space-reflection-specular-occlusion") => {
            stack_effect_enabled(
                stack,
                PostProcessEffectKind::ScreenSpaceReflectionSpecularOcclusion,
            )
        }
        (BuiltinRenderFeature::PostProcess, "post.screen-space-reflection-resolve") => {
            stack_effect_enabled(stack, PostProcessEffectKind::ScreenSpaceReflectionResolve)
        }
        (BuiltinRenderFeature::PostProcess, "post.scene-composite") => {
            stack_effect_enabled(stack, PostProcessEffectKind::SceneComposite)
        }
        (BuiltinRenderFeature::PostProcess, "post.blur") => {
            stack_effect_enabled(stack, PostProcessEffectKind::Blur)
        }
        (BuiltinRenderFeature::PostProcess, "post.uber")
        | (BuiltinRenderFeature::PostProcess, "post.output-transfer") => true,
        _ => true,
    }
}

fn post_process_stack_uses_scene_velocity(stack: &PostProcessStackDescriptor) -> bool {
    stack
        .initial_resources
        .iter()
        .any(|resource| resource == PostProcessGraphResourceNames::SCENE_VELOCITY)
}

fn post_process_stack_uses_reconstructed_motion_vectors(
    stack: &PostProcessStackDescriptor,
) -> bool {
    stack.initial_resources.iter().any(|resource| {
        resource == PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX
            || resource == PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE
            || resource == PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX
    })
}

fn post_process_stack_uses_depth_of_field(stack: &PostProcessStackDescriptor) -> bool {
    stack_effect_enabled(stack, PostProcessEffectKind::DepthOfField)
}

fn active_post_process_graph_resources(stack: &PostProcessStackDescriptor) -> BTreeSet<String> {
    let mut resources = stack
        .initial_resources
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    for effect in stack.effects.iter().filter(|effect| effect.enabled) {
        resources.extend(effect.required_inputs.iter().cloned());
        resources.extend(effect.produced_outputs.iter().cloned());
    }
    resources
}

fn post_process_resource_is_active(
    resource: &RenderFeatureResourceDescriptor,
    active_resources: &BTreeSet<String>,
) -> bool {
    matches!(
        resource.name.as_str(),
        PostProcessGraphResourceNames::FINAL_COLOR
            | PostProcessGraphResourceNames::FINAL_COMPOSITED
            | PostProcessGraphResourceNames::AMBIENT_OCCLUSION
            | PostProcessGraphResourceNames::BLURRED
            | PostProcessGraphResourceNames::SCENE_COMPOSITED
            | PostProcessGraphResourceNames::DEPTH_OF_FIELDED
            | PostProcessGraphResourceNames::MOTION_BLURRED
            | PostProcessGraphResourceNames::TONEMAPPED
            | PostProcessGraphResourceNames::GLOBAL_ILLUMINATION
            | PostProcessGraphResourceNames::CONTACT_SHADOW_OCCLUSION
    ) || active_resources.contains(&resource.name)
}

fn route_bloom_to_latest_scene_color_input(
    pass: &mut RenderFeaturePassDescriptor,
    feature: BuiltinRenderFeature,
    stack: &PostProcessStackDescriptor,
) {
    if feature != BuiltinRenderFeature::Bloom || pass.executor_id.as_str() != "post.bloom-extract" {
        return;
    }

    let source = latest_post_process_scene_color(stack);
    for resource in &mut pass.resources {
        if resource.access == RenderFeatureResourceAccess::Read
            && resource.name == PostProcessGraphResourceNames::SCENE_COLOR
        {
            resource.name = source.to_string();
        }
    }
}

fn latest_post_process_scene_color(stack: &PostProcessStackDescriptor) -> &'static str {
    if stack_effect_enabled(stack, PostProcessEffectKind::MotionBlur) {
        PostProcessGraphResourceNames::MOTION_BLURRED
    } else if stack_effect_enabled(stack, PostProcessEffectKind::DepthOfField) {
        PostProcessGraphResourceNames::DEPTH_OF_FIELDED
    } else if stack_effect_enabled(stack, PostProcessEffectKind::TaaResolve) {
        PostProcessGraphResourceNames::TAA_OUTPUT
    } else {
        PostProcessGraphResourceNames::SCENE_COLOR
    }
}

fn route_scene_composite_to_latest_scene_color_input(
    pass: &mut RenderFeaturePassDescriptor,
    feature: BuiltinRenderFeature,
    stack: &PostProcessStackDescriptor,
) {
    if feature != BuiltinRenderFeature::PostProcess
        || pass.executor_id.as_str() != "post.scene-composite"
    {
        return;
    }

    let source = latest_post_process_scene_color(stack);
    for resource in &mut pass.resources {
        if resource.access == RenderFeatureResourceAccess::Read
            && resource.name == PostProcessGraphResourceNames::SCENE_COLOR
        {
            resource.name = source.to_string();
        }
    }
    retain_single_scene_color_input(pass, source);
}

fn route_blur_to_latest_color_input(
    pass: &mut RenderFeaturePassDescriptor,
    feature: BuiltinRenderFeature,
    stack: &PostProcessStackDescriptor,
) {
    if feature != BuiltinRenderFeature::PostProcess || pass.executor_id.as_str() != "post.blur" {
        return;
    }

    let source = if stack_effect_enabled(stack, PostProcessEffectKind::SceneComposite) {
        PostProcessGraphResourceNames::SCENE_COMPOSITED
    } else {
        latest_post_process_scene_color(stack)
    };
    for resource in &mut pass.resources {
        if resource.access == RenderFeatureResourceAccess::Read
            && resource.name == PostProcessGraphResourceNames::SCENE_COLOR
        {
            resource.name = source.to_string();
        }
    }
    retain_single_scene_color_input(pass, source);
}

fn route_uber_to_latest_color_input(
    pass: &mut RenderFeaturePassDescriptor,
    feature: BuiltinRenderFeature,
    stack: &PostProcessStackDescriptor,
) {
    if feature != BuiltinRenderFeature::PostProcess || pass.executor_id.as_str() != "post.uber" {
        return;
    }

    let scene_composite_enabled =
        stack_effect_enabled(stack, PostProcessEffectKind::SceneComposite);
    let blur_enabled = stack_effect_enabled(stack, PostProcessEffectKind::Blur);
    let source = if blur_enabled {
        PostProcessGraphResourceNames::BLURRED
    } else if scene_composite_enabled {
        PostProcessGraphResourceNames::SCENE_COMPOSITED
    } else {
        latest_post_process_scene_color(stack)
    };
    for resource in &mut pass.resources {
        if resource.access == RenderFeatureResourceAccess::Read
            && resource.name == PostProcessGraphResourceNames::SCENE_COLOR
        {
            resource.name = source.to_string();
        }
    }
    retain_single_scene_color_input(pass, source);
    if scene_composite_enabled {
        pass.resources.retain(|resource| {
            resource.access != RenderFeatureResourceAccess::Read
                || resource.name != PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY
        });
    }
}

fn retain_single_scene_color_input(pass: &mut RenderFeaturePassDescriptor, source: &'static str) {
    let mut kept_source = false;
    pass.resources.retain(|resource| {
        if resource.access != RenderFeatureResourceAccess::Read
            || !post_process_scene_color_chain_resource(resource.name.as_str())
        {
            return true;
        }
        if resource.name.as_str() == source && !kept_source {
            kept_source = true;
            true
        } else {
            false
        }
    });
}

fn post_process_scene_color_chain_resource(name: &str) -> bool {
    matches!(
        name,
        PostProcessGraphResourceNames::SCENE_COLOR
            | PostProcessGraphResourceNames::TAA_OUTPUT
            | PostProcessGraphResourceNames::DEPTH_OF_FIELDED
            | PostProcessGraphResourceNames::MOTION_BLURRED
            | PostProcessGraphResourceNames::SCENE_COMPOSITED
            | PostProcessGraphResourceNames::BLURRED
    )
}

fn route_output_transfer_to_upscaled_input(
    pass: &mut RenderFeaturePassDescriptor,
    feature: BuiltinRenderFeature,
    stack: &PostProcessStackDescriptor,
) {
    if feature != BuiltinRenderFeature::PostProcess
        || pass.executor_id.as_str() != "post.output-transfer"
        || !stack_effect_enabled(stack, PostProcessEffectKind::Upscale)
    {
        return;
    }

    for resource in &mut pass.resources {
        if resource.access == RenderFeatureResourceAccess::Read
            && resource.name == PostProcessGraphResourceNames::TONEMAPPED
        {
            resource.name = PostProcessGraphResourceNames::UPSCALED.to_string();
        }
    }
}

fn route_output_transfer_to_terminal_anti_alias_input(
    pass: &mut RenderFeaturePassDescriptor,
    feature: BuiltinRenderFeature,
    stack: &PostProcessStackDescriptor,
) {
    if feature != BuiltinRenderFeature::PostProcess
        || pass.executor_id.as_str() != "post.output-transfer"
        || !post_process_stack_uses_terminal_anti_alias(stack)
    {
        return;
    }

    for resource in &mut pass.resources {
        if resource.access == RenderFeatureResourceAccess::Write
            && resource.name == PostProcessGraphResourceNames::FINAL_COLOR
        {
            resource.name = PostProcessGraphResourceNames::FINAL_COMPOSITED.to_string();
            resource.kind = RenderFeatureResourceKind::Texture;
        }
    }
}

fn post_process_stack_uses_terminal_anti_alias(stack: &PostProcessStackDescriptor) -> bool {
    stack_effect_enabled(stack, PostProcessEffectKind::Fxaa)
        || stack_effect_enabled(stack, PostProcessEffectKind::Smaa)
}

fn stack_effect_enabled(stack: &PostProcessStackDescriptor, kind: PostProcessEffectKind) -> bool {
    stack
        .effects
        .iter()
        .any(|effect| effect.enabled && effect.kind == kind)
}
