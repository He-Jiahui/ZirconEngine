use std::collections::{BTreeMap, BTreeSet};

use crate::core::framework::render::RenderFrameExtract;
use crate::graphics::feature::{
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderFeatureResourceAccess,
    RenderFeatureResourceDescriptor, RenderFeatureResourceKind, RenderFeatureResourceWriteMode,
};
use crate::graphics::pipeline::declarations::{
    RenderGraphExecutionPassMetadata, RenderPassStage, RenderPipelineCompileOptions,
};
use crate::graphics::scene::{
    IBL_BAKE_IRRADIANCE_CUBE_PASS, IBL_BAKE_IRRADIANCE_SH9_PASS,
    append_ibl_bake_artifact_graph_plan, ibl_bake_pmrem_pass_name,
};
use crate::render_graph::{
    CompiledRenderGraph, ExternalResource, RenderGraphBufferRange, RenderGraphBuilder,
    RenderGraphExternalResourceType, RenderGraphResourceAccessIntent,
    RenderGraphResourceAccessRange, RenderGraphResourceVersionToken,
    RenderGraphTextureSubresourceRange, RenderPassId, RgBufferHandle, RgTextureHandle,
};

use super::super::validation::stage_pass_descriptors;
use super::graph_resources::{PipelineGraphResourcePlan, pipeline_graph_resources};
use super::resource_descriptors::buffer_desc_from_schema;
use super::resource_schema_catalog::RenderResourceSchemaCatalog;
use crate::graphics::pipeline::RenderGraphCompileCameraTargetFingerprint;
use crate::rhi::TextureUsage;

mod terminal_surface_pass;

use terminal_surface_pass::author_terminal_surface_pass;

pub(super) struct AuthoredRenderGraph {
    pub(super) execution_pass_metadata: Vec<RenderGraphExecutionPassMetadata>,
    pub(super) graph: CompiledRenderGraph,
}

pub(super) fn author_render_graph(
    pipeline_name: &str,
    stages: &[RenderPassStage],
    descriptors: &[RenderFeatureDescriptor],
    extract: &RenderFrameExtract,
    options: &RenderPipelineCompileOptions,
    camera_target: RenderGraphCompileCameraTargetFingerprint,
) -> Result<AuthoredRenderGraph, String> {
    let mut graph = RenderGraphBuilder::new(pipeline_name.to_string());
    let graph_resources = pipeline_graph_resources(descriptors)?;
    let authored_resources = author_graph_resources(&mut graph, graph_resources, extract, options)?;
    let execution_pass_metadata = author_graph_passes(
        &mut graph,
        stages,
        descriptors,
        &authored_resources,
        options,
    )?;
    let execution_pass_metadata =
        author_environment_ibl_bake_passes(&mut graph, execution_pass_metadata, options)?;
    let execution_pass_metadata = author_terminal_surface_pass(
        &mut graph,
        execution_pass_metadata,
        &authored_resources,
        extract,
        camera_target,
    )?;

    Ok(AuthoredRenderGraph {
        execution_pass_metadata,
        graph: graph.compile().map_err(|error| error.to_string())?,
    })
}

struct AuthoredGraphResources {
    graph_resources: BTreeMap<String, PipelineGraphResourcePlan>,
    texture_resources: BTreeMap<String, RgTextureHandle>,
    buffer_resources: BTreeMap<String, RgBufferHandle>,
    external_resources: BTreeMap<String, ExternalResource>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ResourceVersionKey {
    resource_name: String,
    resource_kind: RenderFeatureResourceKind,
    producer_pass_name: String,
}

fn author_graph_resources(
    graph: &mut RenderGraphBuilder,
    graph_resources: BTreeMap<String, PipelineGraphResourcePlan>,
    extract: &RenderFrameExtract,
    options: &RenderPipelineCompileOptions,
) -> Result<AuthoredGraphResources, String> {
    let mut texture_resources = BTreeMap::new();
    let mut buffer_resources = BTreeMap::new();
    let mut external_resources = BTreeMap::new();
    let resource_catalog = RenderResourceSchemaCatalog::new(extract, options);

    for (name, plan) in &graph_resources {
        if plan.texture_view_alias.is_some() {
            continue;
        }
        match plan.kind {
            RenderFeatureResourceKind::Texture => {
                let desc = resource_catalog.texture_desc(name, plan.schema)?;
                if plan.requires_storage_texture
                    && (!desc.usage.contains(TextureUsage::STORAGE)
                        || !desc.format.supports_write_only_storage())
                {
                    return Err(format!(
                        "storage texture resource `{name}` requires an explicit RenderResourceSchema"
                    ));
                }
                texture_resources.insert(name.clone(), graph.create_texture(desc));
            }
            RenderFeatureResourceKind::Buffer => {
                let desc =
                    resource_catalog.buffer_desc(name, plan.schema, plan.minimum_size_bytes)?;
                buffer_resources.insert(name.clone(), graph.create_buffer(desc));
            }
            RenderFeatureResourceKind::External => {
                let external = match plan.schema {
                    Some(schema)
                        if matches!(
                            plan.external_binding.resource_type,
                            RenderGraphExternalResourceType::Buffer
                        ) =>
                    {
                        graph.import_external_buffer_with_usage_and_binding(
                            name,
                            plan.usage,
                            buffer_desc_from_schema(name, schema, plan.minimum_size_bytes)?,
                            plan.external_binding,
                        )
                    }
                    _ => match resource_catalog.external_texture_desc(name, plan.schema)? {
                        Some(desc) => graph.import_external_texture_with_usage_and_binding(
                            name,
                            plan.usage,
                            desc,
                            plan.external_binding,
                        ),
                        None => graph.import_external_resource_with_usage_and_binding(
                            name,
                            plan.usage,
                            plan.external_binding,
                        ),
                    },
                };
                external_resources.insert(name.clone(), external);
            }
        }
    }

    for (name, plan) in &graph_resources {
        let Some(alias) = &plan.texture_view_alias else {
            continue;
        };
        let parent = texture_resources
            .get(&alias.parent_resource)
            .copied()
            .ok_or_else(|| {
                format!(
                    "texture view alias `{name}` references parent `{}` which is not a graph-owned transient texture",
                    alias.parent_resource
                )
            })?;
        let texture = graph
            .create_texture_view_alias(name, parent, alias.range)
            .map_err(|error| error.to_string())?;
        texture_resources.insert(name.clone(), texture);
    }

    Ok(AuthoredGraphResources {
        graph_resources,
        texture_resources,
        buffer_resources,
        external_resources,
    })
}

fn author_graph_passes(
    graph: &mut RenderGraphBuilder,
    stages: &[RenderPassStage],
    descriptors: &[RenderFeatureDescriptor],
    resources: &AuthoredGraphResources,
    options: &RenderPipelineCompileOptions,
) -> Result<Vec<RenderGraphExecutionPassMetadata>, String> {
    let mut execution_pass_metadata = Vec::new();
    let mut produced_resource_versions =
        BTreeMap::<ResourceVersionKey, RenderGraphResourceVersionToken>::new();

    for pass_descriptor in ordered_render_feature_passes(stages, descriptors)? {
        if pass_descriptor.flags.has_side_effects && pass_descriptor.resources.is_empty() {
            return Err(format!(
                "render pass `{}` declares side effects without graph resources; declare the affected graph resource",
                pass_descriptor.pass_name
            ));
        }
        let pass = graph.add_pass_with_executor_and_declared_queue(
            pass_descriptor.pass_name.clone(),
            options.resolve_queue(pass_descriptor.queue),
            pass_descriptor.queue,
            Some(pass_descriptor.executor_id.as_str().to_string()),
        );
        execution_pass_metadata.push(RenderGraphExecutionPassMetadata::new(
            pass,
            pass_descriptor.stage,
        ));
        graph
            .set_pass_flags(pass, pass_descriptor.flags)
            .map_err(|error| error.to_string())?;
        for resource in &pass_descriptor.resources {
            let produced_version = author_pass_resource_access(
                graph,
                pass,
                resource,
                resources,
                &produced_resource_versions,
            )?;
            if let Some(produced_version) = produced_version {
                let version_key = ResourceVersionKey {
                    resource_name: resource.name.clone(),
                    resource_kind: resource.kind,
                    producer_pass_name: pass_descriptor.pass_name.clone(),
                };
                if produced_resource_versions
                    .insert(version_key, produced_version)
                    .is_some()
                {
                    return Err(format!(
                        "render pass `{}` produces resource `{}` more than once",
                        pass_descriptor.pass_name, resource.name
                    ));
                }
            }
        }
        if let Some(workload) = pass_descriptor.compute_workload.clone() {
            graph
                .set_compute_workload(pass, workload)
                .map_err(|error| error.to_string())?;
        }
        if let Some(compute_pass) = &pass_descriptor.compute_pass {
            graph
                .set_compute_pass_metadata(pass, compute_pass.graph_metadata())
                .map_err(|error| error.to_string())?;
        }
    }

    Ok(execution_pass_metadata)
}

pub(super) fn ordered_render_feature_passes(
    stages: &[RenderPassStage],
    descriptors: &[RenderFeatureDescriptor],
) -> Result<Vec<RenderFeaturePassDescriptor>, String> {
    let version_producers = declared_resource_version_producers(descriptors);
    let mut ordered = Vec::new();
    for stage in stages {
        ordered.extend(ordered_stage_pass_descriptors(
            *stage,
            descriptors,
            stages,
            &version_producers,
        )?);
    }
    Ok(ordered)
}

fn author_environment_ibl_bake_passes(
    graph: &mut RenderGraphBuilder,
    mut execution_pass_metadata: Vec<RenderGraphExecutionPassMetadata>,
    options: &RenderPipelineCompileOptions,
) -> Result<Vec<RenderGraphExecutionPassMetadata>, String> {
    let Some(request) = options.environment_ibl_bake_request() else {
        return Ok(execution_pass_metadata);
    };

    let plan =
        append_ibl_bake_artifact_graph_plan(graph, request).map_err(|error| error.to_string())?;
    let mut pass_ids = plan.passes.into_iter().map(|pass| pass.pass_id);
    let stage = RenderPassStage::AmbientOcclusion;
    let contents = request.required_contents();
    if contents.contains(crate::core::framework::render::IblBakeArtifactContents::PMREM) {
        for mip_level in 0..request.pmrem_mip_count() {
            let pass_name = ibl_bake_pmrem_pass_name(mip_level);
            execution_pass_metadata.push(RenderGraphExecutionPassMetadata::new(
                next_ibl_pass_id(&mut pass_ids, &pass_name)?,
                stage,
            ));
        }
    }
    if contents.contains(crate::core::framework::render::IblBakeArtifactContents::SH9) {
        execution_pass_metadata.push(RenderGraphExecutionPassMetadata::new(
            next_ibl_pass_id(&mut pass_ids, IBL_BAKE_IRRADIANCE_SH9_PASS)?,
            stage,
        ));
    }
    if contents.contains(crate::core::framework::render::IblBakeArtifactContents::IEM) {
        execution_pass_metadata.push(RenderGraphExecutionPassMetadata::new(
            next_ibl_pass_id(&mut pass_ids, IBL_BAKE_IRRADIANCE_CUBE_PASS)?,
            stage,
        ));
    }
    if pass_ids.next().is_some() {
        return Err("IBL bake graph plan returned unclassified pass identities".to_string());
    }

    Ok(execution_pass_metadata)
}

fn next_ibl_pass_id(
    pass_ids: &mut impl Iterator<Item = RenderPassId>,
    pass_name: &str,
) -> Result<RenderPassId, String> {
    pass_ids
        .next()
        .ok_or_else(|| format!("IBL bake graph plan omitted pass identity for `{pass_name}`"))
}

fn ordered_stage_pass_descriptors(
    stage: RenderPassStage,
    descriptors: &[RenderFeatureDescriptor],
    stages: &[RenderPassStage],
    version_producers: &BTreeMap<ResourceVersionKey, RenderPassStage>,
) -> Result<Vec<RenderFeaturePassDescriptor>, String> {
    order_explicit_resource_version_consumers(
        stage,
        stage_pass_descriptors(stage, descriptors),
        stages,
        version_producers,
    )
}

fn declared_resource_version_producers(
    descriptors: &[RenderFeatureDescriptor],
) -> BTreeMap<ResourceVersionKey, RenderPassStage> {
    let mut producers = BTreeMap::new();
    for descriptor in descriptors {
        for pass in &descriptor.stage_passes {
            for resource in &pass.resources {
                if resource.access == RenderFeatureResourceAccess::Write {
                    producers.insert(
                        ResourceVersionKey {
                            resource_name: resource.name.clone(),
                            resource_kind: resource.kind,
                            producer_pass_name: pass.pass_name.clone(),
                        },
                        pass.stage,
                    );
                }
            }
        }
    }
    producers
}

fn order_explicit_resource_version_consumers(
    stage: RenderPassStage,
    passes: Vec<RenderFeaturePassDescriptor>,
    stages: &[RenderPassStage],
    version_producers: &BTreeMap<ResourceVersionKey, RenderPassStage>,
) -> Result<Vec<RenderFeaturePassDescriptor>, String> {
    let pass_count = passes.len();
    if pass_count < 2 {
        return Ok(passes);
    }

    let mut outgoing = vec![BTreeSet::<usize>::new(); pass_count];
    let mut indegree = vec![0usize; pass_count];
    let mut producers = BTreeMap::<ResourceVersionKey, usize>::new();
    for (pass_index, pass) in passes.iter().enumerate() {
        for resource in &pass.resources {
            if resource.access == RenderFeatureResourceAccess::Write {
                producers.insert(
                    ResourceVersionKey {
                        resource_name: resource.name.clone(),
                        resource_kind: resource.kind,
                        producer_pass_name: pass.pass_name.clone(),
                    },
                    pass_index,
                );
            }
        }
    }

    for (consumer_index, pass) in passes.iter().enumerate() {
        for resource in &pass.resources {
            let Some(version) = &resource.input_version else {
                continue;
            };
            let is_attachment_load = resource.access == RenderFeatureResourceAccess::Write
                && resource.write_mode == RenderFeatureResourceWriteMode::Attachment
                && resource.attachment_ops.is_some_and(|ops| {
                    ops.load == crate::render_graph::RenderGraphAttachmentLoadOp::Load
                });
            if resource.access != RenderFeatureResourceAccess::Read && !is_attachment_load {
                return Err(format!(
                    "render pass `{}` declares output resource `{}` as a versioned input",
                    pass.pass_name, resource.name
                ));
            }
            if version.resource_name() != resource.name || version.resource_kind() != resource.kind
            {
                return Err(format!(
                    "render pass `{}` versioned input `{}` does not match the declared resource",
                    pass.pass_name, resource.name
                ));
            }
            let version_key = ResourceVersionKey {
                resource_name: version.resource_name().to_string(),
                resource_kind: version.resource_kind(),
                producer_pass_name: version.producer_pass_name().to_string(),
            };
            let Some(&producer_stage) = version_producers.get(&version_key) else {
                return Err(format!(
                    "render pass `{}` uses resource version `{}` from producer `{}`, but that output is not declared by an active feature",
                    pass.pass_name,
                    version.resource_name(),
                    version.producer_pass_name()
                ));
            };
            let producer_stage_index = stages
                .iter()
                .position(|candidate| *candidate == producer_stage)
                .ok_or_else(|| {
                    format!(
                        "render pass `{}` reads resource version `{}` from producer `{}`, but producer stage {:?} is not declared by the renderer",
                        pass.pass_name,
                        version.resource_name(),
                        version.producer_pass_name(),
                        producer_stage
                    )
                })?;
            let consumer_stage_index = stages
                .iter()
                .position(|candidate| *candidate == stage)
                .ok_or_else(|| {
                    format!(
                        "render pass `{}` declares versioned input `{}` in undeclared renderer stage {:?}",
                        pass.pass_name,
                        version.resource_name(),
                        stage
                    )
                })?;
            if producer_stage_index > consumer_stage_index {
                return Err(format!(
                    "render pass `{}` uses resource version `{}` from later producer `{}`",
                    pass.pass_name,
                    version.resource_name(),
                    version.producer_pass_name()
                ));
            }
            if producer_stage != stage {
                continue;
            }
            let Some(&producer_index) = producers.get(&version_key) else {
                return Err(format!(
                    "render pass `{}` uses resource version `{}` from producer `{}`, but that output is absent from its stage",
                    pass.pass_name,
                    version.resource_name(),
                    version.producer_pass_name()
                ));
            };
            if producer_index == consumer_index {
                return Err(format!(
                    "render pass `{}` cannot use its own produced resource version `{}`",
                    pass.pass_name,
                    version.resource_name()
                ));
            }
            if outgoing[producer_index].insert(consumer_index) {
                indegree[consumer_index] += 1;
            }
        }
    }

    let mut emitted = vec![false; pass_count];
    let mut order = Vec::with_capacity(pass_count);
    while order.len() < pass_count {
        let Some(next) = (0..pass_count).find(|index| !emitted[*index] && indegree[*index] == 0)
        else {
            return Err("render feature resource version dependencies contain a cycle".to_string());
        };
        emitted[next] = true;
        order.push(next);
        for dependency in outgoing[next].iter().copied() {
            indegree[dependency] -= 1;
        }
    }

    Ok(order
        .into_iter()
        .map(|index| passes[index].clone())
        .collect())
}

fn author_pass_resource_access(
    graph: &mut RenderGraphBuilder,
    pass: RenderPassId,
    resource: &RenderFeatureResourceDescriptor,
    resources: &AuthoredGraphResources,
    produced_resource_versions: &BTreeMap<ResourceVersionKey, RenderGraphResourceVersionToken>,
) -> Result<Option<RenderGraphResourceVersionToken>, String> {
    let input_version = authored_input_version(resource, produced_resource_versions)?;
    match (resource.kind, resource.access) {
        (RenderFeatureResourceKind::Texture, RenderFeatureResourceAccess::Read) => {
            read_texture_resource(graph, pass, resource, resources, input_version).map(|_| None)
        }
        (RenderFeatureResourceKind::Texture, RenderFeatureResourceAccess::Write) => {
            write_texture_resource(graph, pass, resource, resources, input_version).map(Some)
        }
        (RenderFeatureResourceKind::Buffer, RenderFeatureResourceAccess::Read) => {
            read_buffer_resource(graph, pass, resource, resources, input_version).map(|_| None)
        }
        (RenderFeatureResourceKind::Buffer, RenderFeatureResourceAccess::Write) => {
            write_buffer_resource(graph, pass, resource, resources, input_version).map(Some)
        }
        (RenderFeatureResourceKind::External, RenderFeatureResourceAccess::Read) => {
            match (input_version, resource.access_metadata) {
                (Some(input_version), Some(metadata)) => graph
                    .read_external_with_access_from_version(
                        pass,
                        input_version,
                        metadata.range,
                        metadata.intent,
                    )
                    .map_err(|error| error.to_string()),
                (Some(input_version), None) => graph
                    .read_external_from_version(pass, input_version)
                    .map_err(|error| error.to_string()),
                (None, Some(metadata)) => graph
                    .read_external_with_access(
                        pass,
                        resources.external_resources[&resource.name],
                        metadata.range,
                        metadata.intent,
                    )
                    .map_err(|error| error.to_string()),
                (None, None) => graph
                    .read_external(pass, resources.external_resources[&resource.name])
                    .map_err(|error| error.to_string()),
            }
            .map(|_| None)
        }
        (RenderFeatureResourceKind::External, RenderFeatureResourceAccess::Write) => {
            write_external_resource(graph, pass, resource, resources, input_version).map(Some)
        }
    }
}

fn authored_input_version(
    resource: &RenderFeatureResourceDescriptor,
    produced_resource_versions: &BTreeMap<ResourceVersionKey, RenderGraphResourceVersionToken>,
) -> Result<Option<RenderGraphResourceVersionToken>, String> {
    let Some(input_version) = &resource.input_version else {
        return Ok(None);
    };
    let key = ResourceVersionKey {
        resource_name: input_version.resource_name().to_string(),
        resource_kind: input_version.resource_kind(),
        producer_pass_name: input_version.producer_pass_name().to_string(),
    };
    produced_resource_versions
        .get(&key)
        .copied()
        .map(Some)
        .ok_or_else(|| {
            format!(
                "render pass versioned input `{}` from producer `{}` has no authored graph token",
                input_version.resource_name(),
                input_version.producer_pass_name(),
            )
        })
}

fn read_texture_resource(
    graph: &mut RenderGraphBuilder,
    pass: RenderPassId,
    resource: &RenderFeatureResourceDescriptor,
    resources: &AuthoredGraphResources,
    input_version: Option<RenderGraphResourceVersionToken>,
) -> Result<(), String> {
    let resource_name = resource.name.as_str();
    let access_metadata = texture_access_metadata(resource)?;
    match resources.graph_resources[resource_name].kind {
        RenderFeatureResourceKind::Texture => match input_version {
            Some(input_version) => match access_metadata {
                Some((range, intent)) => graph
                    .read_texture_with_access_from_version(pass, input_version, range, intent)
                    .map_err(|error| error.to_string()),
                None => graph
                    .read_texture_from_version(pass, input_version)
                    .map_err(|error| error.to_string()),
            },
            None => match access_metadata {
                Some((range, intent)) => graph
                    .read_texture_with_access(
                        pass,
                        resources.texture_resources[resource_name],
                        range,
                        intent,
                    )
                    .map_err(|error| error.to_string()),
                None => graph
                    .read_texture(pass, resources.texture_resources[resource_name])
                    .map_err(|error| error.to_string()),
            },
        },
        RenderFeatureResourceKind::External => match input_version {
            Some(input_version) => graph
                .read_external_from_version(pass, input_version)
                .map_err(|error| error.to_string()),
            None => graph
                .read_external(pass, resources.external_resources[resource_name])
                .map_err(|error| error.to_string()),
        },
        RenderFeatureResourceKind::Buffer => Err(format!(
            "texture resource `{resource_name}` was compiled as a buffer"
        )),
    }
}

fn write_texture_resource(
    graph: &mut RenderGraphBuilder,
    pass: RenderPassId,
    resource: &RenderFeatureResourceDescriptor,
    resources: &AuthoredGraphResources,
    input_version: Option<RenderGraphResourceVersionToken>,
) -> Result<RenderGraphResourceVersionToken, String> {
    let access_metadata = texture_access_metadata(resource)?;
    match resources.graph_resources[&resource.name].kind {
        RenderFeatureResourceKind::Texture => {
            if let Some((range, intent)) = access_metadata {
                if input_version.is_some() {
                    return Err(format!(
                        "render pass scoped texture write `{}` cannot consume an attachment resource version",
                        resource.name
                    ));
                }
                return graph
                    .write_texture_with_access_versioned(
                        pass,
                        resources.texture_resources[&resource.name],
                        range,
                        intent,
                        resource.attachment_ops,
                    )
                    .map_err(|error| error.to_string());
            }
            if resource.write_mode == RenderFeatureResourceWriteMode::Storage {
                if input_version.is_some() {
                    return Err(format!(
                        "render pass storage write `{}` cannot consume an attachment resource version",
                        resource.name
                    ));
                }
                graph
                    .write_storage_texture_versioned(
                        pass,
                        resources.texture_resources[&resource.name],
                    )
                    .map_err(|error| error.to_string())
            } else {
                let ops = resource.attachment_ops.ok_or_else(|| {
                    format!(
                        "render pass attachment write `{}` is missing normalized initialization",
                        resource.name
                    )
                })?;
                match input_version {
                    Some(input_version) => graph
                        .write_texture_with_ops_from_version(
                            pass,
                            resources.texture_resources[&resource.name],
                            ops,
                            input_version,
                        )
                        .map_err(|error| error.to_string()),
                    None => graph
                        .write_texture_with_ops_versioned(
                            pass,
                            resources.texture_resources[&resource.name],
                            ops,
                        )
                        .map_err(|error| error.to_string()),
                }
            }
        }
        RenderFeatureResourceKind::External => {
            write_external_resource(graph, pass, resource, resources, input_version)
        }
        RenderFeatureResourceKind::Buffer => Err(format!(
            "texture resource `{}` was compiled as a buffer",
            resource.name
        )),
    }
}

fn read_buffer_resource(
    graph: &mut RenderGraphBuilder,
    pass: RenderPassId,
    resource: &RenderFeatureResourceDescriptor,
    resources: &AuthoredGraphResources,
    input_version: Option<RenderGraphResourceVersionToken>,
) -> Result<(), String> {
    let resource_name = resource.name.as_str();
    let access_metadata = buffer_access_metadata(resource)?;
    match resources.graph_resources[resource_name].kind {
        RenderFeatureResourceKind::Buffer => match input_version {
            Some(input_version) => match access_metadata {
                Some((range, intent)) => graph
                    .read_buffer_with_access_from_version(pass, input_version, range, intent)
                    .map_err(|error| error.to_string()),
                None => graph
                    .read_buffer_from_version(pass, input_version)
                    .map_err(|error| error.to_string()),
            },
            None => match access_metadata {
                Some((range, intent)) => graph
                    .read_buffer_with_access(
                        pass,
                        resources.buffer_resources[resource_name],
                        range,
                        intent,
                    )
                    .map_err(|error| error.to_string()),
                None => graph
                    .read_buffer(pass, resources.buffer_resources[resource_name])
                    .map_err(|error| error.to_string()),
            },
        },
        RenderFeatureResourceKind::External => match input_version {
            Some(input_version) => graph
                .read_external_from_version(pass, input_version)
                .map_err(|error| error.to_string()),
            None => graph
                .read_external(pass, resources.external_resources[resource_name])
                .map_err(|error| error.to_string()),
        },
        RenderFeatureResourceKind::Texture => Err(format!(
            "buffer resource `{resource_name}` was compiled as a texture"
        )),
    }
}

fn write_buffer_resource(
    graph: &mut RenderGraphBuilder,
    pass: RenderPassId,
    resource: &RenderFeatureResourceDescriptor,
    resources: &AuthoredGraphResources,
    input_version: Option<RenderGraphResourceVersionToken>,
) -> Result<RenderGraphResourceVersionToken, String> {
    let access_metadata = buffer_access_metadata(resource)?;
    match resources.graph_resources[&resource.name].kind {
        RenderFeatureResourceKind::Buffer => {
            if input_version.is_some() {
                return Err(format!(
                    "render pass buffer write `{}` cannot consume an attachment resource version",
                    resource.name
                ));
            }
            match access_metadata {
                Some((range, intent)) => graph
                    .write_buffer_with_access_versioned(
                        pass,
                        resources.buffer_resources[&resource.name],
                        range,
                        intent,
                    )
                    .map_err(|error| error.to_string()),
                None => graph
                    .write_buffer_versioned(pass, resources.buffer_resources[&resource.name])
                    .map_err(|error| error.to_string()),
            }
        }
        RenderFeatureResourceKind::External => {
            write_external_resource(graph, pass, resource, resources, input_version)
        }
        RenderFeatureResourceKind::Texture => Err(format!(
            "buffer resource `{}` was compiled as a texture",
            resource.name
        )),
    }
}

fn texture_access_metadata(
    resource: &RenderFeatureResourceDescriptor,
) -> Result<
    Option<(
        RenderGraphTextureSubresourceRange,
        RenderGraphResourceAccessIntent,
    )>,
    String,
> {
    match resource.access_metadata {
        Some(
            metadata @ crate::render_graph::RenderGraphResourceAccessMetadata {
                range: RenderGraphResourceAccessRange::Texture(range),
                ..
            },
        ) => Ok(Some((range, metadata.intent))),
        Some(_) => Err(format!(
            "render pass resource `{}` is a texture but has a non-texture access scope",
            resource.name
        )),
        None => Ok(None),
    }
}

fn buffer_access_metadata(
    resource: &RenderFeatureResourceDescriptor,
) -> Result<Option<(RenderGraphBufferRange, RenderGraphResourceAccessIntent)>, String> {
    match resource.access_metadata {
        Some(
            metadata @ crate::render_graph::RenderGraphResourceAccessMetadata {
                range: RenderGraphResourceAccessRange::Buffer(range),
                ..
            },
        ) => Ok(Some((range, metadata.intent))),
        Some(_) => Err(format!(
            "render pass resource `{}` is a buffer but has a non-buffer access scope",
            resource.name
        )),
        None => Ok(None),
    }
}

fn write_external_resource(
    graph: &mut RenderGraphBuilder,
    pass: RenderPassId,
    resource: &RenderFeatureResourceDescriptor,
    resources: &AuthoredGraphResources,
    input_version: Option<RenderGraphResourceVersionToken>,
) -> Result<RenderGraphResourceVersionToken, String> {
    if resource.write_mode == RenderFeatureResourceWriteMode::Storage {
        if input_version.is_some() {
            return Err(format!(
                "render pass external storage write `{}` cannot consume an attachment resource version",
                resource.name
            ));
        }
        match resource.access_metadata {
            Some(metadata) => graph.write_external_with_access_versioned(
                pass,
                resources.external_resources[&resource.name],
                metadata.range,
                metadata.intent,
                None,
            ),
            None => graph.write_storage_external_versioned(
                pass,
                resources.external_resources[&resource.name],
            ),
        }
        .map_err(|error| error.to_string())
    } else {
        let ops = resource.attachment_ops.ok_or_else(|| {
            format!(
                "render pass external attachment write `{}` is missing normalized initialization",
                resource.name
            )
        })?;
        match (input_version, resource.access_metadata) {
            (Some(input_version), Some(metadata)) => graph
                .write_external_with_access_from_version(
                    pass,
                    resources.external_resources[&resource.name],
                    metadata.range,
                    metadata.intent,
                    ops,
                    input_version,
                )
                .map_err(|error| error.to_string()),
            (Some(input_version), None) => graph
                .write_external_with_ops_from_version(
                    pass,
                    resources.external_resources[&resource.name],
                    ops,
                    input_version,
                )
                .map_err(|error| error.to_string()),
            (None, Some(metadata)) => graph
                .write_external_with_access_versioned(
                    pass,
                    resources.external_resources[&resource.name],
                    metadata.range,
                    metadata.intent,
                    Some(ops),
                )
                .map_err(|error| error.to_string()),
            (None, None) => graph
                .write_external_with_ops_versioned(
                    pass,
                    resources.external_resources[&resource.name],
                    ops,
                )
                .map_err(|error| error.to_string()),
        }
    }
}
