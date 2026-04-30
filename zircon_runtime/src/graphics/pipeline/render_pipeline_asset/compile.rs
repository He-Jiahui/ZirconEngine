use std::collections::{BTreeMap, BTreeSet};

use crate::core::framework::render::RenderFrameExtract;
use crate::render_graph::RenderGraphBuilder;
use crate::rhi::{BufferDesc, BufferUsage, TextureDesc, TextureFormat, TextureUsage};

use crate::extract::{FrameHistoryAccess, FrameHistoryBinding, FrameHistorySlot};
use crate::graphics::feature::{
    RenderFeatureDescriptor, RenderFeatureResourceAccess, RenderFeatureResourceKind,
};
use crate::graphics::pipeline::declarations::{
    CompiledRenderPipeline, RenderPassStage, RenderPipelineAsset, RenderPipelineCompileOptions,
    RendererFeatureAsset,
};

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
        let _ = extract;
        validate_renderer_asset(&self.renderer)?;
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
            .map(feature_descriptor)
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
                        graph.create_transient_texture(texture_desc_for(name)),
                    );
                }
                RenderFeatureResourceKind::Buffer => {
                    buffer_resources.insert(
                        name.clone(),
                        graph.create_transient_buffer(buffer_desc_for(name)),
                    );
                }
                RenderFeatureResourceKind::External => {
                    external_resources.insert(name.clone(), graph.import_external_resource(name));
                }
            }
        }
        let mut previous = None;
        for stage in &self.renderer.stages {
            for pass_descriptor in stage_pass_descriptors(*stage, &enabled_descriptors) {
                let pass = graph.add_pass_with_executor_and_declared_queue(
                    pass_descriptor.pass_name.clone(),
                    options.resolve_queue(pass_descriptor.queue),
                    pass_descriptor.queue,
                    Some(pass_descriptor.executor_id.as_str().to_string()),
                );
                graph
                    .set_pass_flags(pass, pass_descriptor.flags)
                    .map_err(|error| error.to_string())?;
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
                            RenderFeatureResourceKind::Texture => graph
                                .write_texture(pass, texture_resources[&resource.name])
                                .map_err(|error| error.to_string())?,
                            RenderFeatureResourceKind::External => graph
                                .write_external(pass, external_resources[&resource.name])
                                .map_err(|error| error.to_string())?,
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
                                RenderFeatureResourceKind::External => graph
                                    .write_external(pass, external_resources[&resource.name])
                                    .map_err(|error| error.to_string())?,
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
                            graph
                                .write_external(pass, external_resources[&resource.name])
                                .map_err(|error| error.to_string())?;
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
            enabled_features,
            required_extract_sections: required_extract_sections.into_iter().collect(),
            capability_requirements,
            history_bindings,
            graph: graph.compile().map_err(|error| error.to_string())?,
        })
    }
}

fn feature_descriptor(feature: &RendererFeatureAsset) -> RenderFeatureDescriptor {
    feature.descriptor()
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
            for resource in &pass.resources {
                if resource.name.trim().is_empty() {
                    return Err(format!(
                        "feature descriptor `{}` pass `{}` resource name must not be empty",
                        descriptor.name, pass.pass_name
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

fn texture_desc_for(name: &str) -> TextureDesc {
    let format = if name.contains("depth") || name.contains("shadow") {
        TextureFormat::Depth32Float
    } else {
        TextureFormat::Rgba8UnormSrgb
    };
    TextureDesc::new(
        name,
        1,
        1,
        format,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED | TextureUsage::COPY_SRC,
    )
}

fn buffer_desc_for(name: &str) -> BufferDesc {
    BufferDesc::new(
        name,
        4,
        BufferUsage::STORAGE | BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
    )
}
