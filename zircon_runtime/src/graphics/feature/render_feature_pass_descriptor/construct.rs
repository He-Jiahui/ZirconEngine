use crate::core::framework::render::{ShaderResourceAccess, ShaderResourceKind};
use crate::graphics::scene::RenderPassExecutorId;
use crate::graphics::shader::invocation::{
    ComputeDispatchPlan, FullscreenPassPlan, ShaderNamedResourceBinding,
};
use crate::render_graph::{
    QueueLane, RenderGraphAttachmentOps, RenderGraphComputeWorkload,
    RenderGraphExternalResourceBinding, RenderGraphResourceAccessMetadata,
    RenderGraphResourceUsageFlags, RenderGraphTextureSubresourceRange, RenderResourceSchema,
};

use crate::graphics::pipeline::RenderPassStage;

use super::super::compute_pass_descriptor::ComputePassDescriptor;

use super::render_feature_pass_descriptor::{
    RenderFeaturePassDescriptor, RenderFeatureResourceAccess, RenderFeatureResourceDescriptor,
    RenderFeatureResourceKind, RenderFeatureResourceVersion, RenderFeatureResourceWriteMode,
    RenderFeatureTextureViewAlias,
};

impl RenderFeaturePassDescriptor {
    pub fn new(stage: RenderPassStage, pass_name: impl Into<String>, queue: QueueLane) -> Self {
        let pass_name = pass_name.into();
        Self {
            stage,
            executor_id: RenderPassExecutorId::new(pass_name.clone()),
            pass_name,
            queue,
            flags: Default::default(),
            compute_workload: None,
            compute_pass: None,
            resources: Vec::new(),
        }
    }

    pub fn with_executor_id(mut self, executor_id: impl Into<RenderPassExecutorId>) -> Self {
        self.executor_id = executor_id.into();
        self
    }

    pub fn with_side_effects(mut self) -> Self {
        self.flags.has_side_effects = true;
        self
    }

    pub fn with_compute_workload(mut self, workload: RenderGraphComputeWorkload) -> Self {
        self.compute_workload = Some(workload);
        self.compute_pass = None;
        self
    }

    pub fn with_compute_pass(self, compute_pass: ComputePassDescriptor) -> Self {
        compute_pass.lower_into(self)
    }

    pub fn with_compute_dispatch_plan(mut self, plan: &ComputeDispatchPlan) -> Self {
        self.compute_workload = Some(RenderGraphComputeWorkload::from_shader_dispatch(plan));
        self.compute_pass = None;
        self.push_shader_resource_bindings(&plan.resources);
        self
    }

    pub fn with_fullscreen_pass_plan(mut self, plan: &FullscreenPassPlan) -> Self {
        self.push_shader_resource_bindings(&plan.resources);
        self
    }

    pub fn read_texture(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::Texture,
            RenderFeatureResourceAccess::Read,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only(),
        )
    }

    /// Declares a texture read with the producer-owned physical contract.
    pub fn read_texture_with_schema(
        self,
        name: impl Into<String>,
        schema: RenderResourceSchema,
    ) -> Self {
        self.with_resource_with_schema(
            name,
            RenderFeatureResourceKind::Texture,
            RenderFeatureResourceAccess::Read,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only(),
            Some(schema),
        )
    }

    pub fn read_texture_from(
        self,
        name: impl Into<String>,
        producer_pass_name: impl Into<String>,
    ) -> Self {
        self.with_resource_from_producer(
            name,
            RenderFeatureResourceKind::Texture,
            RenderFeatureResourceAccess::Read,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only(),
            producer_pass_name,
        )
    }

    /// Reads a specific producer value while retaining its allocation contract.
    pub fn read_texture_from_with_schema(
        self,
        name: impl Into<String>,
        producer_pass_name: impl Into<String>,
        schema: RenderResourceSchema,
    ) -> Self {
        let name = name.into();
        self.with_resource_with_input_version(
            name.clone(),
            RenderFeatureResourceKind::Texture,
            RenderFeatureResourceAccess::Read,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only(),
            Some(schema),
            Some(RenderFeatureResourceVersion::new(
                name,
                RenderFeatureResourceKind::Texture,
                producer_pass_name,
            )),
        )
    }

    pub fn write_texture(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::Texture,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only(),
        )
    }

    /// Writes a graph-owned texture retained until its cross-frame extraction
    /// has completed. This prevents same-frame aliasing and makes the source
    /// an explicit culling root without introducing a synthetic copy pass.
    pub fn write_persistent_texture(self, name: impl Into<String>) -> Self {
        self.with_resource_with_input_version_and_usage(
            name,
            RenderFeatureResourceKind::Texture,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only(),
            None,
            None,
            RenderGraphResourceUsageFlags::persistent(),
        )
    }

    /// Writes a retained graph-owned texture with an explicit attachment
    /// initialization decision.
    pub fn write_persistent_texture_with_ops(
        self,
        name: impl Into<String>,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Self {
        self.with_resource_with_input_version_and_usage(
            name,
            RenderFeatureResourceKind::Texture,
            RenderFeatureResourceAccess::Write,
            Some(attachment_ops),
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only(),
            None,
            None,
            RenderGraphResourceUsageFlags::persistent(),
        )
    }

    /// Declares an attachment write with an explicit physical contract.
    pub fn write_texture_with_schema(
        self,
        name: impl Into<String>,
        schema: RenderResourceSchema,
    ) -> Self {
        self.with_resource_with_schema(
            name,
            RenderFeatureResourceKind::Texture,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only(),
            Some(schema),
        )
    }

    /// Writes an attachment by loading the value produced by `producer_pass_name`.
    pub fn write_texture_load_from(
        self,
        name: impl Into<String>,
        producer_pass_name: impl Into<String>,
    ) -> Self {
        self.with_resource_from_producer(
            name,
            RenderFeatureResourceKind::Texture,
            RenderFeatureResourceAccess::Write,
            Some(RenderGraphAttachmentOps::load_store()),
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only(),
            producer_pass_name,
        )
    }

    /// Loads a producer-owned attachment value with its explicit contract.
    pub fn write_texture_load_from_with_schema(
        self,
        name: impl Into<String>,
        producer_pass_name: impl Into<String>,
        schema: RenderResourceSchema,
    ) -> Self {
        let name = name.into();
        self.with_resource_with_input_version(
            name.clone(),
            RenderFeatureResourceKind::Texture,
            RenderFeatureResourceAccess::Write,
            Some(RenderGraphAttachmentOps::load_store()),
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only(),
            Some(schema),
            Some(RenderFeatureResourceVersion::new(
                name,
                RenderFeatureResourceKind::Texture,
                producer_pass_name,
            )),
        )
    }

    pub fn write_storage_texture(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::Texture,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Storage,
            RenderGraphExternalResourceBinding::report_only(),
        )
    }

    /// Writes a graph-owned storage texture retained until its cross-frame
    /// extraction has completed.
    pub fn write_persistent_storage_texture(self, name: impl Into<String>) -> Self {
        self.with_resource_with_input_version_and_usage(
            name,
            RenderFeatureResourceKind::Texture,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Storage,
            RenderGraphExternalResourceBinding::report_only(),
            None,
            None,
            RenderGraphResourceUsageFlags::persistent(),
        )
    }

    pub fn write_storage_texture_with_schema(
        self,
        name: impl Into<String>,
        schema: RenderResourceSchema,
    ) -> Self {
        self.with_resource_with_schema(
            name,
            RenderFeatureResourceKind::Texture,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Storage,
            RenderGraphExternalResourceBinding::report_only(),
            Some(schema),
        )
    }

    pub fn write_texture_with_ops(
        self,
        name: impl Into<String>,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::Texture,
            RenderFeatureResourceAccess::Write,
            Some(attachment_ops),
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only(),
        )
    }

    /// Declares an attachment write and its physical contract atomically.
    pub fn write_texture_with_ops_and_schema(
        self,
        name: impl Into<String>,
        attachment_ops: RenderGraphAttachmentOps,
        schema: RenderResourceSchema,
    ) -> Self {
        self.with_resource_with_schema(
            name,
            RenderFeatureResourceKind::Texture,
            RenderFeatureResourceAccess::Write,
            Some(attachment_ops),
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only(),
            Some(schema),
        )
    }

    /// Writes a logical texture resource backed by an exact view of `parent`.
    /// The graph compiler owns parent-name resolution and range validation.
    pub fn write_texture_view_alias_with_ops(
        mut self,
        name: impl Into<String>,
        parent: impl Into<String>,
        range: RenderGraphTextureSubresourceRange,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Self {
        self.resources.push(RenderFeatureResourceDescriptor {
            name: name.into(),
            kind: RenderFeatureResourceKind::Texture,
            access: RenderFeatureResourceAccess::Write,
            input_version: None,
            minimum_size_bytes: None,
            attachment_ops: Some(attachment_ops),
            write_mode: RenderFeatureResourceWriteMode::Attachment,
            access_metadata: None,
            external_binding: RenderGraphExternalResourceBinding::report_only(),
            texture_view_alias: Some(RenderFeatureTextureViewAlias::new(parent, range)),
            schema: None,
            usage: RenderGraphResourceUsageFlags::default(),
        });
        self
    }

    pub fn read_buffer(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::Buffer,
            RenderFeatureResourceAccess::Read,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only(),
        )
    }

    /// Declares a buffer read with the producer-owned physical contract.
    pub fn read_buffer_with_schema(
        self,
        name: impl Into<String>,
        schema: RenderResourceSchema,
    ) -> Self {
        self.with_resource_with_schema(
            name,
            RenderFeatureResourceKind::Buffer,
            RenderFeatureResourceAccess::Read,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only(),
            Some(schema),
        )
    }

    pub fn read_buffer_from(
        self,
        name: impl Into<String>,
        producer_pass_name: impl Into<String>,
    ) -> Self {
        self.with_resource_from_producer(
            name,
            RenderFeatureResourceKind::Buffer,
            RenderFeatureResourceAccess::Read,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only(),
            producer_pass_name,
        )
    }

    /// Reads a specific buffer producer value with its explicit contract.
    pub fn read_buffer_from_with_schema(
        self,
        name: impl Into<String>,
        producer_pass_name: impl Into<String>,
        schema: RenderResourceSchema,
    ) -> Self {
        let name = name.into();
        self.with_resource_with_input_version(
            name.clone(),
            RenderFeatureResourceKind::Buffer,
            RenderFeatureResourceAccess::Read,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only(),
            Some(schema),
            Some(RenderFeatureResourceVersion::new(
                name,
                RenderFeatureResourceKind::Buffer,
                producer_pass_name,
            )),
        )
    }

    pub fn write_buffer(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::Buffer,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Storage,
            RenderGraphExternalResourceBinding::report_only(),
        )
    }

    pub fn write_buffer_with_schema(
        self,
        name: impl Into<String>,
        schema: RenderResourceSchema,
    ) -> Self {
        self.with_resource_with_schema(
            name,
            RenderFeatureResourceKind::Buffer,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Storage,
            RenderGraphExternalResourceBinding::report_only(),
            Some(schema),
        )
    }

    pub fn write_buffer_with_minimum_size(
        self,
        name: impl Into<String>,
        minimum_size_bytes: u64,
    ) -> Self {
        self.with_resource_minimum_size(
            name,
            RenderFeatureResourceKind::Buffer,
            RenderFeatureResourceAccess::Write,
            RenderFeatureResourceWriteMode::Storage,
            minimum_size_bytes,
        )
    }

    pub(super) fn with_resource(
        self,
        name: impl Into<String>,
        kind: RenderFeatureResourceKind,
        access: RenderFeatureResourceAccess,
        attachment_ops: Option<RenderGraphAttachmentOps>,
        write_mode: RenderFeatureResourceWriteMode,
        external_binding: RenderGraphExternalResourceBinding,
    ) -> Self {
        self.with_resource_with_schema(
            name,
            kind,
            access,
            attachment_ops,
            write_mode,
            external_binding,
            None,
        )
    }

    pub(super) fn with_resource_with_schema(
        self,
        name: impl Into<String>,
        kind: RenderFeatureResourceKind,
        access: RenderFeatureResourceAccess,
        attachment_ops: Option<RenderGraphAttachmentOps>,
        write_mode: RenderFeatureResourceWriteMode,
        external_binding: RenderGraphExternalResourceBinding,
        schema: Option<RenderResourceSchema>,
    ) -> Self {
        self.with_resource_with_input_version(
            name,
            kind,
            access,
            attachment_ops,
            write_mode,
            external_binding,
            schema,
            None,
        )
    }

    pub(super) fn with_resource_from_producer(
        self,
        name: impl Into<String>,
        kind: RenderFeatureResourceKind,
        access: RenderFeatureResourceAccess,
        attachment_ops: Option<RenderGraphAttachmentOps>,
        write_mode: RenderFeatureResourceWriteMode,
        external_binding: RenderGraphExternalResourceBinding,
        producer_pass_name: impl Into<String>,
    ) -> Self {
        let name = name.into();
        self.with_resource_with_input_version(
            name.clone(),
            kind,
            access,
            attachment_ops,
            write_mode,
            external_binding,
            None,
            Some(RenderFeatureResourceVersion::new(
                name,
                kind,
                producer_pass_name,
            )),
        )
    }

    fn with_resource_with_input_version(
        mut self,
        name: impl Into<String>,
        kind: RenderFeatureResourceKind,
        access: RenderFeatureResourceAccess,
        attachment_ops: Option<RenderGraphAttachmentOps>,
        write_mode: RenderFeatureResourceWriteMode,
        external_binding: RenderGraphExternalResourceBinding,
        schema: Option<RenderResourceSchema>,
        input_version: Option<RenderFeatureResourceVersion>,
    ) -> Self {
        self.with_resource_with_input_version_and_usage(
            name,
            kind,
            access,
            attachment_ops,
            write_mode,
            external_binding,
            schema,
            input_version,
            RenderGraphResourceUsageFlags::default(),
        )
    }

    pub(super) fn with_resource_with_input_version_and_usage(
        self,
        name: impl Into<String>,
        kind: RenderFeatureResourceKind,
        access: RenderFeatureResourceAccess,
        attachment_ops: Option<RenderGraphAttachmentOps>,
        write_mode: RenderFeatureResourceWriteMode,
        external_binding: RenderGraphExternalResourceBinding,
        schema: Option<RenderResourceSchema>,
        input_version: Option<RenderFeatureResourceVersion>,
        usage: RenderGraphResourceUsageFlags,
    ) -> Self {
        self.with_resource_contract(
            name,
            kind,
            access,
            attachment_ops,
            write_mode,
            external_binding,
            schema,
            input_version,
            usage,
            None,
        )
    }

    pub(super) fn with_resource_contract(
        mut self,
        name: impl Into<String>,
        kind: RenderFeatureResourceKind,
        access: RenderFeatureResourceAccess,
        attachment_ops: Option<RenderGraphAttachmentOps>,
        write_mode: RenderFeatureResourceWriteMode,
        external_binding: RenderGraphExternalResourceBinding,
        schema: Option<RenderResourceSchema>,
        input_version: Option<RenderFeatureResourceVersion>,
        usage: RenderGraphResourceUsageFlags,
        access_metadata: Option<RenderGraphResourceAccessMetadata>,
    ) -> Self {
        self.resources.push(RenderFeatureResourceDescriptor {
            name: name.into(),
            kind,
            access,
            input_version,
            minimum_size_bytes: None,
            attachment_ops,
            write_mode,
            access_metadata,
            external_binding,
            texture_view_alias: None,
            schema,
            usage,
        });
        self
    }

    fn with_resource_minimum_size(
        mut self,
        name: impl Into<String>,
        kind: RenderFeatureResourceKind,
        access: RenderFeatureResourceAccess,
        write_mode: RenderFeatureResourceWriteMode,
        minimum_size_bytes: u64,
    ) -> Self {
        self.resources.push(RenderFeatureResourceDescriptor {
            name: name.into(),
            kind,
            access,
            input_version: None,
            minimum_size_bytes: Some(minimum_size_bytes),
            attachment_ops: None,
            write_mode,
            access_metadata: None,
            external_binding: RenderGraphExternalResourceBinding::report_only(),
            texture_view_alias: None,
            schema: None,
            usage: RenderGraphResourceUsageFlags::default(),
        });
        self
    }

    fn push_shader_resource_bindings(&mut self, bindings: &[ShaderNamedResourceBinding]) {
        self.resources.extend(
            bindings
                .iter()
                .filter_map(render_feature_resource_for_shader_binding),
        );
    }
}

fn render_feature_resource_for_shader_binding(
    binding: &ShaderNamedResourceBinding,
) -> Option<RenderFeatureResourceDescriptor> {
    let kind = match binding.kind {
        ShaderResourceKind::UniformBuffer | ShaderResourceKind::StorageBuffer => {
            RenderFeatureResourceKind::Buffer
        }
        ShaderResourceKind::Texture | ShaderResourceKind::StorageTexture => {
            RenderFeatureResourceKind::Texture
        }
        ShaderResourceKind::Sampler => return None,
    };
    let access = match binding.access {
        ShaderResourceAccess::Read => RenderFeatureResourceAccess::Read,
        ShaderResourceAccess::ReadWrite | ShaderResourceAccess::Write => {
            RenderFeatureResourceAccess::Write
        }
    };
    let write_mode = if matches!(access, RenderFeatureResourceAccess::Write)
        || matches!(
            binding.kind,
            ShaderResourceKind::StorageBuffer | ShaderResourceKind::StorageTexture
        ) {
        RenderFeatureResourceWriteMode::Storage
    } else {
        RenderFeatureResourceWriteMode::Attachment
    };

    Some(RenderFeatureResourceDescriptor {
        name: binding.name.clone(),
        kind,
        access,
        input_version: None,
        minimum_size_bytes: None,
        attachment_ops: None,
        write_mode,
        access_metadata: None,
        external_binding: RenderGraphExternalResourceBinding::report_only(),
        texture_view_alias: None,
        schema: None,
        usage: RenderGraphResourceUsageFlags::default(),
    })
}

#[cfg(test)]
#[path = "tests/construct.rs"]
mod tests;
