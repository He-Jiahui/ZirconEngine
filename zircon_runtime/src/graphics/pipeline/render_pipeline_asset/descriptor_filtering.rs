use std::{cell::OnceCell, collections::BTreeSet};

use crate::core::framework::render::{
    PostProcessEffectKind, PostProcessGraphResourceNames, PostProcessStackDescriptor,
};
use crate::graphics::feature::{
    BuiltinRenderFeature, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
    RenderFeatureResourceAccess, RenderFeatureResourceDescriptor, RenderFeatureResourceVersion,
};
use crate::graphics::pipeline::declarations::{RenderPipelineCompileOptions, RendererFeatureAsset};
use crate::graphics::scene::anti_alias::fxaa::{FXAA_EXECUTOR_ID, FXAA_PASS_NAME};
use crate::graphics::scene::anti_alias::smaa::{SMAA_EXECUTOR_ID, SMAA_PASS_NAME};
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
            "temporal.taa-reactive-mask-mesh" | "temporal.taa-resolve"
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
                && resource.name != PostProcessGraphResourceNames::HYBRID_GI_LIGHTING
        });
    }
    descriptor
}

fn filter_no_stack_plugin_post_process_resources(
    mut descriptor: RenderFeatureDescriptor,
) -> RenderFeatureDescriptor {
    let preserves_default_motion_vector_chain = descriptor.name == "post_process";
    descriptor.stage_passes.retain(|pass| {
        pass.executor_id.as_str() != SMAA_EXECUTOR_ID
            && (preserves_default_motion_vector_chain
                || !plugin_pass_references_scene_velocity(pass))
    });
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
    let active_resources = OnceCell::new();
    descriptor.stage_passes = descriptor
        .stage_passes
        .into_iter()
        .filter_map(|pass| filter_plugin_post_process_pass(pass, stack, &active_resources))
        .collect();
    descriptor
}

fn filter_plugin_post_process_pass(
    pass: RenderFeaturePassDescriptor,
    stack: &PostProcessStackDescriptor,
    active_resources: &OnceCell<BTreeSet<String>>,
) -> Option<RenderFeaturePassDescriptor> {
    if !plugin_post_process_pass_enabled(&pass, stack) {
        return None;
    }
    if plugin_pass_requires_inactive_scene_velocity(&pass, stack) {
        return None;
    }
    if post_process_pass_can_be_filtered(
        BuiltinRenderFeature::PostProcess,
        pass.executor_id.as_str(),
    ) {
        return filter_post_process_pass(
            pass,
            BuiltinRenderFeature::PostProcess,
            stack,
            active_resources.get_or_init(|| active_post_process_graph_resources(stack)),
        );
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

fn plugin_pass_requires_inactive_scene_velocity(
    pass: &RenderFeaturePassDescriptor,
    stack: &PostProcessStackDescriptor,
) -> bool {
    !post_process_stack_uses_scene_velocity(stack) && plugin_pass_references_scene_velocity(pass)
}

fn plugin_pass_references_scene_velocity(pass: &RenderFeaturePassDescriptor) -> bool {
    pass.resources
        .iter()
        .any(|resource| resource.name == PostProcessGraphResourceNames::SCENE_VELOCITY)
}

fn filter_post_process_descriptor(
    mut descriptor: RenderFeatureDescriptor,
    feature: BuiltinRenderFeature,
    stack: &PostProcessStackDescriptor,
) -> RenderFeatureDescriptor {
    let active_resources = active_post_process_graph_resources(stack);
    descriptor.stage_passes = descriptor
        .stage_passes
        .into_iter()
        .filter_map(|pass| filter_post_process_pass(pass, feature, stack, &active_resources))
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
    active_resources: &BTreeSet<String>,
) -> Option<RenderFeaturePassDescriptor> {
    if !post_process_pass_can_be_filtered(feature, pass.executor_id.as_str()) {
        return Some(pass);
    }
    if !optional_post_process_pass_enabled(feature, pass.executor_id.as_str(), stack) {
        return None;
    }
    pass.resources = pass
        .resources
        .into_iter()
        .filter(|resource| post_process_resource_is_active(resource, active_resources))
        .collect();
    route_bloom_to_latest_scene_color_input(&mut pass, feature, stack);
    route_scene_composite_to_latest_scene_color_input(&mut pass, feature, stack);
    route_blur_to_latest_color_input(&mut pass, feature, stack);
    route_uber_to_latest_color_input(&mut pass, feature, stack);
    route_output_transfer_to_upscaled_input(&mut pass, feature, stack);
    route_output_transfer_to_terminal_anti_alias_input(&mut pass, feature, stack);
    route_upscale_to_terminal_anti_alias_input(&mut pass, feature, stack);
    (!pass.resources.is_empty()).then_some(pass)
}

fn post_process_pass_can_be_filtered(feature: BuiltinRenderFeature, executor_id: &str) -> bool {
    match (feature, executor_id) {
        (BuiltinRenderFeature::Temporal, "temporal.velocity-object")
        | (BuiltinRenderFeature::Temporal, "temporal.velocity-camera")
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
            "temporal.taa-reactive-mask-mesh" | "temporal.taa-resolve",
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
            route_resource_to_produced_input(resource, source);
        }
    }
}

#[derive(Clone, Copy)]
struct PostProcessSceneColorInput {
    name: &'static str,
    producer_pass_name: Option<&'static str>,
}

fn latest_post_process_scene_color(
    stack: &PostProcessStackDescriptor,
) -> PostProcessSceneColorInput {
    if stack_effect_enabled(stack, PostProcessEffectKind::MotionBlur) {
        PostProcessSceneColorInput {
            name: PostProcessGraphResourceNames::MOTION_BLURRED,
            producer_pass_name: Some("motion-blur"),
        }
    } else if stack_effect_enabled(stack, PostProcessEffectKind::DepthOfField) {
        PostProcessSceneColorInput {
            name: PostProcessGraphResourceNames::DEPTH_OF_FIELDED,
            producer_pass_name: Some("depth-of-field"),
        }
    } else if stack_effect_enabled(stack, PostProcessEffectKind::TaaResolve) {
        PostProcessSceneColorInput {
            name: PostProcessGraphResourceNames::TAA_OUTPUT,
            producer_pass_name: Some("taa-resolve"),
        }
    } else {
        PostProcessSceneColorInput {
            name: PostProcessGraphResourceNames::SCENE_COLOR,
            producer_pass_name: None,
        }
    }
}

fn route_resource_to_produced_input(
    resource: &mut RenderFeatureResourceDescriptor,
    input: PostProcessSceneColorInput,
) {
    resource.name = input.name.to_string();
    resource.input_version = input.producer_pass_name.map(|producer_pass_name| {
        RenderFeatureResourceVersion::new(input.name, resource.kind, producer_pass_name)
    });
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
            route_resource_to_produced_input(resource, source);
        }
    }
    retain_single_scene_color_input(pass, source.name);
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
        PostProcessSceneColorInput {
            name: PostProcessGraphResourceNames::SCENE_COMPOSITED,
            producer_pass_name: Some("scene-composite"),
        }
    } else {
        latest_post_process_scene_color(stack)
    };
    for resource in &mut pass.resources {
        if resource.access == RenderFeatureResourceAccess::Read
            && resource.name == PostProcessGraphResourceNames::SCENE_COLOR
        {
            route_resource_to_produced_input(resource, source);
        }
    }
    retain_single_scene_color_input(pass, source.name);
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
        PostProcessSceneColorInput {
            name: PostProcessGraphResourceNames::BLURRED,
            producer_pass_name: Some("blur"),
        }
    } else if scene_composite_enabled {
        PostProcessSceneColorInput {
            name: PostProcessGraphResourceNames::SCENE_COMPOSITED,
            producer_pass_name: Some("scene-composite"),
        }
    } else {
        latest_post_process_scene_color(stack)
    };
    for resource in &mut pass.resources {
        if resource.access == RenderFeatureResourceAccess::Read
            && resource.name == PostProcessGraphResourceNames::SCENE_COLOR
        {
            route_resource_to_produced_input(resource, source);
        }
    }
    retain_single_scene_color_input(pass, source.name);
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
            route_resource_to_produced_input(
                resource,
                PostProcessSceneColorInput {
                    name: PostProcessGraphResourceNames::UPSCALED,
                    producer_pass_name: Some("upscale"),
                },
            );
        }
    }
}

fn route_output_transfer_to_terminal_anti_alias_input(
    pass: &mut RenderFeaturePassDescriptor,
    feature: BuiltinRenderFeature,
    stack: &PostProcessStackDescriptor,
) {
    let Some(producer_pass_name) = terminal_anti_alias_pass_name(stack) else {
        return;
    };
    if feature != BuiltinRenderFeature::PostProcess
        || pass.executor_id.as_str() != "post.output-transfer"
    {
        return;
    }

    for resource in &mut pass.resources {
        if resource.access == RenderFeatureResourceAccess::Read
            && resource.name == PostProcessGraphResourceNames::TONEMAPPED
        {
            route_resource_to_produced_input(
                resource,
                PostProcessSceneColorInput {
                    name: PostProcessGraphResourceNames::FINAL_COMPOSITED,
                    producer_pass_name: Some(producer_pass_name),
                },
            );
        }
    }
}

fn route_upscale_to_terminal_anti_alias_input(
    pass: &mut RenderFeaturePassDescriptor,
    feature: BuiltinRenderFeature,
    stack: &PostProcessStackDescriptor,
) {
    let Some(producer_pass_name) = terminal_anti_alias_pass_name(stack) else {
        return;
    };
    if feature != BuiltinRenderFeature::PostProcess || pass.executor_id.as_str() != "post.upscale" {
        return;
    }

    for resource in &mut pass.resources {
        if resource.access == RenderFeatureResourceAccess::Read
            && resource.name == PostProcessGraphResourceNames::TONEMAPPED
        {
            route_resource_to_produced_input(
                resource,
                PostProcessSceneColorInput {
                    name: PostProcessGraphResourceNames::FINAL_COMPOSITED,
                    producer_pass_name: Some(producer_pass_name),
                },
            );
        }
    }
}

fn terminal_anti_alias_pass_name(stack: &PostProcessStackDescriptor) -> Option<&'static str> {
    if stack_effect_enabled(stack, PostProcessEffectKind::Fxaa) {
        Some(FXAA_PASS_NAME)
    } else if stack_effect_enabled(stack, PostProcessEffectKind::Smaa) {
        Some(SMAA_PASS_NAME)
    } else {
        None
    }
}

fn stack_effect_enabled(stack: &PostProcessStackDescriptor, kind: PostProcessEffectKind) -> bool {
    stack
        .effects
        .iter()
        .any(|effect| effect.enabled && effect.kind == kind)
}
