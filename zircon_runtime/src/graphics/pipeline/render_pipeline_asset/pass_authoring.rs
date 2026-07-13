use std::collections::{BTreeMap, BTreeSet};

use crate::core::framework::render::RenderFrameExtract;
use crate::graphics::feature::{
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderFeatureResourceAccess,
    RenderFeatureResourceDescriptor, RenderFeatureResourceKind, RenderFeatureResourceWriteMode,
};
use crate::graphics::pipeline::declarations::{
    CompiledRenderPipelinePassStage, RenderPassStage, RenderPipelineCompileOptions,
};
use crate::graphics::scene::{
    append_ibl_bake_artifact_graph_plan, ibl_bake_pmrem_pass_name, IBL_BAKE_IRRADIANCE_CUBE_PASS,
    IBL_BAKE_IRRADIANCE_SH9_PASS,
};
use crate::render_graph::{
    CompiledRenderGraph, ExternalResource, RenderGraphAttachmentOps, RenderGraphBuilder,
    RenderPassId, RgBufferHandle, RgTextureHandle,
};

use super::super::validation::stage_pass_descriptors;
use super::graph_resources::{pipeline_graph_resources, PipelineGraphResourcePlan};
use super::resource_descriptors::{buffer_desc_for, texture_desc_for};

pub(super) struct AuthoredRenderGraph {
    pub(super) pass_stages: Vec<CompiledRenderPipelinePassStage>,
    pub(super) graph: CompiledRenderGraph,
}

pub(super) fn author_render_graph(
    pipeline_name: &str,
    stages: &[RenderPassStage],
    descriptors: &[RenderFeatureDescriptor],
    extract: &RenderFrameExtract,
    options: &RenderPipelineCompileOptions,
) -> Result<AuthoredRenderGraph, String> {
    let mut graph = RenderGraphBuilder::new(pipeline_name.to_string());
    let graph_resources = pipeline_graph_resources(descriptors)?;
    let authored_resources = author_graph_resources(&mut graph, &graph_resources, extract, options);
    let pass_stages = author_graph_passes(
        &mut graph,
        stages,
        descriptors,
        &authored_resources,
        options,
    )?;
    let pass_stages = author_environment_ibl_bake_passes(&mut graph, pass_stages, options)?;

    Ok(AuthoredRenderGraph {
        pass_stages,
        graph: graph.compile().map_err(|error| error.to_string())?,
    })
}

struct AuthoredGraphResources {
    graph_resources: BTreeMap<String, PipelineGraphResourcePlan>,
    texture_resources: BTreeMap<String, RgTextureHandle>,
    buffer_resources: BTreeMap<String, RgBufferHandle>,
    external_resources: BTreeMap<String, ExternalResource>,
}

fn author_graph_resources(
    graph: &mut RenderGraphBuilder,
    graph_resources: &BTreeMap<String, PipelineGraphResourcePlan>,
    extract: &RenderFrameExtract,
    options: &RenderPipelineCompileOptions,
) -> AuthoredGraphResources {
    let mut texture_resources = BTreeMap::new();
    let mut buffer_resources = BTreeMap::new();
    let mut external_resources = BTreeMap::new();

    for (name, plan) in graph_resources {
        match plan.kind {
            RenderFeatureResourceKind::Texture => {
                texture_resources.insert(
                    name.clone(),
                    graph.create_texture(texture_desc_for(name, extract, options)),
                );
            }
            RenderFeatureResourceKind::Buffer => {
                buffer_resources.insert(
                    name.clone(),
                    graph.create_buffer(buffer_desc_for(name, extract, plan.minimum_size_bytes)),
                );
            }
            RenderFeatureResourceKind::External => {
                external_resources.insert(
                    name.clone(),
                    graph.import_external_resource_with_binding(name, plan.external_binding),
                );
            }
        }
    }

    AuthoredGraphResources {
        graph_resources: graph_resources.clone(),
        texture_resources,
        buffer_resources,
        external_resources,
    }
}

fn author_graph_passes(
    graph: &mut RenderGraphBuilder,
    stages: &[RenderPassStage],
    descriptors: &[RenderFeatureDescriptor],
    resources: &AuthoredGraphResources,
    options: &RenderPipelineCompileOptions,
) -> Result<Vec<CompiledRenderPipelinePassStage>, String> {
    let mut previous = None;
    let mut pass_stages = Vec::new();
    let mut produced_texture_resources = BTreeSet::<String>::new();

    for stage in stages {
        for pass_descriptor in ordered_stage_pass_descriptors(*stage, descriptors) {
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
                author_pass_resource_access(
                    graph,
                    pass,
                    resource,
                    resources,
                    &mut produced_texture_resources,
                )?;
            }
            if let Some(before) = previous {
                graph
                    .add_dependency(before, pass)
                    .map_err(|error| error.to_string())?;
            }
            previous = Some(pass);
        }
    }

    Ok(pass_stages)
}

fn author_environment_ibl_bake_passes(
    graph: &mut RenderGraphBuilder,
    mut pass_stages: Vec<CompiledRenderPipelinePassStage>,
    options: &RenderPipelineCompileOptions,
) -> Result<Vec<CompiledRenderPipelinePassStage>, String> {
    let Some(request) = options.environment_ibl_bake_request() else {
        return Ok(pass_stages);
    };

    append_ibl_bake_artifact_graph_plan(graph, request).map_err(|error| error.to_string())?;
    let stage = RenderPassStage::AmbientOcclusion;
    let contents = request.required_contents();
    if contents.contains(crate::core::framework::render::IblBakeArtifactContents::PMREM) {
        for mip_level in 0..request.pmrem_mip_count() {
            pass_stages.push(CompiledRenderPipelinePassStage::new(
                ibl_bake_pmrem_pass_name(mip_level),
                stage,
            ));
        }
    }
    if contents.contains(crate::core::framework::render::IblBakeArtifactContents::SH9) {
        pass_stages.push(CompiledRenderPipelinePassStage::new(
            IBL_BAKE_IRRADIANCE_SH9_PASS,
            stage,
        ));
    }
    if contents.contains(crate::core::framework::render::IblBakeArtifactContents::IEM) {
        pass_stages.push(CompiledRenderPipelinePassStage::new(
            IBL_BAKE_IRRADIANCE_CUBE_PASS,
            stage,
        ));
    }

    Ok(pass_stages)
}

fn ordered_stage_pass_descriptors(
    stage: RenderPassStage,
    descriptors: &[RenderFeatureDescriptor],
) -> Vec<RenderFeaturePassDescriptor> {
    let mut passes = stage_pass_descriptors(stage, descriptors);
    order_unique_resource_producers_before_readers(&mut passes);
    if stage == RenderPassStage::PostProcess {
        order_post_process_bloom_after_scene_color_splits(&mut passes);
    }
    passes
}

fn order_unique_resource_producers_before_readers(passes: &mut Vec<RenderFeaturePassDescriptor>) {
    let pass_count = passes.len();
    if pass_count < 2 {
        return;
    }

    let mut outgoing = vec![BTreeSet::<usize>::new(); pass_count];
    let mut indegree = vec![0usize; pass_count];
    for producer_index in 0..pass_count {
        for write in passes[producer_index]
            .resources
            .iter()
            .filter(|resource| resource.access == RenderFeatureResourceAccess::Write)
        {
            let writer_count = passes
                .iter()
                .filter(|pass| {
                    pass.resources.iter().any(|resource| {
                        resource.name == write.name
                            && resource.kind == write.kind
                            && resource.access == RenderFeatureResourceAccess::Write
                    })
                })
                .count();
            if writer_count != 1 {
                continue;
            }
            for (reader_index, reader) in passes.iter().enumerate() {
                if reader_index == producer_index
                    || !reader.resources.iter().any(|resource| {
                        resource.name == write.name
                            && resource.kind == write.kind
                            && resource.access == RenderFeatureResourceAccess::Read
                    })
                {
                    continue;
                }
                if outgoing[producer_index].insert(reader_index) {
                    indegree[reader_index] += 1;
                }
            }
        }
    }

    let mut emitted = vec![false; pass_count];
    let mut order = Vec::with_capacity(pass_count);
    while order.len() < pass_count {
        let Some(next) = (0..pass_count).find(|index| !emitted[*index] && indegree[*index] == 0)
        else {
            return;
        };
        emitted[next] = true;
        order.push(next);
        for dependency in outgoing[next].iter().copied() {
            indegree[dependency] -= 1;
        }
    }

    let original = passes.clone();
    *passes = order
        .into_iter()
        .map(|index| original[index].clone())
        .collect();
}

fn order_post_process_bloom_after_scene_color_splits(
    passes: &mut Vec<RenderFeaturePassDescriptor>,
) {
    let Some(bloom_index) = passes
        .iter()
        .position(|pass| pass.executor_id.as_str() == "post.bloom-extract")
    else {
        return;
    };

    let bloom = passes.remove(bloom_index);
    let after_latest_scene_color_split = passes
        .iter()
        .rposition(|pass| is_bloom_scene_color_input_producer(pass.executor_id.as_str()))
        .map(|index| index + 1)
        .unwrap_or(0);
    let before_exposure = passes
        .iter()
        .position(|pass| is_bloom_downstream_post_process(pass.executor_id.as_str()))
        .unwrap_or(passes.len());
    let insert_index = before_exposure.max(after_latest_scene_color_split);
    passes.insert(insert_index.min(passes.len()), bloom);
}

fn is_bloom_scene_color_input_producer(executor_id: &str) -> bool {
    matches!(
        executor_id,
        "temporal.taa-resolve" | "post.depth-of-field" | "post.motion-blur"
    )
}

fn is_bloom_downstream_post_process(executor_id: &str) -> bool {
    matches!(
        executor_id,
        "post.exposure.histogram"
            | "post.exposure.resolve"
            | "post.screen-space-reflection-reflection-pyramid"
            | "post.screen-space-reflection-reflection-pyramid-coarse"
            | "post.screen-space-reflection-specular-occlusion"
            | "post.screen-space-reflection-resolve"
            | "post.scene-composite"
            | "post.blur"
            | "post.color-lut-bake"
            | "post.uber"
            | "post.upscale"
            | "post.output-transfer"
    )
}

fn author_pass_resource_access(
    graph: &mut RenderGraphBuilder,
    pass: RenderPassId,
    resource: &RenderFeatureResourceDescriptor,
    resources: &AuthoredGraphResources,
    produced_texture_resources: &mut BTreeSet<String>,
) -> Result<(), String> {
    match (resource.kind, resource.access) {
        (RenderFeatureResourceKind::Texture, RenderFeatureResourceAccess::Read) => {
            read_texture_resource(graph, pass, &resource.name, resources)
        }
        (RenderFeatureResourceKind::Texture, RenderFeatureResourceAccess::Write) => {
            write_texture_resource(graph, pass, resource, resources, produced_texture_resources)
        }
        (RenderFeatureResourceKind::Buffer, RenderFeatureResourceAccess::Read) => {
            read_buffer_resource(graph, pass, &resource.name, resources)
        }
        (RenderFeatureResourceKind::Buffer, RenderFeatureResourceAccess::Write) => {
            write_buffer_resource(graph, pass, resource, resources)
        }
        (RenderFeatureResourceKind::External, RenderFeatureResourceAccess::Read) => graph
            .read_external(pass, resources.external_resources[&resource.name])
            .map_err(|error| error.to_string()),
        (RenderFeatureResourceKind::External, RenderFeatureResourceAccess::Write) => {
            write_external_resource(graph, pass, resource, resources)
        }
    }
}

fn read_texture_resource(
    graph: &mut RenderGraphBuilder,
    pass: RenderPassId,
    resource_name: &str,
    resources: &AuthoredGraphResources,
) -> Result<(), String> {
    match resources.graph_resources[resource_name].kind {
        RenderFeatureResourceKind::Texture => graph
            .read_texture(pass, resources.texture_resources[resource_name])
            .map_err(|error| error.to_string()),
        RenderFeatureResourceKind::External => graph
            .read_external(pass, resources.external_resources[resource_name])
            .map_err(|error| error.to_string()),
        RenderFeatureResourceKind::Buffer => unreachable!(
            "texture resource `{}` was compiled as a buffer",
            resource_name
        ),
    }
}

fn write_texture_resource(
    graph: &mut RenderGraphBuilder,
    pass: RenderPassId,
    resource: &RenderFeatureResourceDescriptor,
    resources: &AuthoredGraphResources,
    produced_texture_resources: &mut BTreeSet<String>,
) -> Result<(), String> {
    match resources.graph_resources[&resource.name].kind {
        RenderFeatureResourceKind::Texture => {
            if resource.write_mode == RenderFeatureResourceWriteMode::Storage {
                graph
                    .write_storage_texture(pass, resources.texture_resources[&resource.name])
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
                    .write_texture_with_ops(pass, resources.texture_resources[&resource.name], ops)
                    .map_err(|error| error.to_string())?;
            }
            produced_texture_resources.insert(resource.name.clone());
            Ok(())
        }
        RenderFeatureResourceKind::External => {
            write_external_resource(graph, pass, resource, resources)
        }
        RenderFeatureResourceKind::Buffer => unreachable!(
            "texture resource `{}` was compiled as a buffer",
            resource.name
        ),
    }
}

fn read_buffer_resource(
    graph: &mut RenderGraphBuilder,
    pass: RenderPassId,
    resource_name: &str,
    resources: &AuthoredGraphResources,
) -> Result<(), String> {
    match resources.graph_resources[resource_name].kind {
        RenderFeatureResourceKind::Buffer => graph
            .read_buffer(pass, resources.buffer_resources[resource_name])
            .map_err(|error| error.to_string()),
        RenderFeatureResourceKind::External => graph
            .read_external(pass, resources.external_resources[resource_name])
            .map_err(|error| error.to_string()),
        RenderFeatureResourceKind::Texture => unreachable!(
            "buffer resource `{}` was compiled as a texture",
            resource_name
        ),
    }
}

fn write_buffer_resource(
    graph: &mut RenderGraphBuilder,
    pass: RenderPassId,
    resource: &RenderFeatureResourceDescriptor,
    resources: &AuthoredGraphResources,
) -> Result<(), String> {
    match resources.graph_resources[&resource.name].kind {
        RenderFeatureResourceKind::Buffer => graph
            .write_buffer(pass, resources.buffer_resources[&resource.name])
            .map_err(|error| error.to_string()),
        RenderFeatureResourceKind::External => {
            write_external_resource(graph, pass, resource, resources)
        }
        RenderFeatureResourceKind::Texture => unreachable!(
            "buffer resource `{}` was compiled as a texture",
            resource.name
        ),
    }
}

fn write_external_resource(
    graph: &mut RenderGraphBuilder,
    pass: RenderPassId,
    resource: &RenderFeatureResourceDescriptor,
    resources: &AuthoredGraphResources,
) -> Result<(), String> {
    if resource.write_mode == RenderFeatureResourceWriteMode::Storage {
        graph
            .write_storage_external(pass, resources.external_resources[&resource.name])
            .map_err(|error| error.to_string())
    } else {
        let ops = resource
            .attachment_ops
            .unwrap_or_else(RenderGraphAttachmentOps::load_store);
        graph
            .write_external_with_ops(pass, resources.external_resources[&resource.name], ops)
            .map_err(|error| error.to_string())
    }
}
