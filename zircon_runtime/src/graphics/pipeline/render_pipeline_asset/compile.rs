use std::collections::{BTreeMap, BTreeSet};

use crate::core::framework::render::{
    resolve_subsurface_profile_table, PostProcessGraphResourceNames, RenderFrameExtract,
    RenderPhase,
};
use crate::render_graph::{QueueLane, RenderGraphAttachmentOps};

use crate::graphics::extract::{FrameHistoryAccess, FrameHistoryBinding, FrameHistorySlot};
use crate::graphics::feature::{
    BuiltinRenderFeature, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
    RenderFeatureResourceAccess, RenderFeatureResourceDescriptor, RenderFeatureResourceWriteMode,
};
use crate::graphics::pipeline::declarations::{
    transmission_mesh_pass_name, transmission_scene_copy_pass_name, CompiledRenderPipeline,
    CompiledRenderPipelineParts, RenderPassStage, RenderPipelineAsset,
    RenderPipelineCompileOptions, ADVANCED_PBR_OPAQUE_EXECUTOR_ID, ADVANCED_PBR_OPAQUE_PASS_NAME,
    TRANSMISSION_MESH_EXECUTOR_IDS, TRANSMISSION_SCENE_COPY_EXECUTOR_IDS,
};
use crate::graphics::scene::HALF_RES_TRANSPARENCY_PARTICLE_EXECUTOR_ID;

use super::super::validation::validate_renderer_asset;
use super::descriptor_filtering::{feature_descriptor, feature_descriptor_for_options};
use super::graph_resources::pipeline_graph_resources;
use super::half_resolution_transparency::{
    half_resolution_transparency_enabled, maybe_insert_half_resolution_transparency_passes,
};
use super::pass_authoring::author_render_graph;

const CORE_SCENE_PARTICLE_DESCRIPTOR_NAME: &str = "scene_particles";
const CORE_SCENE_PARTICLE_PLUGIN_FEATURE_NAME: &str = "particle";
const CORE_SCENE_PARTICLE_PASS_NAME: &str = "particle-render";
const CORE_SCENE_PARTICLE_EXECUTOR_ID: &str = "particle.transparent";
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

        let mut enabled_features = self
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
        let mut enabled_descriptors = enabled_features
            .iter()
            .map(|feature| feature_descriptor_for_options(feature, options))
            .collect::<Vec<_>>();
        let deferred_subsurface_supported = enabled_features.iter().any(|feature| {
            feature.is_builtin(crate::graphics::feature::BuiltinRenderFeature::DeferredGeometry)
        }) && enabled_features.iter().any(|feature| {
            feature.is_builtin(crate::graphics::feature::BuiltinRenderFeature::DeferredLighting)
        });
        let subsurface_table = resolve_subsurface_profile_table(
            &extract.lighting.advanced_lighting.subsurface_profiles,
        );
        let view_uses_active_subsurface_profile = extract
            .lighting
            .advanced_lighting
            .subsurface_material_profile_indices
            .iter()
            .any(|profile_id| subsurface_table.profile_is_active(*profile_id));
        let deferred_subsurface_single_sample =
            options.graph_msaa_sample_count(extract.view.camera.msaa_samples) == 1;
        let inactive_descriptor_names = enabled_descriptors
            .iter()
            .filter(|descriptor| {
                (descriptor.requires_advanced_lighting_oit()
                    && extract.lighting.advanced_lighting.oit.is_none())
                    || (descriptor.requires_advanced_lighting_cookies()
                        && extract.lighting.advanced_lighting.cookies.is_empty())
                    || (descriptor.requires_advanced_lighting_irradiance_volumes()
                        && extract
                            .lighting
                            .advanced_lighting
                            .irradiance_volumes
                            .is_empty())
                    || (descriptor.requires_advanced_lighting_planar_capture()
                        && !selected_camera_is_planar_capture(extract))
                    || (descriptor.requires_advanced_lighting_subsurface()
                        && (!view_uses_active_subsurface_profile
                            || !deferred_subsurface_supported
                            || !deferred_subsurface_single_sample))
            })
            .map(|descriptor| descriptor.name.clone())
            .collect::<BTreeSet<_>>();
        enabled_descriptors
            .retain(|descriptor| !inactive_descriptor_names.contains(&descriptor.name));
        enabled_features
            .retain(|feature| !inactive_descriptor_names.contains(feature.feature_name().as_str()));
        maybe_insert_core_scene_particle_descriptor(
            extract,
            options,
            &self.renderer.stages,
            &mut enabled_descriptors,
        );
        maybe_insert_late_forward_opaque_pass(extract, &mut enabled_descriptors)?;
        maybe_insert_transmission_passes(extract, &mut enabled_descriptors)?;
        apply_pass_resource_extensions(&self.renderer.stages, &mut enabled_descriptors)?;
        apply_pass_replacements(&mut enabled_descriptors)?;
        maybe_insert_half_resolution_transparency_passes(
            extract,
            options,
            &self.renderer.stages,
            &mut enabled_descriptors,
        )?;

        let mut required_extract_sections = BTreeSet::new();
        let mut capability_requirements = Vec::new();
        let mut history_access_by_slot = BTreeMap::<FrameHistorySlot, FrameHistoryAccess>::new();
        for descriptor in &enabled_descriptors {
            for section in &descriptor.required_extract_sections {
                required_extract_sections.insert(section.clone());
            }
            for requirement in &descriptor.capability_requirements {
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
        for feature in &enabled_features {
            for requirement in &feature.capability_requirements {
                if !capability_requirements.contains(requirement) {
                    capability_requirements.push(*requirement);
                }
            }
        }
        let history_bindings = history_access_by_slot
            .into_iter()
            .map(|(slot, access)| FrameHistoryBinding { slot, access })
            .collect::<Vec<_>>();

        let authored_graph = author_render_graph(
            &self.name,
            &self.renderer.stages,
            &enabled_descriptors,
            extract,
            options,
        )?;

        Ok(CompiledRenderPipeline::from_parts(
            CompiledRenderPipelineParts {
                handle: self.handle,
                name: self.name.clone(),
                renderer_name: self.renderer.name.clone(),
                stages: self.renderer.stages.clone(),
                pass_stages: authored_graph.pass_stages,
                enabled_features,
                required_extract_sections: required_extract_sections.into_iter().collect(),
                capability_requirements,
                history_bindings,
                environment_ibl_bake_request: options.environment_ibl_bake_request,
                half_resolution_transparency_depth_sigma: options
                    .half_resolution_transparency_depth_sigma,
                graph: authored_graph.graph,
            },
        ))
    }
}

fn maybe_insert_late_forward_opaque_pass(
    extract: &RenderFrameExtract,
    descriptors: &mut [RenderFeatureDescriptor],
) -> Result<(), String> {
    if !extract
        .lighting
        .advanced_lighting
        .material_features
        .requires_late_forward_opaque_pass()
    {
        return Ok(());
    }
    let (descriptor_index, transparent_index) = unique_transparent_mesh_owner(descriptors)?;
    let descriptor = &mut descriptors[descriptor_index];
    let mut pass = descriptor.stage_passes[transparent_index].clone();
    pass.pass_name = ADVANCED_PBR_OPAQUE_PASS_NAME.to_string();
    pass.executor_id = ADVANCED_PBR_OPAQUE_EXECUTOR_ID.into();
    for resource in &mut pass.resources {
        if resource.name == PostProcessGraphResourceNames::SCENE_COLOR
            && resource.access == RenderFeatureResourceAccess::Write
        {
            resource.attachment_ops = Some(RenderGraphAttachmentOps::load_store());
        }
    }
    descriptor.stage_passes.insert(transparent_index, pass);
    Ok(())
}

fn maybe_insert_transmission_passes(
    extract: &RenderFrameExtract,
    descriptors: &mut [RenderFeatureDescriptor],
) -> Result<(), String> {
    let advanced_lighting = &extract.lighting.advanced_lighting;
    let draw_step_count = advanced_lighting.transmission_draw_step_count();
    if draw_step_count == 0 {
        return Ok(());
    }
    let copy_step_count = advanced_lighting.transmission_scene_copy_step_count();

    let (descriptor_index, transparent_index) = unique_transparent_mesh_owner(descriptors)?;
    let descriptor = &mut descriptors[descriptor_index];
    let transparent_template = descriptor.stage_passes[transparent_index].clone();
    let mut transmission_passes = Vec::with_capacity(draw_step_count.saturating_mul(2));

    for step_index in 0..draw_step_count {
        if step_index < copy_step_count {
            transmission_passes.push(
                RenderFeaturePassDescriptor::new(
                    RenderPassStage::Transparent3d,
                    transmission_scene_copy_pass_name(step_index),
                    QueueLane::Graphics,
                )
                .with_executor_id(TRANSMISSION_SCENE_COPY_EXECUTOR_IDS[step_index])
                .read_texture(PostProcessGraphResourceNames::SCENE_COLOR)
                .write_texture(PostProcessGraphResourceNames::TRANSMISSION_SCENE_COLOR),
            );
        }

        let mut draw = transparent_template.clone();
        draw.pass_name = transmission_mesh_pass_name(step_index);
        draw.executor_id = TRANSMISSION_MESH_EXECUTOR_IDS[step_index].into();
        if step_index < copy_step_count {
            draw = draw.read_texture_from(
                PostProcessGraphResourceNames::TRANSMISSION_SCENE_COLOR,
                transmission_scene_copy_pass_name(step_index),
            );
        }
        transmission_passes.push(draw);
    }

    descriptor
        .stage_passes
        .splice(transparent_index..transparent_index, transmission_passes);
    Ok(())
}

fn unique_transparent_mesh_owner(
    descriptors: &[RenderFeatureDescriptor],
) -> Result<(usize, usize), String> {
    let owners = descriptors
        .iter()
        .enumerate()
        .filter_map(|(descriptor_index, descriptor)| {
            descriptor
                .stage_passes
                .iter()
                .position(|pass| pass.pass_name == "transparent-mesh")
                .map(|pass_index| (descriptor_index, pass_index))
        })
        .collect::<Vec<_>>();
    match owners.as_slice() {
        [owner] => Ok(*owner),
        [] => Err(
            "advanced PBR material requires a transparent-mesh pass in the selected pipeline"
                .to_string(),
        ),
        _ => Err("advanced PBR routing requires exactly one transparent-mesh owner".to_string()),
    }
}

fn selected_camera_is_planar_capture(extract: &RenderFrameExtract) -> bool {
    let crate::core::framework::render::RenderCameraTarget::Texture(selected_target) =
        extract.view.selected_camera_target()
    else {
        return false;
    };
    extract
        .lighting
        .advanced_lighting
        .planar_probes
        .iter()
        .any(|probe| probe.capture_target() == Some(*selected_target))
}

fn apply_pass_replacements(descriptors: &mut [RenderFeatureDescriptor]) -> Result<(), String> {
    let replacements = descriptors
        .iter()
        .flat_map(|descriptor| {
            descriptor
                .replaced_passes()
                .map(move |pass_name| (pass_name.to_string(), descriptor.name.clone()))
        })
        .collect::<Vec<_>>();

    let mut owners = BTreeMap::<String, String>::new();
    for (pass_name, owner) in &replacements {
        if let Some(existing_owner) = owners.insert(pass_name.clone(), owner.clone()) {
            return Err(format!(
                "render pass `{pass_name}` has competing replacement owners `{existing_owner}` and `{owner}`"
            ));
        }
        let target_count = descriptors
            .iter()
            .filter(|descriptor| descriptor.name != *owner)
            .flat_map(|descriptor| descriptor.stage_passes.iter())
            .filter(|pass| pass.pass_name == *pass_name)
            .count();
        if target_count != 1 {
            return Err(format!(
                "feature descriptor `{owner}` replaces pass `{pass_name}`, but the enabled graph contains {target_count} eligible targets"
            ));
        }
    }

    for descriptor in descriptors {
        descriptor.stage_passes.retain(|pass| {
            !owners
                .get(&pass.pass_name)
                .is_some_and(|owner| owner != &descriptor.name)
        });
    }
    Ok(())
}

fn apply_pass_resource_extensions(
    stages: &[RenderPassStage],
    descriptors: &mut [RenderFeatureDescriptor],
) -> Result<(), String> {
    let extensions = descriptors
        .iter()
        .flat_map(|descriptor| {
            descriptor
                .resource_extensions()
                .cloned()
                .map(move |extension| (descriptor.name.clone(), extension))
        })
        .collect::<Vec<_>>();

    for (owner, extension) in extensions {
        if extension.target_pass_name.trim().is_empty() {
            return Err(format!(
                "feature descriptor `{owner}` pass resource extension target must not be empty"
            ));
        }
        if extension.resource.name.trim().is_empty() {
            return Err(format!(
                "feature descriptor `{owner}` pass resource extension for `{}` must name a resource",
                extension.target_pass_name
            ));
        }
        let producer_stage =
            unique_producer_stage_for_read_extension(descriptors, &extension.resource);
        let Some(target) = descriptors
            .iter_mut()
            .flat_map(|descriptor| descriptor.stage_passes.iter_mut())
            .find(|pass| pass.pass_name == extension.target_pass_name)
        else {
            continue;
        };
        if !target.resources.iter().any(|resource| {
            resource.name == extension.resource.name
                && resource.kind == extension.resource.kind
                && resource.access == extension.resource.access
        }) {
            target.resources.push(extension.resource);
        }
        if let Some(producer_stage) = producer_stage {
            promote_pass_to_producer_stage(stages, target, producer_stage);
        }
    }

    Ok(())
}

fn unique_producer_stage_for_read_extension(
    descriptors: &[RenderFeatureDescriptor],
    resource_extension: &RenderFeatureResourceDescriptor,
) -> Option<RenderPassStage> {
    if resource_extension.access != RenderFeatureResourceAccess::Read {
        return None;
    }
    let mut producers = descriptors
        .iter()
        .flat_map(|descriptor| descriptor.stage_passes.iter())
        .filter(|pass| {
            pass.resources.iter().any(|resource| {
                resource.name == resource_extension.name
                    && resource.kind == resource_extension.kind
                    && resource.access == RenderFeatureResourceAccess::Write
            })
        });
    let producer = producers.next()?;
    if producers.next().is_some() {
        return None;
    }
    Some(producer.stage)
}

fn promote_pass_to_producer_stage(
    stages: &[RenderPassStage],
    target: &mut RenderFeaturePassDescriptor,
    producer_stage: RenderPassStage,
) {
    let Some(target_index) = stages.iter().position(|stage| *stage == target.stage) else {
        return;
    };
    let Some(producer_index) = stages.iter().position(|stage| *stage == producer_stage) else {
        return;
    };
    if producer_index > target_index {
        target.stage = producer_stage;
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

fn maybe_insert_core_scene_particle_descriptor(
    extract: &RenderFrameExtract,
    options: &RenderPipelineCompileOptions,
    declared_stages: &[RenderPassStage],
    descriptors: &mut Vec<RenderFeatureDescriptor>,
) {
    if !core_scene_particle_descriptor_is_needed(extract, options, declared_stages, descriptors) {
        return;
    }

    let half_resolution = half_resolution_transparency_enabled(extract, options, declared_stages);
    let color_resource = if half_resolution {
        PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_COLOR
    } else {
        PostProcessGraphResourceNames::SCENE_COLOR
    };
    let depth_resource = if half_resolution {
        PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_DEPTH
    } else {
        PostProcessGraphResourceNames::SCENE_DEPTH
    };
    let executor_id = if half_resolution {
        HALF_RES_TRANSPARENCY_PARTICLE_EXECUTOR_ID
    } else {
        CORE_SCENE_PARTICLE_EXECUTOR_ID
    };
    descriptors.push(RenderFeatureDescriptor::new(
        CORE_SCENE_PARTICLE_DESCRIPTOR_NAME,
        vec![
            "view".to_string(),
            "particles".to_string(),
            "visibility".to_string(),
        ],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::Transparent3d,
            CORE_SCENE_PARTICLE_PASS_NAME,
            QueueLane::Graphics,
        )
        .with_executor_id(executor_id)
        .read_texture(depth_resource)
        .write_texture(color_resource)],
    ));
}

fn core_scene_particle_descriptor_is_needed(
    extract: &RenderFrameExtract,
    options: &RenderPipelineCompileOptions,
    declared_stages: &[RenderPassStage],
    descriptors: &[RenderFeatureDescriptor],
) -> bool {
    particle_sprites_intersect_selected_camera_layers(extract)
        && !options
            .disabled_features
            .contains(&BuiltinRenderFeature::Particle)
        && !options
            .disabled_plugin_features
            .contains(CORE_SCENE_PARTICLE_PLUGIN_FEATURE_NAME)
        && declared_stages.contains(&RenderPassStage::Transparent3d)
        && !descriptors
            .iter()
            .any(descriptor_provides_scene_particle_pass)
}

fn particle_sprites_intersect_selected_camera_layers(extract: &RenderFrameExtract) -> bool {
    let camera_layers = extract.view.selected_camera_layers();
    extract
        .particles
        .sprites
        .iter()
        .any(|sprite| camera_layers.intersects(&sprite.render_layer_mask))
}

fn descriptor_provides_scene_particle_pass(descriptor: &RenderFeatureDescriptor) -> bool {
    descriptor.stage_passes.iter().any(|pass| {
        pass.pass_name == CORE_SCENE_PARTICLE_PASS_NAME
            || pass.executor_id.as_str() == CORE_SCENE_PARTICLE_EXECUTOR_ID
    })
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
        let mut replaced_passes = BTreeSet::new();
        for pass_name in descriptor.replaced_passes() {
            if pass_name.trim().is_empty() {
                return Err(format!(
                    "feature descriptor `{}` replacement pass name must not be empty",
                    descriptor.name
                ));
            }
            if !replaced_passes.insert(pass_name) {
                return Err(format!(
                    "feature descriptor `{}` declares duplicate replacement for pass `{pass_name}`",
                    descriptor.name
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
