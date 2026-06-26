use crate::rhi::{BufferDesc, TextureDesc};

use super::error::RenderGraphError;
use super::types::{
    ExternalResource, PassFlags, QueueLane, RenderGraphAttachmentOps, RenderGraphComputeWorkload,
    RenderGraphExternalResourceBinding, RenderGraphResource, RenderGraphResourceDesc,
    RenderGraphResourceUsageFlags, RenderPassId, RgBufferHandle, RgTextureHandle,
};

mod compile;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ResourceAccessKind {
    Read,
    Write,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ResourceAccess {
    resource: RenderGraphResource,
    kind: ResourceAccessKind,
    attachment_ops: Option<RenderGraphAttachmentOps>,
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
    dependencies: Vec<RenderPassId>,
    resources: Vec<ResourceAccess>,
}

#[derive(Clone, Debug)]
struct ResourceNode {
    resource: RenderGraphResource,
    name: String,
    desc: RenderGraphResourceDesc,
    external_binding: RenderGraphExternalResourceBinding,
    usage: RenderGraphResourceUsageFlags,
}

#[derive(Clone, Debug)]
pub struct RenderGraphBuilder {
    name: String,
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
        let id = RenderPassId(self.passes.len());
        self.passes.push(RenderPassNode {
            id,
            name: name.into(),
            declared_queue,
            queue,
            flags: PassFlags::default(),
            executor_id: executor_id.map(Into::into),
            compute_workload: None,
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
        let handle = RgTextureHandle::from_index(id);
        let name = desc
            .label
            .clone()
            .unwrap_or_else(|| format!("rg-texture-{id}"));
        self.resources.push(ResourceNode {
            resource: RenderGraphResource::TransientTexture(handle),
            name,
            desc: RenderGraphResourceDesc::Texture(desc),
            external_binding: RenderGraphExternalResourceBinding::report_only(),
            usage: RenderGraphResourceUsageFlags::default(),
        });
        handle
    }

    pub fn create_buffer(&mut self, desc: BufferDesc) -> RgBufferHandle {
        let id = self.next_buffer;
        self.next_buffer += 1;
        let handle = RgBufferHandle::from_index(id);
        let name = desc
            .label
            .clone()
            .unwrap_or_else(|| format!("rg-buffer-{id}"));
        self.resources.push(ResourceNode {
            resource: RenderGraphResource::TransientBuffer(handle),
            name,
            desc: RenderGraphResourceDesc::Buffer(desc),
            external_binding: RenderGraphExternalResourceBinding::report_only(),
            usage: RenderGraphResourceUsageFlags::default(),
        });
        handle
    }

    pub fn import_external_resource(&mut self, name: impl Into<String>) -> ExternalResource {
        self.import_external_resource_with_usage(name, RenderGraphResourceUsageFlags::present())
    }

    pub fn import_external_resource_with_binding(
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
        let id = self.next_external_resource;
        self.next_external_resource += 1;
        let handle = ExternalResource::from_index(id);
        self.resources.push(ResourceNode {
            resource: RenderGraphResource::External(handle),
            name: name.into(),
            desc: RenderGraphResourceDesc::External,
            external_binding,
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

    pub fn read_texture(
        &mut self,
        pass: RenderPassId,
        texture: RgTextureHandle,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access(
            pass,
            RenderGraphResource::TransientTexture(texture),
            ResourceAccessKind::Read,
            None,
        )
    }

    pub fn write_texture(
        &mut self,
        pass: RenderPassId,
        texture: RgTextureHandle,
    ) -> Result<(), RenderGraphError> {
        self.write_texture_with_ops(pass, texture, RenderGraphAttachmentOps::clear_store())
    }

    pub fn write_storage_texture(
        &mut self,
        pass: RenderPassId,
        texture: RgTextureHandle,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access(
            pass,
            RenderGraphResource::TransientTexture(texture),
            ResourceAccessKind::Write,
            None,
        )
    }

    pub fn write_texture_with_ops(
        &mut self,
        pass: RenderPassId,
        texture: RgTextureHandle,
        ops: RenderGraphAttachmentOps,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access(
            pass,
            RenderGraphResource::TransientTexture(texture),
            ResourceAccessKind::Write,
            Some(ops),
        )
    }

    pub fn read_buffer(
        &mut self,
        pass: RenderPassId,
        buffer: RgBufferHandle,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access(
            pass,
            RenderGraphResource::TransientBuffer(buffer),
            ResourceAccessKind::Read,
            None,
        )
    }

    pub fn write_buffer(
        &mut self,
        pass: RenderPassId,
        buffer: RgBufferHandle,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access(
            pass,
            RenderGraphResource::TransientBuffer(buffer),
            ResourceAccessKind::Write,
            None,
        )
    }

    pub fn read_external(
        &mut self,
        pass: RenderPassId,
        external: ExternalResource,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access(
            pass,
            RenderGraphResource::External(external),
            ResourceAccessKind::Read,
            None,
        )
    }

    pub fn write_external(
        &mut self,
        pass: RenderPassId,
        external: ExternalResource,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access(
            pass,
            RenderGraphResource::External(external),
            ResourceAccessKind::Write,
            Some(RenderGraphAttachmentOps::load_store()),
        )
    }

    pub fn write_storage_external(
        &mut self,
        pass: RenderPassId,
        external: ExternalResource,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access(
            pass,
            RenderGraphResource::External(external),
            ResourceAccessKind::Write,
            None,
        )
    }

    pub fn write_external_with_ops(
        &mut self,
        pass: RenderPassId,
        external: ExternalResource,
        ops: RenderGraphAttachmentOps,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access(
            pass,
            RenderGraphResource::External(external),
            ResourceAccessKind::Write,
            Some(ops),
        )
    }

    fn add_resource_access(
        &mut self,
        pass: RenderPassId,
        resource: RenderGraphResource,
        kind: ResourceAccessKind,
        attachment_ops: Option<RenderGraphAttachmentOps>,
    ) -> Result<(), RenderGraphError> {
        self.ensure_pass(pass)?;
        self.ensure_resource(resource)?;
        self.passes[pass.0].resources.push(ResourceAccess {
            resource,
            kind,
            attachment_ops,
        });
        Ok(())
    }

    fn ensure_pass(&self, id: RenderPassId) -> Result<(), RenderGraphError> {
        if id.0 >= self.passes.len() {
            return Err(RenderGraphError::UnknownPass { pass: id.0 });
        }
        Ok(())
    }

    fn ensure_resource(&self, resource: RenderGraphResource) -> Result<(), RenderGraphError> {
        if self.resources.iter().any(|node| node.resource == resource) {
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
