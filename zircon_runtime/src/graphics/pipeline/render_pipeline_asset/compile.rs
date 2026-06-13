use std::collections::{BTreeMap, BTreeSet};

use crate::core::framework::render::{
    PostProcessEffectKind, PostProcessGraphResourceNames, PostProcessStackDescriptor,
    RenderFrameExtract, RenderPhase,
};
use crate::core::math::UVec2;
use crate::render_graph::{RenderGraphAttachmentOps, RenderGraphBuilder};
use crate::rhi::{BufferDesc, BufferUsage, TextureDesc, TextureFormat, TextureUsage};

use crate::extract::{FrameHistoryAccess, FrameHistoryBinding, FrameHistorySlot};
use crate::graphics::feature::{
    BuiltinRenderFeature, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
    RenderFeatureResourceAccess, RenderFeatureResourceDescriptor, RenderFeatureResourceKind,
    RenderFeatureResourceWriteMode,
};
use crate::graphics::pipeline::declarations::{
    CompiledRenderPipeline, CompiledRenderPipelinePassStage, RenderPassStage, RenderPipelineAsset,
    RenderPipelineCompileOptions, RendererFeatureAsset,
};
use crate::graphics::visibility::HzbBuilder;

use super::super::validation::{stage_pass_descriptors, validate_renderer_asset};

impl RenderPipelineAsset {
    pub fn compile(&self, extract: &RenderFrameExtract) -> Result<CompiledRenderPipeline, String> {
        self.compile_with_options(extract, &RenderPipelineCompileOptions::default())
    }

    pub fn compile_with_options(
        &self,
        extract: &RenderFrameExtract,
        options: &RenderPipelineCompileOptions,
    ) -> Result<CompiledRenderPipeline, String> {
        validate_core_pipeline_matches_extract(self, extract)?;
        validate_renderer_asset(&self.renderer)?;
        validate_renderer_stage_phase_mapping(self)?;
        let asset_descriptors = self
            .renderer
            .features
            .iter()
            .filter(|feature| feature.enabled)
            .map(feature_descriptor)
            .collect::<Vec<_>>();
        validate_feature_descriptors(&self.renderer.stages, &asset_descriptors)?;

        let enabled_features = self
            .renderer
            .features
            .iter()
            .filter(|feature| {
                feature.enabled
                    && options.permits_feature_asset(feature)
                    && feature
                        .quality_gate
                        .is_none_or(|gate| options.permits_feature(gate))
            })
            .cloned()
            .collect::<Vec<_>>();
        let enabled_descriptors = enabled_features
            .iter()
            .map(|feature| feature_descriptor_for_options(feature, options))
            .collect::<Vec<_>>();

        let mut required_extract_sections = BTreeSet::new();
        let mut capability_requirements = Vec::new();
        let mut history_access_by_slot = BTreeMap::<FrameHistorySlot, FrameHistoryAccess>::new();
        for (feature, descriptor) in enabled_features.iter().zip(&enabled_descriptors) {
            for section in &descriptor.required_extract_sections {
                required_extract_sections.insert(section.clone());
            }
            for requirement in &descriptor.capability_requirements {
                if !capability_requirements.contains(requirement) {
                    capability_requirements.push(*requirement);
                }
            }
            for requirement in &feature.capability_requirements {
                if !capability_requirements.contains(requirement) {
                    capability_requirements.push(*requirement);
                }
            }
            for binding in &descriptor.history_bindings {
                history_access_by_slot
                    .entry(binding.slot)
                    .and_modify(|access| *access = access.merge(binding.access))
                    .or_insert(binding.access);
            }
        }
        let history_bindings = history_access_by_slot
            .into_iter()
            .map(|(slot, access)| FrameHistoryBinding { slot, access })
            .collect::<Vec<_>>();

        let mut graph = RenderGraphBuilder::new(self.name.clone());
        let graph_resources = pipeline_graph_resources(&enabled_descriptors)?;
        let mut texture_resources = BTreeMap::new();
        let mut buffer_resources = BTreeMap::new();
        let mut external_resources = BTreeMap::new();
        for (name, kind) in &graph_resources {
            match kind {
                RenderFeatureResourceKind::Texture => {
                    texture_resources.insert(
                        name.clone(),
                        graph.create_texture(texture_desc_for(name, extract, options)),
                    );
                }
                RenderFeatureResourceKind::Buffer => {
                    buffer_resources.insert(
                        name.clone(),
                        graph.create_buffer(buffer_desc_for(name, extract)),
                    );
                }
                RenderFeatureResourceKind::External => {
                    external_resources.insert(name.clone(), graph.import_external_resource(name));
                }
            }
        }
        let mut previous = None;
        let mut pass_stages = Vec::new();
        let mut produced_texture_resources = BTreeSet::<String>::new();
        for stage in &self.renderer.stages {
            for pass_descriptor in stage_pass_descriptors(*stage, &enabled_descriptors) {
                pass_stages.push(CompiledRenderPipelinePassStage::new(
                    pass_descriptor.pass_name.clone(),
                    *stage,
                ));
                let pass = graph.add_pass_with_executor_and_declared_queue(
                    pass_descriptor.pass_name.clone(),
                    options.resolve_queue(pass_descriptor.queue),
                    pass_descriptor.queue,
                    Some(pass_descriptor.executor_id.as_str().to_string()),
                );
                graph
                    .set_pass_flags(pass, pass_descriptor.flags)
                    .map_err(|error| error.to_string())?;
                if let Some(workload) = pass_descriptor.compute_workload.clone() {
                    graph
                        .set_compute_workload(pass, workload)
                        .map_err(|error| error.to_string())?;
                }
                for resource in &pass_descriptor.resources {
                    match (resource.kind, resource.access) {
                        (RenderFeatureResourceKind::Texture, RenderFeatureResourceAccess::Read) => {
                            match graph_resources[&resource.name] {
                                RenderFeatureResourceKind::Texture => graph
                                    .read_texture(pass, texture_resources[&resource.name])
                                    .map_err(|error| error.to_string())?,
                                RenderFeatureResourceKind::External => graph
                                    .read_external(pass, external_resources[&resource.name])
                                    .map_err(|error| error.to_string())?,
                                RenderFeatureResourceKind::Buffer => unreachable!(
                                    "texture resource `{}` was compiled as a buffer",
                                    resource.name
                                ),
                            }
                        }
                        (
                            RenderFeatureResourceKind::Texture,
                            RenderFeatureResourceAccess::Write,
                        ) => match graph_resources[&resource.name] {
                            RenderFeatureResourceKind::Texture => {
                                if resource.write_mode == RenderFeatureResourceWriteMode::Storage {
                                    graph
                                        .write_storage_texture(
                                            pass,
                                            texture_resources[&resource.name],
                                        )
                                        .map_err(|error| error.to_string())?;
                                } else {
                                    let ops = resource.attachment_ops.unwrap_or_else(|| {
                                        if produced_texture_resources.contains(&resource.name) {
                                            RenderGraphAttachmentOps::load_store()
                                        } else {
                                            RenderGraphAttachmentOps::clear_store()
                                        }
                                    });
                                    graph
                                        .write_texture_with_ops(
                                            pass,
                                            texture_resources[&resource.name],
                                            ops,
                                        )
                                        .map_err(|error| error.to_string())?;
                                }
                                produced_texture_resources.insert(resource.name.clone());
                            }
                            RenderFeatureResourceKind::External => {
                                if resource.write_mode == RenderFeatureResourceWriteMode::Storage {
                                    graph
                                        .write_storage_external(
                                            pass,
                                            external_resources[&resource.name],
                                        )
                                        .map_err(|error| error.to_string())?;
                                } else {
                                    let ops = resource
                                        .attachment_ops
                                        .unwrap_or_else(RenderGraphAttachmentOps::load_store);
                                    graph
                                        .write_external_with_ops(
                                            pass,
                                            external_resources[&resource.name],
                                            ops,
                                        )
                                        .map_err(|error| error.to_string())?;
                                }
                            }
                            RenderFeatureResourceKind::Buffer => unreachable!(
                                "texture resource `{}` was compiled as a buffer",
                                resource.name
                            ),
                        },
                        (RenderFeatureResourceKind::Buffer, RenderFeatureResourceAccess::Read) => {
                            match graph_resources[&resource.name] {
                                RenderFeatureResourceKind::Buffer => graph
                                    .read_buffer(pass, buffer_resources[&resource.name])
                                    .map_err(|error| error.to_string())?,
                                RenderFeatureResourceKind::External => graph
                                    .read_external(pass, external_resources[&resource.name])
                                    .map_err(|error| error.to_string())?,
                                RenderFeatureResourceKind::Texture => unreachable!(
                                    "buffer resource `{}` was compiled as a texture",
                                    resource.name
                                ),
                            }
                        }
                        (RenderFeatureResourceKind::Buffer, RenderFeatureResourceAccess::Write) => {
                            match graph_resources[&resource.name] {
                                RenderFeatureResourceKind::Buffer => graph
                                    .write_buffer(pass, buffer_resources[&resource.name])
                                    .map_err(|error| error.to_string())?,
                                RenderFeatureResourceKind::External => {
                                    if resource.write_mode
                                        == RenderFeatureResourceWriteMode::Storage
                                    {
                                        graph
                                            .write_storage_external(
                                                pass,
                                                external_resources[&resource.name],
                                            )
                                            .map_err(|error| error.to_string())?;
                                    } else {
                                        let ops = resource
                                            .attachment_ops
                                            .unwrap_or_else(RenderGraphAttachmentOps::load_store);
                                        graph
                                            .write_external_with_ops(
                                                pass,
                                                external_resources[&resource.name],
                                                ops,
                                            )
                                            .map_err(|error| error.to_string())?;
                                    }
                                }
                                RenderFeatureResourceKind::Texture => unreachable!(
                                    "buffer resource `{}` was compiled as a texture",
                                    resource.name
                                ),
                            }
                        }
                        (
                            RenderFeatureResourceKind::External,
                            RenderFeatureResourceAccess::Read,
                        ) => {
                            graph
                                .read_external(pass, external_resources[&resource.name])
                                .map_err(|error| error.to_string())?;
                        }
                        (
                            RenderFeatureResourceKind::External,
                            RenderFeatureResourceAccess::Write,
                        ) => {
                            if resource.write_mode == RenderFeatureResourceWriteMode::Storage {
                                graph
                                    .write_storage_external(
                                        pass,
                                        external_resources[&resource.name],
                                    )
                                    .map_err(|error| error.to_string())?;
                            } else {
                                let ops = resource
                                    .attachment_ops
                                    .unwrap_or_else(RenderGraphAttachmentOps::load_store);
                                graph
                                    .write_external_with_ops(
                                        pass,
                                        external_resources[&resource.name],
                                        ops,
                                    )
                                    .map_err(|error| error.to_string())?;
                            }
                        }
                    }
                }
                if let Some(before) = previous {
                    graph
                        .add_dependency(before, pass)
                        .map_err(|error| error.to_string())?;
                }
                previous = Some(pass);
            }
        }

        Ok(CompiledRenderPipeline {
            handle: self.handle,
            name: self.name.clone(),
            renderer_name: self.renderer.name.clone(),
            stages: self.renderer.stages.clone(),
            pass_stages,
            enabled_features,
            required_extract_sections: required_extract_sections.into_iter().collect(),
            capability_requirements,
            history_bindings,
            graph: graph.compile().map_err(|error| error.to_string())?,
        })
    }
}

fn validate_core_pipeline_matches_extract(
    pipeline: &RenderPipelineAsset,
    extract: &RenderFrameExtract,
) -> Result<(), String> {
    if pipeline.core_pipeline != extract.view.core_pipeline {
        return Err(format!(
            "core pipeline mismatch: pipeline `{}` declares {:?} but extract requires {:?}",
            pipeline.name, pipeline.core_pipeline, extract.view.core_pipeline
        ));
    }

    Ok(())
}

fn validate_renderer_stage_phase_mapping(pipeline: &RenderPipelineAsset) -> Result<(), String> {
    for stage in &pipeline.renderer.stages {
        let Some(phase) = render_phase_for_stage(*stage) else {
            continue;
        };
        if !pipeline.phase_mapping.contains(&phase) {
            return Err(format!(
                "renderer `{}` declares stage `{:?}` but pipeline `{}` phase mapping is missing product phase `{:?}`",
                pipeline.renderer.name, stage, pipeline.name, phase
            ));
        }
    }

    Ok(())
}

fn render_phase_for_stage(stage: RenderPassStage) -> Option<RenderPhase> {
    match stage {
        RenderPassStage::DepthPrepass => Some(RenderPhase::Prepass),
        RenderPassStage::Shadow => Some(RenderPhase::Shadow),
        RenderPassStage::Deferred => Some(RenderPhase::Deferred),
        RenderPassStage::Opaque2d => Some(RenderPhase::Opaque2d),
        RenderPassStage::AlphaMask2d => Some(RenderPhase::AlphaMask2d),
        RenderPassStage::Transparent2d => Some(RenderPhase::Transparent2d),
        RenderPassStage::Opaque3d => Some(RenderPhase::Opaque3d),
        RenderPassStage::AlphaMask3d => Some(RenderPhase::AlphaMask3d),
        RenderPassStage::Transparent3d => Some(RenderPhase::Transparent3d),
        RenderPassStage::PostProcess => Some(RenderPhase::PostProcess),
        RenderPassStage::Ui => Some(RenderPhase::Ui),
        RenderPassStage::Overlay => Some(RenderPhase::Overlay),
        RenderPassStage::Debug => Some(RenderPhase::Debug),
        RenderPassStage::AmbientOcclusion
        | RenderPassStage::Lighting
        | RenderPassStage::Opaque
        | RenderPassStage::Transparent => None,
    }
}

fn feature_descriptor(feature: &RendererFeatureAsset) -> RenderFeatureDescriptor {
    feature.descriptor()
}

fn feature_descriptor_for_options(
    feature: &RendererFeatureAsset,
    options: &RenderPipelineCompileOptions,
) -> RenderFeatureDescriptor {
    let mut descriptor = feature.descriptor();
    if feature.is_builtin(BuiltinRenderFeature::Hzb) && !options.enable_hzb_occlusion_culling {
        descriptor = filter_hzb_occlusion_descriptor(descriptor);
    }
    let Some(post_process_stack) = options.post_process_stack.as_ref() else {
        return descriptor;
    };
    let Some(builtin_feature) = feature.builtin_feature() else {
        return descriptor;
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

fn post_process_stack_filters_feature(feature: BuiltinRenderFeature) -> bool {
    matches!(
        feature,
        BuiltinRenderFeature::Bloom
            | BuiltinRenderFeature::ColorGrading
            | BuiltinRenderFeature::HistoryResolve
            | BuiltinRenderFeature::AntiAlias
            | BuiltinRenderFeature::PostProcess
    )
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
    descriptor
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
    (!pass.resources.is_empty()).then_some(pass)
}

fn post_process_pass_can_be_filtered(feature: BuiltinRenderFeature, executor_id: &str) -> bool {
    match (feature, executor_id) {
        (BuiltinRenderFeature::PostProcess, "post.motion-vector-clear")
        | (BuiltinRenderFeature::PostProcess, "post.motion-vector-camera")
        | (BuiltinRenderFeature::PostProcess, "post.motion-vector-object")
        | (BuiltinRenderFeature::PostProcess, "post.motion-vector-tile-max")
        | (BuiltinRenderFeature::PostProcess, "post.motion-vector-tile-max-coarse")
        | (BuiltinRenderFeature::PostProcess, "post.motion-vector-neighbor-max")
        | (BuiltinRenderFeature::PostProcess, "post.depth-of-field-prepare")
        | (BuiltinRenderFeature::PostProcess, "post.screen-space-reflection-reflection-pyramid")
        | (
            BuiltinRenderFeature::PostProcess,
            "post.screen-space-reflection-reflection-pyramid-coarse",
        )
        | (BuiltinRenderFeature::PostProcess, "post.screen-space-reflection-specular-occlusion")
        | (BuiltinRenderFeature::PostProcess, "post.screen-space-reflection-resolve")
        | (BuiltinRenderFeature::PostProcess, "post.stack")
        | (BuiltinRenderFeature::Bloom, "post.bloom-extract" | "post.bloom")
        | (BuiltinRenderFeature::ColorGrading, "post.color-grade")
        | (BuiltinRenderFeature::HistoryResolve, "history.scene-color" | "post.history-resolve")
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
        (BuiltinRenderFeature::ColorGrading, "post.color-grade") => {
            stack_effect_enabled(stack, PostProcessEffectKind::ColorGrading)
        }
        (BuiltinRenderFeature::HistoryResolve, "history.scene-color" | "post.history-resolve") => {
            stack_effect_enabled(stack, PostProcessEffectKind::HistoryResolve)
        }
        (BuiltinRenderFeature::AntiAlias, _) => {
            stack_effect_enabled(stack, PostProcessEffectKind::Fxaa)
        }
        (BuiltinRenderFeature::PostProcess, "post.motion-vector-clear")
        | (BuiltinRenderFeature::PostProcess, "post.motion-vector-camera")
        | (BuiltinRenderFeature::PostProcess, "post.motion-vector-object")
        | (BuiltinRenderFeature::PostProcess, "post.motion-vector-tile-max")
        | (BuiltinRenderFeature::PostProcess, "post.motion-vector-tile-max-coarse")
        | (BuiltinRenderFeature::PostProcess, "post.motion-vector-neighbor-max") => {
            post_process_stack_uses_motion_vectors(stack)
        }
        (BuiltinRenderFeature::PostProcess, "post.depth-of-field-prepare") => {
            post_process_stack_uses_depth_of_field(stack)
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
        (BuiltinRenderFeature::PostProcess, "post.stack") => true,
        _ => true,
    }
}

fn post_process_stack_uses_motion_vectors(stack: &PostProcessStackDescriptor) -> bool {
    stack.initial_resources.iter().any(|resource| {
        resource == PostProcessGraphResourceNames::SCENE_MOTION_VECTOR
            || resource == PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX
    })
}

fn post_process_stack_uses_depth_of_field(stack: &PostProcessStackDescriptor) -> bool {
    stack.effects.iter().any(|effect| {
        effect.enabled
            && effect.produced_outputs.iter().any(|resource| {
                resource == PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC
                    || resource == PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH
            })
    })
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
            | PostProcessGraphResourceNames::GLOBAL_ILLUMINATION
            | PostProcessGraphResourceNames::CONTACT_SHADOW_OCCLUSION
    ) || active_resources.contains(&resource.name)
}

fn stack_effect_enabled(stack: &PostProcessStackDescriptor, kind: PostProcessEffectKind) -> bool {
    stack
        .effects
        .iter()
        .any(|effect| effect.enabled && effect.kind == kind)
}

fn validate_feature_descriptors(
    declared_stages: &[RenderPassStage],
    descriptors: &[RenderFeatureDescriptor],
) -> Result<(), String> {
    validate_descriptor_names(descriptors)?;
    validate_descriptor_stages(declared_stages, descriptors)?;
    validate_descriptor_pass_names(descriptors)?;
    pipeline_graph_resources(descriptors).map(|_| ())
}

fn validate_descriptor_names(descriptors: &[RenderFeatureDescriptor]) -> Result<(), String> {
    for descriptor in descriptors {
        if descriptor.name.trim().is_empty() {
            return Err("feature descriptor name must not be empty".to_string());
        }
        let mut extract_sections = BTreeSet::new();
        for section in &descriptor.required_extract_sections {
            if section.trim().is_empty() {
                return Err(format!(
                    "feature descriptor `{}` extract section name must not be empty",
                    descriptor.name
                ));
            }
            if !extract_sections.insert(section.as_str()) {
                return Err(format!(
                    "feature descriptor `{}` declares duplicate extract section `{}`",
                    descriptor.name, section
                ));
            }
        }
        let mut history_slots = BTreeSet::new();
        for binding in &descriptor.history_bindings {
            if !history_slots.insert(binding.slot) {
                return Err(format!(
                    "feature descriptor `{}` declares duplicate history binding for slot `{:?}`",
                    descriptor.name, binding.slot
                ));
            }
        }
        for pass in &descriptor.stage_passes {
            if pass.pass_name.trim().is_empty() {
                return Err(format!(
                    "feature descriptor `{}` pass name must not be empty",
                    descriptor.name
                ));
            }
            if pass.executor_id.as_str().trim().is_empty() {
                return Err(format!(
                    "feature descriptor `{}` pass `{}` executor id must not be empty",
                    descriptor.name, pass.pass_name
                ));
            }
            if pass.queue == crate::render_graph::QueueLane::AsyncCompute
                && pass.compute_workload.is_none()
            {
                return Err(format!(
                    "feature descriptor `{}` pass `{}` declares `AsyncCompute` queue but no compute workload",
                    descriptor.name, pass.pass_name
                ));
            }
            if let Some(workload) = &pass.compute_workload {
                if pass.queue != crate::render_graph::QueueLane::AsyncCompute {
                    return Err(format!(
                        "feature descriptor `{}` pass `{}` cannot declare compute workload on `{:?}` queue",
                        descriptor.name, pass.pass_name, pass.queue
                    ));
                }
                if workload.pipeline_label.trim().is_empty() {
                    return Err(format!(
                        "feature descriptor `{}` pass `{}` compute workload pipeline label must not be empty",
                        descriptor.name, pass.pass_name
                    ));
                }
                if workload
                    .workgroup_size
                    .iter()
                    .any(|dimension| *dimension == 0)
                {
                    return Err(format!(
                        "feature descriptor `{}` pass `{}` compute workload workgroup size dimensions must be nonzero",
                        descriptor.name, pass.pass_name
                    ));
                }
            }
            for resource in &pass.resources {
                if resource.name.trim().is_empty() {
                    return Err(format!(
                        "feature descriptor `{}` pass `{}` resource name must not be empty",
                        descriptor.name, pass.pass_name
                    ));
                }
                if resource.access == RenderFeatureResourceAccess::Read
                    && resource.attachment_ops.is_some()
                {
                    return Err(format!(
                        "feature descriptor `{}` pass `{}` resource `{}` cannot declare attachment ops for a read access",
                        descriptor.name, pass.pass_name, resource.name
                    ));
                }
                if resource.access == RenderFeatureResourceAccess::Read
                    && resource.write_mode == RenderFeatureResourceWriteMode::Storage
                {
                    return Err(format!(
                        "feature descriptor `{}` pass `{}` resource `{}` cannot declare storage write mode for a read access",
                        descriptor.name, pass.pass_name, resource.name
                    ));
                }
                if resource.write_mode == RenderFeatureResourceWriteMode::Storage
                    && resource.attachment_ops.is_some()
                {
                    return Err(format!(
                        "feature descriptor `{}` pass `{}` resource `{}` cannot declare attachment ops for a storage write",
                        descriptor.name, pass.pass_name, resource.name
                    ));
                }
            }
        }
    }
    Ok(())
}

fn validate_descriptor_stages(
    declared_stages: &[RenderPassStage],
    descriptors: &[RenderFeatureDescriptor],
) -> Result<(), String> {
    for descriptor in descriptors {
        for pass in &descriptor.stage_passes {
            if !declared_stages.contains(&pass.stage) {
                return Err(format!(
                    "feature descriptor `{}` pass `{}` targets undeclared stage `{:?}`",
                    descriptor.name, pass.pass_name, pass.stage
                ));
            }
        }
    }
    Ok(())
}

fn validate_descriptor_pass_names(descriptors: &[RenderFeatureDescriptor]) -> Result<(), String> {
    let mut seen_pass_names = BTreeSet::new();
    for descriptor in descriptors {
        for pass in &descriptor.stage_passes {
            if !seen_pass_names.insert(pass.pass_name.as_str()) {
                return Err(format!(
                    "duplicate render graph pass name `{}` in feature descriptor `{}`",
                    pass.pass_name, descriptor.name
                ));
            }
        }
    }
    Ok(())
}

fn pipeline_graph_resources(
    descriptors: &[RenderFeatureDescriptor],
) -> Result<BTreeMap<String, RenderFeatureResourceKind>, String> {
    let mut resources = BTreeMap::<String, PipelineGraphResourceUsage>::new();
    for descriptor in descriptors {
        for pass in &descriptor.stage_passes {
            for resource in &pass.resources {
                resources
                    .entry(resource.name.clone())
                    .and_modify(|usage| {
                        usage.add_access(
                            &resource.name,
                            resource.kind,
                            resource.access,
                            &descriptor.name,
                            &pass.pass_name,
                        )
                    })
                    .or_insert_with(|| {
                        PipelineGraphResourceUsage::new(resource.kind, resource.access)
                    });
                if let Some(error) = resources
                    .get(&resource.name)
                    .and_then(PipelineGraphResourceUsage::take_error)
                {
                    return Err(error);
                }
            }
        }
    }

    Ok(resources
        .into_iter()
        .map(|(name, usage)| (name, usage.graph_kind()))
        .collect())
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PipelineGraphResourceUsage {
    kind: RenderFeatureResourceKind,
    has_read: bool,
    has_write: bool,
    explicit_external: bool,
    error: Option<String>,
}

impl PipelineGraphResourceUsage {
    fn new(kind: RenderFeatureResourceKind, access: RenderFeatureResourceAccess) -> Self {
        let mut usage = Self {
            kind,
            has_read: false,
            has_write: false,
            explicit_external: kind == RenderFeatureResourceKind::External,
            error: None,
        };
        usage.record_access(access);
        usage
    }

    fn add_access(
        &mut self,
        resource_name: &str,
        kind: RenderFeatureResourceKind,
        access: RenderFeatureResourceAccess,
        descriptor_name: &str,
        pass_name: &str,
    ) {
        if self.conflicts_with(kind) {
            self.error = Some(format!(
                "resource `{resource_name}` has conflicting resource kind or explicit external resource usage in feature descriptor `{descriptor_name}` pass `{pass_name}`"
            ));
            return;
        }
        if kind == RenderFeatureResourceKind::External {
            self.kind = RenderFeatureResourceKind::External;
            self.explicit_external = true;
        }
        self.record_access(access);
    }

    fn conflicts_with(&self, kind: RenderFeatureResourceKind) -> bool {
        if self.kind == kind {
            return false;
        }

        if self.explicit_external || kind == RenderFeatureResourceKind::External {
            return true;
        }

        self.kind != RenderFeatureResourceKind::External
            && kind != RenderFeatureResourceKind::External
    }

    fn record_access(&mut self, access: RenderFeatureResourceAccess) {
        match access {
            RenderFeatureResourceAccess::Read => self.has_read = true,
            RenderFeatureResourceAccess::Write => self.has_write = true,
        }
    }

    fn take_error(&self) -> Option<String> {
        self.error.clone()
    }

    fn graph_kind(self) -> RenderFeatureResourceKind {
        if self.kind == RenderFeatureResourceKind::External || !self.has_write {
            RenderFeatureResourceKind::External
        } else {
            self.kind
        }
    }
}

fn texture_desc_for(
    name: &str,
    extract: &RenderFrameExtract,
    options: &RenderPipelineCompileOptions,
) -> TextureDesc {
    let view_size = extract.view.effective_render_size();
    let base_width = view_size.x.max(1);
    let base_height = view_size.y.max(1);
    let (width, height) = post_process_intermediate_size(name, base_width, base_height);
    let post_process_format = post_process_intermediate_format(name);
    let format = match post_process_format {
        Some(format) => format,
        None if name.contains("depth") || name.contains("shadow") => TextureFormat::Depth32Float,
        None if extract.view.camera.hdr && is_scene_color_resource(name) => {
            TextureFormat::Rgba16Float
        }
        None => TextureFormat::Rgba8UnormSrgb,
    };
    let mut usage =
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED | TextureUsage::COPY_SRC;
    if !format.is_depth() {
        usage |= TextureUsage::STORAGE | TextureUsage::COPY_DST;
    }
    let sample_count = if post_process_format.is_some() || name.contains("shadow") {
        1
    } else {
        options.graph_msaa_sample_count(extract.view.camera.msaa_samples)
    };
    TextureDesc::new(name, width, height, format, usage)
        .with_sample_count(sample_count)
        .with_mip_levels(post_process_intermediate_mip_levels(name, width, height))
}

fn post_process_intermediate_format(name: &str) -> Option<TextureFormat> {
    match name {
        PostProcessGraphResourceNames::SCENE_MOTION_VECTOR
        | PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX
        | PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE
        | PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX
        | PostProcessGraphResourceNames::HZB_FURTHEST
        | PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID
        | PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE => {
            Some(TextureFormat::Rgba16Float)
        }
        PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC
        | PostProcessGraphResourceNames::CONTACT_SHADOW_OCCLUSION
        | PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION => {
            Some(TextureFormat::Rgba8Unorm)
        }
        PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH
        | PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY => {
            Some(TextureFormat::Rgba8UnormSrgb)
        }
        _ => None,
    }
}

fn post_process_intermediate_size(name: &str, width: u32, height: u32) -> (u32, u32) {
    match name {
        PostProcessGraphResourceNames::HZB_FURTHEST => {
            let plan = HzbBuilder::new(UVec2::new(width, height)).build_plan();
            (plan.hzb_size.x, plan.hzb_size.y)
        }
        PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX
        | PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID => {
            (half_extent(width), half_extent(height))
        }
        PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE
        | PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE => {
            let half_width = half_extent(width);
            let half_height = half_extent(height);
            (half_extent(half_width), half_extent(half_height))
        }
        _ => (width, height),
    }
}

fn post_process_intermediate_mip_levels(name: &str, width: u32, height: u32) -> u32 {
    match name {
        PostProcessGraphResourceNames::HZB_FURTHEST => full_mip_chain_level_count(width, height),
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID => {
            full_mip_chain_level_count(width, height)
        }
        _ => 1,
    }
}

fn full_mip_chain_level_count(width: u32, height: u32) -> u32 {
    u32::BITS - width.max(height).max(1).leading_zeros()
}

fn half_extent(value: u32) -> u32 {
    (value.saturating_add(1) / 2).max(1)
}

fn buffer_desc_for(name: &str, extract: &RenderFrameExtract) -> BufferDesc {
    use crate::graphics::scene::lighting::light_grid_builder::{
        LightGridParams, LIGHT_GRID_MAX_TILE_WORDS, LIGHT_GRID_MAX_ZBIN_WORDS,
    };

    let view_size = extract.view.effective_render_size();
    let pixel_count = u64::from(view_size.x.max(1)) * u64::from(view_size.y.max(1));
    let size_bytes = match name {
        PostProcessGraphResourceNames::LIGHT_GRID_PARAMS => {
            std::mem::size_of::<LightGridParams>() as u64
        }
        PostProcessGraphResourceNames::LIGHT_ZBINS => u64::from(LIGHT_GRID_MAX_ZBIN_WORDS) * 4,
        PostProcessGraphResourceNames::LIGHT_TILE_MASKS => u64::from(LIGHT_GRID_MAX_TILE_WORDS) * 4,
        _ => pixel_count.max(1) * 4,
    };
    let usage = match name {
        PostProcessGraphResourceNames::LIGHT_GRID_PARAMS => {
            BufferUsage::UNIFORM | BufferUsage::COPY_DST
        }
        _ => BufferUsage::STORAGE | BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
    };
    BufferDesc::new(name, size_bytes, usage)
}

fn is_scene_color_resource(name: &str) -> bool {
    matches!(
        name,
        "scene-color" | "final-color" | "final-composited" | "bloom-texture" | "ambient-occlusion"
    ) || name.starts_with("gbuffer-")
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::PostProcessGraphResourceNames;
    use crate::core::framework::render::{
        RenderFrameExtract, RenderPhase, RenderPipelineHandle, RenderWorldSnapshotHandle,
    };
    use crate::graphics::feature::{RenderFeatureDescriptor, RenderFeaturePassDescriptor};
    use crate::graphics::pipeline::{RenderPassStage, RenderPipelineAsset, RendererAsset};
    use crate::render_graph::{
        QueueLane, RenderGraphComputeDispatchExtent, RenderGraphComputeWorkload,
    };
    use crate::scene::world::World;

    #[test]
    fn compile_preserves_renderer_stage_for_each_graph_pass() {
        let pipeline = RenderPipelineAsset {
            handle: RenderPipelineHandle::new(77),
            name: "stage-test".to_string(),
            core_pipeline: crate::core::framework::render::CorePipelineKind::Core3d,
            phase_mapping: vec![
                crate::core::framework::render::RenderPhase::Prepass,
                crate::core::framework::render::RenderPhase::Transparent3d,
            ],
            renderer: RendererAsset {
                name: "stage-test-renderer".to_string(),
                stages: vec![
                    RenderPassStage::DepthPrepass,
                    RenderPassStage::Transparent3d,
                ],
                features: vec![crate::graphics::pipeline::RendererFeatureAsset::plugin(
                    RenderFeatureDescriptor::new(
                        "stage-test-feature",
                        Vec::new(),
                        Vec::new(),
                        vec![
                            RenderFeaturePassDescriptor::new(
                                RenderPassStage::Transparent3d,
                                "particle-render",
                                QueueLane::Graphics,
                            )
                            .with_executor_id("particle.transparent"),
                            RenderFeaturePassDescriptor::new(
                                RenderPassStage::DepthPrepass,
                                "depth-prepass",
                                QueueLane::Graphics,
                            )
                            .with_executor_id("mesh.depth-prepass"),
                        ],
                    ),
                )],
            },
        };

        let compiled = pipeline.compile(&test_extract()).unwrap();

        assert_eq!(
            compiled.pass_stage("depth-prepass"),
            Some(RenderPassStage::DepthPrepass)
        );
        assert_eq!(
            compiled.pass_stage("particle-render"),
            Some(RenderPassStage::Transparent3d)
        );
        assert_eq!(compiled.pass_stage("missing-pass"), None);
        assert_eq!(compiled.pass_stages.len(), compiled.graph.passes().len());
    }

    #[test]
    fn compile_preserves_compute_workload_from_feature_descriptor() {
        let pipeline = RenderPipelineAsset {
            handle: RenderPipelineHandle::new(78),
            name: "compute-workload-test".to_string(),
            core_pipeline: crate::core::framework::render::CorePipelineKind::Core3d,
            phase_mapping: vec![RenderPhase::PostProcess],
            renderer: RendererAsset {
                name: "compute-workload-renderer".to_string(),
                stages: vec![RenderPassStage::AmbientOcclusion],
                features: vec![crate::graphics::pipeline::RendererFeatureAsset::plugin(
                    RenderFeatureDescriptor::new(
                        "compute-workload-feature",
                        Vec::new(),
                        Vec::new(),
                        vec![RenderFeaturePassDescriptor::new(
                            RenderPassStage::AmbientOcclusion,
                            "ssao-evaluate",
                            QueueLane::AsyncCompute,
                        )
                        .with_executor_id("ao.ssao-evaluate")
                        .with_compute_workload(RenderGraphComputeWorkload::viewport(
                            "zircon-ssao-pipeline",
                            [8, 8, 1],
                        ))
                        .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
                        .write_storage_external(PostProcessGraphResourceNames::AMBIENT_OCCLUSION)],
                    ),
                )],
            },
        };

        let compiled = pipeline.compile(&test_extract()).unwrap();
        let pass = compiled
            .graph
            .passes()
            .iter()
            .find(|pass| pass.name == "ssao-evaluate")
            .unwrap();
        let workload = pass.compute_workload.as_ref().unwrap();

        assert_eq!(workload.pipeline_label, "zircon-ssao-pipeline");
        assert_eq!(workload.workgroup_size, [8, 8, 1]);
        assert_eq!(
            workload.dispatch_extent,
            RenderGraphComputeDispatchExtent::Viewport
        );
    }

    #[test]
    fn compile_describes_hzb_and_ssr_reflection_pyramids_as_mip_chain_transients() {
        let mut extract = test_extract();
        extract.apply_viewport_size(crate::core::math::UVec2::new(128, 64));
        let compiled = RenderPipelineAsset::default_forward_plus()
            .compile(&extract)
            .unwrap();

        let hzb_furthest = texture_lifetime(&compiled, PostProcessGraphResourceNames::HZB_FURTHEST);
        let reflection_pyramid = texture_lifetime(
            &compiled,
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID,
        );
        let reflection_pyramid_coarse = texture_lifetime(
            &compiled,
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE,
        );

        assert_eq!((hzb_furthest.width, hzb_furthest.height), (64, 32));
        assert_eq!(hzb_furthest.mip_levels, 7);
        assert_eq!(
            (reflection_pyramid.width, reflection_pyramid.height),
            (64, 32)
        );
        assert_eq!(reflection_pyramid.mip_levels, 7);
        assert_eq!(
            (
                reflection_pyramid_coarse.width,
                reflection_pyramid_coarse.height
            ),
            (32, 16)
        );
        assert_eq!(reflection_pyramid_coarse.mip_levels, 1);
    }

    #[test]
    fn compile_describes_hzb_as_half_power_of_two_mip_chain() {
        let mut extract = test_extract();
        extract.apply_viewport_size(crate::core::math::UVec2::new(1923, 1081));
        let compiled = RenderPipelineAsset::default_forward_plus()
            .compile(&extract)
            .unwrap();

        let hzb_furthest = texture_lifetime(&compiled, PostProcessGraphResourceNames::HZB_FURTHEST);
        let hzb_pass = compiled
            .graph
            .passes()
            .iter()
            .find(|pass| pass.name == "hzb-build")
            .expect("default 3D pipelines should build the shared HZB resource");

        assert_eq!((hzb_furthest.width, hzb_furthest.height), (1024, 1024));
        assert_eq!(hzb_furthest.mip_levels, 11);
        assert_eq!(hzb_furthest.format, crate::rhi::TextureFormat::Rgba16Float);
        assert!(!hzb_pass.culled);
        assert!(hzb_pass.flags.has_side_effects);
    }

    #[test]
    fn compile_rejects_compute_workload_on_non_compute_queue() {
        let pipeline = RenderPipelineAsset {
            handle: RenderPipelineHandle::new(79),
            name: "compute-workload-queue-test".to_string(),
            core_pipeline: crate::core::framework::render::CorePipelineKind::Core3d,
            phase_mapping: vec![RenderPhase::PostProcess],
            renderer: RendererAsset {
                name: "compute-workload-queue-renderer".to_string(),
                stages: vec![RenderPassStage::PostProcess],
                features: vec![crate::graphics::pipeline::RendererFeatureAsset::plugin(
                    RenderFeatureDescriptor::new(
                        "invalid-compute-workload-feature",
                        Vec::new(),
                        Vec::new(),
                        vec![RenderFeaturePassDescriptor::new(
                            RenderPassStage::PostProcess,
                            "bad-compute",
                            QueueLane::Graphics,
                        )
                        .with_executor_id("bad.compute")
                        .with_compute_workload(
                            RenderGraphComputeWorkload::fixed("bad-pipeline", [1, 1, 1], [1, 1, 1]),
                        )],
                    ),
                )],
            },
        };

        let error = pipeline.compile(&test_extract()).unwrap_err();

        assert!(
            error.contains(
                "feature descriptor `invalid-compute-workload-feature` pass `bad-compute` cannot declare compute workload on `Graphics` queue"
            ),
            "{error}"
        );
    }

    fn test_extract() -> RenderFrameExtract {
        RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(1),
            World::new().to_render_snapshot(),
        )
    }

    fn texture_lifetime<'a>(
        compiled: &'a crate::graphics::pipeline::CompiledRenderPipeline,
        name: &str,
    ) -> &'a crate::rhi::TextureDesc {
        let lifetime = compiled
            .graph
            .resource_lifetimes()
            .iter()
            .find(|lifetime| lifetime.name == name)
            .unwrap_or_else(|| panic!("missing graph resource lifetime `{name}`"));
        match &lifetime.desc {
            crate::render_graph::RenderGraphResourceDesc::Texture(desc) => desc,
            other => panic!("expected texture desc for `{name}`, got {other:?}"),
        }
    }
}
