use std::collections::{BTreeMap, BTreeSet};

use crate::core::framework::render::{
    PostProcessGraphResourceNames, RenderFrameExtract, RenderPhase,
};
use crate::render_graph::QueueLane;

use crate::graphics::extract::{FrameHistoryAccess, FrameHistoryBinding, FrameHistorySlot};
use crate::graphics::feature::{
    BuiltinRenderFeature, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
    RenderFeatureResourceAccess, RenderFeatureResourceWriteMode,
};
use crate::graphics::pipeline::declarations::{
    CompiledRenderPipeline, RenderPassStage, RenderPipelineAsset, RenderPipelineCompileOptions,
};

use super::super::validation::validate_renderer_asset;
use super::descriptor_filtering::{feature_descriptor, feature_descriptor_for_options};
use super::graph_resources::pipeline_graph_resources;
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
        let mut enabled_descriptors = enabled_features
            .iter()
            .map(|feature| feature_descriptor_for_options(feature, options))
            .collect::<Vec<_>>();
        maybe_insert_core_scene_particle_descriptor(
            extract,
            options,
            &self.renderer.stages,
            &mut enabled_descriptors,
        );

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

        Ok(CompiledRenderPipeline {
            handle: self.handle,
            name: self.name.clone(),
            renderer_name: self.renderer.name.clone(),
            stages: self.renderer.stages.clone(),
            pass_stages: authored_graph.pass_stages,
            enabled_features,
            required_extract_sections: required_extract_sections.into_iter().collect(),
            capability_requirements,
            history_bindings,
            graph: authored_graph.graph,
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

fn maybe_insert_core_scene_particle_descriptor(
    extract: &RenderFrameExtract,
    options: &RenderPipelineCompileOptions,
    declared_stages: &[RenderPassStage],
    descriptors: &mut Vec<RenderFeatureDescriptor>,
) {
    if !core_scene_particle_descriptor_is_needed(extract, options, declared_stages, descriptors) {
        return;
    }

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
        .with_executor_id(CORE_SCENE_PARTICLE_EXECUTOR_ID)
        .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
        .write_texture(PostProcessGraphResourceNames::SCENE_COLOR)],
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
