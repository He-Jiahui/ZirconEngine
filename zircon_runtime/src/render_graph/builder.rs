use std::sync::atomic::{AtomicU64, Ordering};

use crate::rhi::{BufferDesc, TextureDesc, TextureDimension};

use super::access::{
    RenderGraphResourceAccessMetadata, RenderGraphTextureAspect, RenderGraphTextureSubresourceRange,
};
use super::error::RenderGraphError;
use super::types::{
    ExternalResource, PassFlags, QueueLane, RenderGraphAttachmentOps,
    RenderGraphComputePassMetadata, RenderGraphComputeWorkload, RenderGraphExternalResourceBinding,
    RenderGraphExternalResourceType, RenderGraphResource, RenderGraphResourceDesc,
    RenderGraphResourceKind, RenderGraphResourceUsageFlags, RenderGraphResourceVersionToken,
    RenderGraphTextureViewAlias, RenderPassId, RgBufferHandle, RgTextureHandle,
};

mod access_authoring;
mod access_scope_tracker;
mod access_validation;
mod compile;
mod resource_dependency_inference;

static NEXT_RENDER_GRAPH_BUILDER_GENERATION: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ResourceAccessKind {
    Read,
    Write,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ResourceAccess {
    resource: RenderGraphResource,
    kind: ResourceAccessKind,
    input_version: Option<RenderGraphResourceVersionToken>,
    attachment_ops: Option<RenderGraphAttachmentOps>,
    metadata: RenderGraphResourceAccessMetadata,
}

#[derive(Clone, Debug)]
struct RenderPassNode {
    id: RenderPassId,
    name: String,
    declared_queue: QueueLane,
    queue: QueueLane,
    flags: PassFlags,
    executor_id: Option<String>,
    compute_workload: Option<RenderGraphComputeWorkload>,
    compute_pass_metadata: Option<RenderGraphComputePassMetadata>,
    dependencies: Vec<RenderPassId>,
    resources: Vec<ResourceAccess>,
}

#[derive(Clone, Debug)]
struct ResourceNode {
    resource: RenderGraphResource,
    name: String,
    desc: RenderGraphResourceDesc,
    external_binding: RenderGraphExternalResourceBinding,
    external_texture_desc: Option<TextureDesc>,
    external_buffer_desc: Option<BufferDesc>,
    external_alias_group: Option<String>,
    texture_view_alias: Option<RenderGraphTextureViewAlias>,
    usage: RenderGraphResourceUsageFlags,
}

#[derive(Clone, Debug)]
pub struct RenderGraphBuilder {
    name: String,
    generation: u64,
    passes: Vec<RenderPassNode>,
    resources: Vec<ResourceNode>,
    next_texture: usize,
    next_buffer: usize,
    next_external_resource: usize,
}

impl RenderGraphBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            generation: next_render_graph_builder_generation(),
            passes: Vec::new(),
            resources: Vec::new(),
            next_texture: 0,
            next_buffer: 0,
            next_external_resource: 0,
        }
    }

    pub fn add_pass(&mut self, name: impl Into<String>, queue: QueueLane) -> RenderPassId {
        self.add_pass_with_executor(name, queue, None::<String>)
    }

    pub fn add_pass_with_executor(
        &mut self,
        name: impl Into<String>,
        queue: QueueLane,
        executor_id: Option<impl Into<String>>,
    ) -> RenderPassId {
        self.add_pass_with_executor_and_declared_queue(name, queue, queue, executor_id)
    }

    pub fn add_pass_with_executor_and_declared_queue(
        &mut self,
        name: impl Into<String>,
        queue: QueueLane,
        declared_queue: QueueLane,
        executor_id: Option<impl Into<String>>,
    ) -> RenderPassId {
        let id = RenderPassId::from_index(self.passes.len(), self.generation);
        self.passes.push(RenderPassNode {
            id,
            name: name.into(),
            declared_queue,
            queue,
            flags: PassFlags::default(),
            executor_id: executor_id.map(Into::into),
            compute_workload: None,
            compute_pass_metadata: None,
            dependencies: Vec::new(),
            resources: Vec::new(),
        });
        id
    }

    pub fn set_compute_workload(
        &mut self,
        pass: RenderPassId,
        workload: RenderGraphComputeWorkload,
    ) -> Result<(), RenderGraphError> {
        self.ensure_pass(pass)?;
        self.passes[pass.0].compute_workload = Some(workload);
        Ok(())
    }

    pub fn set_compute_pass_metadata(
        &mut self,
        pass: RenderPassId,
        metadata: RenderGraphComputePassMetadata,
    ) -> Result<(), RenderGraphError> {
        self.ensure_pass(pass)?;
        self.passes[pass.0].compute_pass_metadata = Some(metadata);
        Ok(())
    }

    pub fn set_pass_flags(
        &mut self,
        pass: RenderPassId,
        flags: PassFlags,
    ) -> Result<(), RenderGraphError> {
        self.ensure_pass(pass)?;
        self.passes[pass.0].flags = flags;
        Ok(())
    }

    pub fn add_dependency(
        &mut self,
        before: RenderPassId,
        after: RenderPassId,
    ) -> Result<(), RenderGraphError> {
        self.ensure_pass(before)?;
        self.ensure_pass(after)?;
        let pass = &mut self.passes[after.0];
        if !pass.dependencies.contains(&before) {
            pass.dependencies.push(before);
        }
        Ok(())
    }

    pub fn create_texture(&mut self, desc: TextureDesc) -> RgTextureHandle {
        let id = self.next_texture;
        self.next_texture += 1;
        let handle = RgTextureHandle::from_index(id, self.generation);
        let name = desc
            .label
            .clone()
            .unwrap_or_else(|| format!("rg-texture-{id}"));
        self.resources.push(ResourceNode {
            resource: RenderGraphResource::TransientTexture(handle),
            name,
            desc: RenderGraphResourceDesc::Texture(desc),
            external_binding: RenderGraphExternalResourceBinding::report_only(),
            external_texture_desc: None,
            external_buffer_desc: None,
            external_alias_group: None,
            texture_view_alias: None,
            usage: RenderGraphResourceUsageFlags::default(),
        });
        handle
    }

    /// Declares a distinct logical texture resource backed by an exact view of
    /// a graph-owned transient texture. The view never receives an allocation
    /// slot of its own.
    pub fn create_texture_view_alias(
        &mut self,
        name: impl Into<String>,
        parent: RgTextureHandle,
        range: RenderGraphTextureSubresourceRange,
    ) -> Result<RgTextureHandle, RenderGraphError> {
        let name = name.into();
        self.ensure_resource(RenderGraphResource::TransientTexture(parent))?;
        let parent_node = self
            .resources
            .iter()
            .find(|node| node.resource == RenderGraphResource::TransientTexture(parent))
            .ok_or_else(|| RenderGraphError::ResourceDeclarationMissing {
                resource: format!("{parent:?}"),
            })?;
        if parent_node.texture_view_alias.is_some() {
            return Err(RenderGraphError::TextureViewAliasParentIsAlias {
                alias: name,
                parent_name: parent_node.name.clone(),
            });
        }
        let RenderGraphResourceDesc::Texture(parent_desc) = &parent_node.desc else {
            return Err(RenderGraphError::ResourceDeclarationMissing {
                resource: parent_node.name.clone(),
            });
        };
        let desc = texture_view_alias_desc(&name, &parent_node.name, parent_desc, range)?;

        let id = self.next_texture;
        self.next_texture += 1;
        let handle = RgTextureHandle::from_index(id, self.generation);
        self.resources.push(ResourceNode {
            resource: RenderGraphResource::TransientTexture(handle),
            name,
            desc: RenderGraphResourceDesc::Texture(desc),
            external_binding: RenderGraphExternalResourceBinding::report_only(),
            external_texture_desc: None,
            external_buffer_desc: None,
            external_alias_group: None,
            texture_view_alias: Some(RenderGraphTextureViewAlias::new(parent, range)),
            usage: RenderGraphResourceUsageFlags::default(),
        });
        Ok(handle)
    }

    pub fn create_buffer(&mut self, desc: BufferDesc) -> RgBufferHandle {
        let id = self.next_buffer;
        self.next_buffer += 1;
        let handle = RgBufferHandle::from_index(id, self.generation);
        let name = desc
            .label
            .clone()
            .unwrap_or_else(|| format!("rg-buffer-{id}"));
        self.resources.push(ResourceNode {
            resource: RenderGraphResource::TransientBuffer(handle),
            name,
            desc: RenderGraphResourceDesc::Buffer(desc),
            external_binding: RenderGraphExternalResourceBinding::report_only(),
            external_texture_desc: None,
            external_buffer_desc: None,
            external_alias_group: None,
            texture_view_alias: None,
            usage: RenderGraphResourceUsageFlags::default(),
        });
        handle
    }

    /// Imports an external resource without assigning it a cull-root role.
    pub fn import_external_resource(&mut self, name: impl Into<String>) -> ExternalResource {
        self.import_external_resource_with_usage(name, RenderGraphResourceUsageFlags::default())
    }

    /// Imports the external result that this graph presents to its consumer.
    pub fn import_present_external_resource(
        &mut self,
        name: impl Into<String>,
    ) -> ExternalResource {
        self.import_external_resource_with_usage(name, RenderGraphResourceUsageFlags::present())
    }

    /// Imports an external resource binding without assigning it a cull-root role.
    pub fn import_external_resource_with_binding(
        &mut self,
        name: impl Into<String>,
        external_binding: RenderGraphExternalResourceBinding,
    ) -> ExternalResource {
        self.import_external_resource_with_usage_and_binding(
            name,
            RenderGraphResourceUsageFlags::default(),
            external_binding,
        )
    }

    /// Imports an externally bound result that this graph presents to its consumer.
    pub fn import_present_external_resource_with_binding(
        &mut self,
        name: impl Into<String>,
        external_binding: RenderGraphExternalResourceBinding,
    ) -> ExternalResource {
        self.import_external_resource_with_usage_and_binding(
            name,
            RenderGraphResourceUsageFlags::present(),
            external_binding,
        )
    }

    /// Imports an external texture without assigning it a cull-root role.
    pub fn import_external_texture_with_binding(
        &mut self,
        name: impl Into<String>,
        texture_desc: TextureDesc,
        external_binding: RenderGraphExternalResourceBinding,
    ) -> ExternalResource {
        self.import_external_texture_with_usage_and_binding(
            name,
            RenderGraphResourceUsageFlags::default(),
            texture_desc,
            external_binding,
        )
    }

    /// Imports a presented external texture whose physical shape is part of the graph contract.
    pub fn import_present_external_texture_with_binding(
        &mut self,
        name: impl Into<String>,
        texture_desc: TextureDesc,
        external_binding: RenderGraphExternalResourceBinding,
    ) -> ExternalResource {
        self.import_external_texture_with_usage_and_binding(
            name,
            RenderGraphResourceUsageFlags::present(),
            texture_desc,
            external_binding,
        )
    }

    /// Imports a physical external texture with an explicit graph lifetime role.
    pub fn import_external_texture_with_usage_and_binding(
        &mut self,
        name: impl Into<String>,
        usage: RenderGraphResourceUsageFlags,
        texture_desc: TextureDesc,
        external_binding: RenderGraphExternalResourceBinding,
    ) -> ExternalResource {
        let external_binding = match external_binding.resource_type {
            RenderGraphExternalResourceType::Unknown => RenderGraphExternalResourceBinding {
                resource_type: RenderGraphExternalResourceType::Texture,
                requirement: external_binding.requirement,
            },
            _ => external_binding,
        };
        self.import_external_resource_with_usage_binding_optional_alias_and_physical_desc(
            name,
            usage,
            external_binding,
            None,
            Some(texture_desc),
            None,
        )
    }

    /// Imports an external buffer without assigning it a cull-root role.
    pub fn import_external_buffer_with_binding(
        &mut self,
        name: impl Into<String>,
        buffer_desc: BufferDesc,
        external_binding: RenderGraphExternalResourceBinding,
    ) -> ExternalResource {
        self.import_external_buffer_with_usage_and_binding(
            name,
            RenderGraphResourceUsageFlags::default(),
            buffer_desc,
            external_binding,
        )
    }

    /// Imports a presented external buffer whose physical shape is part of the graph contract.
    pub fn import_present_external_buffer_with_binding(
        &mut self,
        name: impl Into<String>,
        buffer_desc: BufferDesc,
        external_binding: RenderGraphExternalResourceBinding,
    ) -> ExternalResource {
        self.import_external_buffer_with_usage_and_binding(
            name,
            RenderGraphResourceUsageFlags::present(),
            buffer_desc,
            external_binding,
        )
    }

    /// Imports a physical external buffer with an explicit graph lifetime role.
    pub fn import_external_buffer_with_usage_and_binding(
        &mut self,
        name: impl Into<String>,
        usage: RenderGraphResourceUsageFlags,
        buffer_desc: BufferDesc,
        external_binding: RenderGraphExternalResourceBinding,
    ) -> ExternalResource {
        let external_binding = match external_binding.resource_type {
            RenderGraphExternalResourceType::Unknown => RenderGraphExternalResourceBinding {
                resource_type: RenderGraphExternalResourceType::Buffer,
                requirement: external_binding.requirement,
            },
            _ => external_binding,
        };
        self.import_external_resource_with_usage_binding_optional_alias_and_physical_desc(
            name,
            usage,
            external_binding,
            None,
            None,
            Some(buffer_desc),
        )
    }

    pub fn import_external_resource_with_usage(
        &mut self,
        name: impl Into<String>,
        usage: RenderGraphResourceUsageFlags,
    ) -> ExternalResource {
        self.import_external_resource_with_usage_and_binding(
            name,
            usage,
            RenderGraphExternalResourceBinding::report_only(),
        )
    }

    pub fn import_external_resource_with_usage_and_binding(
        &mut self,
        name: impl Into<String>,
        usage: RenderGraphResourceUsageFlags,
        external_binding: RenderGraphExternalResourceBinding,
    ) -> ExternalResource {
        self.import_external_resource_with_usage_binding_and_optional_alias_group(
            name,
            usage,
            external_binding,
            None,
        )
    }

    /// Imports a view of an external physical allocation. Views sharing an
    /// alias group have one conservative dependency history during compilation.
    pub fn import_external_resource_with_usage_binding_and_alias_group(
        &mut self,
        name: impl Into<String>,
        usage: RenderGraphResourceUsageFlags,
        external_binding: RenderGraphExternalResourceBinding,
        alias_group: impl Into<String>,
    ) -> ExternalResource {
        self.import_external_resource_with_usage_binding_and_optional_alias_group(
            name,
            usage,
            external_binding,
            Some(alias_group.into()),
        )
    }

    fn import_external_resource_with_usage_binding_and_optional_alias_group(
        &mut self,
        name: impl Into<String>,
        usage: RenderGraphResourceUsageFlags,
        external_binding: RenderGraphExternalResourceBinding,
        external_alias_group: Option<String>,
    ) -> ExternalResource {
        self.import_external_resource_with_usage_binding_optional_alias_and_physical_desc(
            name,
            usage,
            external_binding,
            external_alias_group,
            None,
            None,
        )
    }

    fn import_external_resource_with_usage_binding_optional_alias_and_physical_desc(
        &mut self,
        name: impl Into<String>,
        usage: RenderGraphResourceUsageFlags,
        external_binding: RenderGraphExternalResourceBinding,
        external_alias_group: Option<String>,
        external_texture_desc: Option<TextureDesc>,
        external_buffer_desc: Option<BufferDesc>,
    ) -> ExternalResource {
        let id = self.next_external_resource;
        self.next_external_resource += 1;
        let handle = ExternalResource::from_index(id, self.generation);
        self.resources.push(ResourceNode {
            resource: RenderGraphResource::External(handle),
            name: name.into(),
            desc: RenderGraphResourceDesc::External,
            external_binding,
            external_texture_desc,
            external_buffer_desc,
            external_alias_group,
            texture_view_alias: None,
            usage,
        });
        handle
    }

    pub fn mark_persistent(&mut self, texture: RgTextureHandle) -> Result<(), RenderGraphError> {
        self.mark_resource_usage(RenderGraphResource::TransientTexture(texture), |usage| {
            usage.persistent = true;
        })
    }

    pub fn mark_readback(&mut self, resource: RenderGraphResource) -> Result<(), RenderGraphError> {
        self.mark_resource_usage(resource, |usage| {
            usage.readback = true;
        })
    }

    fn validate_resource_version_token(
        &self,
        consumer_pass: RenderPassId,
        expected_resource: RenderGraphResource,
        token: RenderGraphResourceVersionToken,
    ) -> Result<(), RenderGraphError> {
        if token.builder_generation() != self.generation {
            return Err(RenderGraphError::ForeignResourceVersion {
                handle_generation: token.builder_generation(),
                builder_generation: self.generation,
            });
        }
        if token.resource() != expected_resource {
            return Err(RenderGraphError::ResourceVersionResourceMismatch {
                pass: self.passes[consumer_pass.0].name.clone(),
                expected_resource: self.resource_name(expected_resource),
                producer_resource: self.resource_name(token.resource()),
            });
        }
        let Some(producer_pass) = self.passes.get(token.producer_pass().index()) else {
            return Err(RenderGraphError::ResourceVersionProducerMissing {
                producer_pass: token.producer_pass().index(),
                producer_access: token.producer_access_index(),
            });
        };
        let Some(producer_access) = producer_pass.resources.get(token.producer_access_index())
        else {
            return Err(RenderGraphError::ResourceVersionProducerNotWrite {
                producer_pass: producer_pass.name.clone(),
                producer_access: token.producer_access_index(),
            });
        };
        if producer_access.resource != expected_resource
            || producer_access.kind != ResourceAccessKind::Write
        {
            return Err(RenderGraphError::ResourceVersionProducerNotWrite {
                producer_pass: producer_pass.name.clone(),
                producer_access: token.producer_access_index(),
            });
        }
        Ok(())
    }

    fn resource_name(&self, resource: RenderGraphResource) -> String {
        self.resources
            .iter()
            .find(|candidate| candidate.resource == resource)
            .map(|candidate| candidate.name.clone())
            .unwrap_or_else(|| format!("{resource:?}"))
    }

    fn ensure_pass(&self, id: RenderPassId) -> Result<(), RenderGraphError> {
        if id.generation() != self.generation {
            return Err(RenderGraphError::ForeignPass {
                pass: id.index(),
                handle_generation: id.generation(),
                builder_generation: self.generation,
            });
        }
        if id.0 >= self.passes.len() {
            return Err(RenderGraphError::UnknownPass { pass: id.0 });
        }
        Ok(())
    }

    fn ensure_resource(&self, resource: RenderGraphResource) -> Result<(), RenderGraphError> {
        let (kind, index, handle_generation, known) = match resource {
            RenderGraphResource::TransientTexture(handle) => (
                RenderGraphResourceKind::TransientTexture,
                handle.index(),
                handle.generation(),
                handle.index() < self.next_texture,
            ),
            RenderGraphResource::TransientBuffer(handle) => (
                RenderGraphResourceKind::TransientBuffer,
                handle.index(),
                handle.generation(),
                handle.index() < self.next_buffer,
            ),
            RenderGraphResource::External(handle) => (
                RenderGraphResourceKind::External,
                handle.index(),
                handle.generation(),
                handle.index() < self.next_external_resource,
            ),
        };
        if handle_generation != self.generation {
            return Err(RenderGraphError::ForeignResource {
                kind,
                index,
                handle_generation,
                builder_generation: self.generation,
            });
        }
        if known {
            return Ok(());
        }

        Err(RenderGraphError::UnknownResource {
            resource: format!("{resource:?}"),
        })
    }

    fn mark_resource_usage(
        &mut self,
        resource: RenderGraphResource,
        update: impl FnOnce(&mut RenderGraphResourceUsageFlags),
    ) -> Result<(), RenderGraphError> {
        self.ensure_resource(resource)?;
        let Some(node) = self
            .resources
            .iter_mut()
            .find(|node| node.resource == resource)
        else {
            return Err(RenderGraphError::UnknownResource {
                resource: format!("{resource:?}"),
            });
        };

        update(&mut node.usage);
        Ok(())
    }
}

fn next_render_graph_builder_generation() -> u64 {
    let generation = NEXT_RENDER_GRAPH_BUILDER_GENERATION.fetch_add(1, Ordering::Relaxed);
    if generation == 0 {
        NEXT_RENDER_GRAPH_BUILDER_GENERATION.fetch_add(1, Ordering::Relaxed)
    } else {
        generation
    }
}

fn resource_access_kind(access: super::types::RenderGraphResourceAccessKind) -> ResourceAccessKind {
    match access {
        super::types::RenderGraphResourceAccessKind::Read => ResourceAccessKind::Read,
        super::types::RenderGraphResourceAccessKind::Write => ResourceAccessKind::Write,
    }
}

fn texture_view_alias_desc(
    alias: &str,
    parent_name: &str,
    parent: &TextureDesc,
    range: RenderGraphTextureSubresourceRange,
) -> Result<TextureDesc, RenderGraphError> {
    let array_layers = match parent.dimension {
        TextureDimension::D2Array | TextureDimension::Cube => parent.depth,
        TextureDimension::D1 | TextureDimension::D2 | TextureDimension::D3 => 1,
    };
    let mip_level_count = range
        .mip_level_count
        .unwrap_or_else(|| parent.mip_levels.saturating_sub(range.base_mip_level));
    let array_layer_count = range
        .array_layer_count
        .unwrap_or_else(|| array_layers.saturating_sub(range.base_array_layer));
    let mip_end = range.base_mip_level.checked_add(mip_level_count);
    let array_end = range.base_array_layer.checked_add(array_layer_count);
    if mip_level_count == 0
        || array_layer_count == 0
        || mip_end.is_none_or(|end| end > parent.mip_levels)
        || array_end.is_none_or(|end| end > array_layers)
    {
        return Err(RenderGraphError::TextureViewAliasRangeOutOfBounds {
            alias: alias.to_owned(),
            parent_name: parent_name.to_owned(),
            base_mip_level: range.base_mip_level,
            mip_level_count: range.mip_level_count,
            mip_levels: parent.mip_levels,
            base_array_layer: range.base_array_layer,
            array_layer_count: range.array_layer_count,
            array_layers,
        });
    }
    let aspect_supported = match range.aspect {
        RenderGraphTextureAspect::All => true,
        RenderGraphTextureAspect::Color => !parent.format.is_depth(),
        RenderGraphTextureAspect::Depth => parent.format.is_depth(),
        RenderGraphTextureAspect::Stencil => parent.format.has_stencil(),
    };
    if !aspect_supported {
        return Err(RenderGraphError::TextureViewAliasAspectUnsupported {
            alias: alias.to_owned(),
            parent_name: parent_name.to_owned(),
            aspect: range.aspect,
            format: parent.format,
        });
    }

    let mut desc = parent.clone();
    desc.label = Some(alias.to_owned());
    desc.width = mip_extent(parent.width, range.base_mip_level);
    desc.height = mip_extent(parent.height, range.base_mip_level);
    if parent.dimension == TextureDimension::D3 {
        desc.depth = mip_extent(parent.depth, range.base_mip_level);
    } else if matches!(
        parent.dimension,
        TextureDimension::D2Array | TextureDimension::Cube
    ) {
        desc.depth = array_layer_count;
    }
    desc.mip_levels = mip_level_count;
    Ok(desc)
}

fn mip_extent(value: u32, mip_level: u32) -> u32 {
    value.checked_shr(mip_level).unwrap_or(0).max(1)
}
