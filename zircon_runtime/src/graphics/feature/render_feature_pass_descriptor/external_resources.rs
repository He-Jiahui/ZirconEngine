use crate::render_graph::{
    RenderGraphAttachmentOps, RenderGraphBufferRange, RenderGraphExternalResourceBinding,
    RenderGraphResourceAccessIntent, RenderGraphResourceAccessMetadata,
    RenderGraphResourceAccessRange, RenderGraphResourceUsageFlags,
    RenderGraphTextureSubresourceRange, RenderResourceSchema,
};

use super::render_feature_pass_descriptor::{
    RenderFeaturePassDescriptor, RenderFeatureResourceAccess, RenderFeatureResourceKind,
    RenderFeatureResourceWriteMode,
};

impl RenderFeaturePassDescriptor {
    pub fn read_external(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Read,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only(),
        )
    }

    pub fn read_external_from(
        self,
        name: impl Into<String>,
        producer_pass_name: impl Into<String>,
    ) -> Self {
        self.with_resource_from_producer(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Read,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only(),
            producer_pass_name,
        )
    }

    pub fn read_external_texture(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Read,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only_texture(),
        )
    }

    /// Reads a cross-frame external texture. Persistent roles are explicit
    /// culling roots and never arise from an external resource name.
    pub fn read_persistent_external_texture(self, name: impl Into<String>) -> Self {
        self.with_resource_with_input_version_and_usage(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Read,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only_texture(),
            None,
            None,
            RenderGraphResourceUsageFlags::persistent(),
        )
    }

    /// Reads a cross-frame external texture with an exact physical contract.
    pub fn read_persistent_external_texture_with_schema_and_access(
        self,
        name: impl Into<String>,
        schema: RenderResourceSchema,
        range: RenderGraphTextureSubresourceRange,
        intent: RenderGraphResourceAccessIntent,
    ) -> Self {
        self.with_resource_contract(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Read,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only_texture(),
            Some(schema),
            None,
            RenderGraphResourceUsageFlags::persistent(),
            Some(RenderGraphResourceAccessMetadata::new(
                RenderGraphResourceAccessRange::Texture(range),
                intent,
            )),
        )
    }

    /// Reads a catalog-defined cross-frame texture with exact access metadata.
    /// The pipeline resource catalog remains the single owner of its dynamic
    /// physical descriptor.
    pub fn read_persistent_external_texture_with_access(
        self,
        name: impl Into<String>,
        range: RenderGraphTextureSubresourceRange,
        intent: RenderGraphResourceAccessIntent,
    ) -> Self {
        self.with_resource_contract(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Read,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only_texture(),
            None,
            None,
            RenderGraphResourceUsageFlags::persistent(),
            Some(RenderGraphResourceAccessMetadata::new(
                RenderGraphResourceAccessRange::Texture(range),
                intent,
            )),
        )
    }

    pub fn read_external_buffer(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Read,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only_buffer(),
        )
    }

    /// Reads a frame-scoped external buffer with an exact physical contract.
    pub fn read_external_buffer_with_schema_and_access(
        self,
        name: impl Into<String>,
        schema: RenderResourceSchema,
        range: RenderGraphBufferRange,
        intent: RenderGraphResourceAccessIntent,
    ) -> Self {
        self.with_resource_contract(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Read,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only_buffer(),
            Some(schema),
            None,
            RenderGraphResourceUsageFlags::default(),
            Some(RenderGraphResourceAccessMetadata::new(
                RenderGraphResourceAccessRange::Buffer(range),
                intent,
            )),
        )
    }

    /// Reads a cross-frame external buffer such as a temporal exposure slot.
    pub fn read_persistent_external_buffer(self, name: impl Into<String>) -> Self {
        self.with_resource_with_input_version_and_usage(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Read,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only_buffer(),
            None,
            None,
            RenderGraphResourceUsageFlags::persistent(),
        )
    }

    /// Reads a cross-frame external buffer with an exact physical contract.
    pub fn read_persistent_external_buffer_with_schema_and_access(
        self,
        name: impl Into<String>,
        schema: RenderResourceSchema,
        range: RenderGraphBufferRange,
        intent: RenderGraphResourceAccessIntent,
    ) -> Self {
        self.with_resource_contract(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Read,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only_buffer(),
            Some(schema),
            None,
            RenderGraphResourceUsageFlags::persistent(),
            Some(RenderGraphResourceAccessMetadata::new(
                RenderGraphResourceAccessRange::Buffer(range),
                intent,
            )),
        )
    }

    pub fn write_external(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only(),
        )
    }

    pub fn write_external_texture(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only_texture(),
        )
    }

    pub fn write_external_buffer(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Storage,
            RenderGraphExternalResourceBinding::report_only_buffer(),
        )
    }

    /// Writes a cross-frame external buffer such as the next temporal
    /// exposure slot.
    pub fn write_persistent_external_buffer(self, name: impl Into<String>) -> Self {
        self.with_resource_with_input_version_and_usage(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Storage,
            RenderGraphExternalResourceBinding::report_only_buffer(),
            None,
            None,
            RenderGraphResourceUsageFlags::persistent(),
        )
    }

    /// Writes a cross-frame external buffer with an exact physical contract.
    pub fn write_persistent_external_buffer_with_schema_and_access(
        self,
        name: impl Into<String>,
        schema: RenderResourceSchema,
        range: RenderGraphBufferRange,
        intent: RenderGraphResourceAccessIntent,
    ) -> Self {
        self.with_resource_contract(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Storage,
            RenderGraphExternalResourceBinding::report_only_buffer(),
            Some(schema),
            None,
            RenderGraphResourceUsageFlags::persistent(),
            Some(RenderGraphResourceAccessMetadata::new(
                RenderGraphResourceAccessRange::Buffer(range),
                intent,
            )),
        )
    }

    pub fn write_storage_external(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Storage,
            RenderGraphExternalResourceBinding::report_only(),
        )
    }

    pub fn write_storage_external_texture(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Storage,
            RenderGraphExternalResourceBinding::report_only_texture(),
        )
    }

    /// Writes an externally owned storage texture whose value is extracted
    /// into cross-frame history after graph execution.
    pub fn write_persistent_storage_external_texture(self, name: impl Into<String>) -> Self {
        self.with_resource_with_input_version_and_usage(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Storage,
            RenderGraphExternalResourceBinding::report_only_texture(),
            None,
            None,
            RenderGraphResourceUsageFlags::persistent(),
        )
    }

    pub fn write_storage_external_texture_with_schema(
        self,
        name: impl Into<String>,
        schema: RenderResourceSchema,
    ) -> Self {
        self.with_resource_with_schema(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Storage,
            RenderGraphExternalResourceBinding::report_only_texture(),
            Some(schema),
        )
    }

    pub fn write_storage_external_buffer(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Storage,
            RenderGraphExternalResourceBinding::report_only_buffer(),
        )
    }

    pub fn read_required_external_buffer(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Read,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::required_buffer(),
        )
    }

    pub fn read_required_external_texture(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Read,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::required_texture(),
        )
    }

    pub fn write_required_external_buffer(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Storage,
            RenderGraphExternalResourceBinding::required_buffer(),
        )
    }

    pub fn write_required_external_texture(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::required_texture(),
        )
    }

    pub fn write_required_external_texture_with_ops(
        self,
        name: impl Into<String>,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            Some(attachment_ops),
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::required_texture(),
        )
    }

    pub fn write_required_storage_external_texture(self, name: impl Into<String>) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Storage,
            RenderGraphExternalResourceBinding::required_texture(),
        )
    }

    pub fn write_external_with_ops(
        self,
        name: impl Into<String>,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            Some(attachment_ops),
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only(),
        )
    }

    pub fn write_external_texture_with_ops(
        self,
        name: impl Into<String>,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Self {
        self.with_resource(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            Some(attachment_ops),
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only_texture(),
        )
    }

    /// Writes a terminal external texture. This is the typed culling-root
    /// declaration used by presentation paths.
    pub fn write_present_external_texture(self, name: impl Into<String>) -> Self {
        self.with_resource_with_input_version_and_usage(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only_texture(),
            None,
            None,
            RenderGraphResourceUsageFlags::present(),
        )
    }

    /// Writes a terminal external texture with an explicit attachment load/store decision.
    pub fn write_present_external_texture_with_ops(
        self,
        name: impl Into<String>,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Self {
        self.with_resource_with_input_version_and_usage(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            Some(attachment_ops),
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only_texture(),
            None,
            None,
            RenderGraphResourceUsageFlags::present(),
        )
    }

    /// Writes a cross-frame external texture such as a temporal-history slot.
    pub fn write_persistent_external_texture(self, name: impl Into<String>) -> Self {
        self.with_resource_with_input_version_and_usage(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only_texture(),
            None,
            None,
            RenderGraphResourceUsageFlags::persistent(),
        )
    }

    /// Writes a cross-frame external attachment with an exact physical contract.
    pub fn write_persistent_external_texture_with_schema_and_access(
        self,
        name: impl Into<String>,
        schema: RenderResourceSchema,
        range: RenderGraphTextureSubresourceRange,
        intent: RenderGraphResourceAccessIntent,
    ) -> Self {
        self.with_resource_contract(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only_texture(),
            Some(schema),
            None,
            RenderGraphResourceUsageFlags::persistent(),
            Some(RenderGraphResourceAccessMetadata::new(
                RenderGraphResourceAccessRange::Texture(range),
                intent,
            )),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::graphics::feature::{
        RenderFeaturePassDescriptor, RenderFeatureResourceAccess, RenderFeatureResourceWriteMode,
    };
    use crate::graphics::pipeline::RenderPassStage;
    use crate::render_graph::{
        QueueLane, RenderBufferSchema, RenderGraphBufferRange, RenderGraphExternalResourceBinding,
        RenderGraphResourceAccessIntent, RenderGraphResourceAccessMetadata,
        RenderGraphResourceAccessRange, RenderGraphShaderStages,
        RenderGraphTextureSubresourceRange, RenderResourceSchema, RenderTextureSchema,
    };
    use crate::rhi::{BufferUsage, TextureFormat, TextureUsage};

    #[test]
    fn frame_external_buffer_access_retains_schema_and_exact_uniform_intent() {
        let schema = RenderResourceSchema::buffer(RenderBufferSchema::new(
            32,
            BufferUsage::UNIFORM | BufferUsage::COPY_DST,
        ));
        let range = RenderGraphBufferRange::full();
        let intent = RenderGraphResourceAccessIntent::UniformBuffer {
            stages: RenderGraphShaderStages::COMPUTE,
        };
        let pass = RenderFeaturePassDescriptor::new(
            RenderPassStage::AmbientOcclusion,
            "ssao-exact-params",
            QueueLane::AsyncCompute,
        )
        .read_external_buffer_with_schema_and_access("ssao.params", schema, range, intent);

        assert_eq!(pass.resources.len(), 1);
        let resource = &pass.resources[0];
        assert_eq!(resource.access, RenderFeatureResourceAccess::Read);
        assert_eq!(resource.schema, Some(schema));
        assert!(!resource.usage.persistent);
        assert_eq!(
            resource.external_binding,
            RenderGraphExternalResourceBinding::report_only_buffer()
        );
        assert_eq!(
            resource.access_metadata,
            Some(RenderGraphResourceAccessMetadata::new(
                RenderGraphResourceAccessRange::Buffer(range),
                intent,
            ))
        );
    }

    #[test]
    fn persistent_external_buffer_accesses_retain_typed_buffer_bindings() {
        let schema = RenderResourceSchema::buffer(RenderBufferSchema::new(
            16,
            BufferUsage::STORAGE | BufferUsage::COPY_SRC | BufferUsage::COPY_DST,
        ));
        let range = RenderGraphBufferRange::full();
        let read_intent =
            RenderGraphResourceAccessIntent::storage_buffer_read(RenderGraphShaderStages::COMPUTE);
        let write_intent = RenderGraphResourceAccessIntent::storage_buffer_read_write(
            RenderGraphShaderStages::COMPUTE,
        );
        let pass = RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "persistent-exposure-history",
            QueueLane::AsyncCompute,
        )
        .read_persistent_external_buffer_with_schema_and_access(
            "exposure.previous",
            schema,
            range,
            read_intent,
        )
        .write_persistent_external_buffer_with_schema_and_access(
            "exposure.current",
            schema,
            range,
            write_intent,
        );

        assert_eq!(pass.resources.len(), 2);
        assert!(
            pass.resources
                .iter()
                .all(|resource| resource.usage.persistent)
        );
        assert!(pass.resources.iter().all(|resource| {
            resource.external_binding == RenderGraphExternalResourceBinding::report_only_buffer()
        }));
        assert_eq!(pass.resources[0].access, RenderFeatureResourceAccess::Read);
        assert_eq!(pass.resources[1].access, RenderFeatureResourceAccess::Write);
        assert!(
            pass.resources
                .iter()
                .all(|resource| resource.schema == Some(schema))
        );
        assert_eq!(
            pass.resources[0].access_metadata,
            Some(RenderGraphResourceAccessMetadata::new(
                RenderGraphResourceAccessRange::Buffer(range),
                read_intent,
            ))
        );
        assert_eq!(
            pass.resources[1].access_metadata,
            Some(RenderGraphResourceAccessMetadata::new(
                RenderGraphResourceAccessRange::Buffer(range),
                write_intent,
            ))
        );
        assert_eq!(
            pass.resources[1].write_mode,
            RenderFeatureResourceWriteMode::Storage
        );
    }

    #[test]
    fn persistent_external_texture_accesses_retain_typed_view_bindings() {
        let schema = RenderResourceSchema::texture(RenderTextureSchema::new(
            TextureFormat::Rgba16Float,
            TextureUsage::SAMPLED | TextureUsage::RENDER_ATTACHMENT,
        ));
        let range = RenderGraphTextureSubresourceRange::full();
        let read_intent =
            RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::FRAGMENT);
        let write_intent = RenderGraphResourceAccessIntent::ColorAttachment;
        let pass = RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "persistent-texture-history",
            QueueLane::Graphics,
        )
        .read_persistent_external_texture_with_schema_and_access(
            "history.previous",
            schema,
            range,
            read_intent,
        )
        .write_persistent_external_texture_with_schema_and_access(
            "history.current",
            schema,
            range,
            write_intent,
        );

        assert_eq!(pass.resources.len(), 2);
        assert!(pass.resources.iter().all(|resource| {
            resource.usage.persistent
                && resource.external_binding
                    == RenderGraphExternalResourceBinding::report_only_texture()
                && resource.schema == Some(schema)
        }));
        assert_eq!(
            pass.resources[0].access_metadata,
            Some(RenderGraphResourceAccessMetadata::new(
                RenderGraphResourceAccessRange::Texture(range),
                read_intent,
            ))
        );
        assert_eq!(
            pass.resources[1].access_metadata,
            Some(RenderGraphResourceAccessMetadata::new(
                RenderGraphResourceAccessRange::Texture(range),
                write_intent,
            ))
        );
        assert_eq!(
            pass.resources[1].write_mode,
            RenderFeatureResourceWriteMode::Attachment
        );
    }

    #[test]
    fn catalog_backed_persistent_external_texture_retains_exact_access_without_schema() {
        let range = RenderGraphTextureSubresourceRange::full();
        let intent =
            RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::COMPUTE);
        let pass = RenderFeaturePassDescriptor::new(
            RenderPassStage::DepthPrepass,
            "catalog-backed-history",
            QueueLane::AsyncCompute,
        )
        .read_persistent_external_texture_with_access(
            "history.previous.hzb",
            range,
            intent,
        );

        assert_eq!(pass.resources.len(), 1);
        let resource = &pass.resources[0];
        assert_eq!(resource.access, RenderFeatureResourceAccess::Read);
        assert!(resource.usage.persistent);
        assert_eq!(
            resource.external_binding,
            RenderGraphExternalResourceBinding::report_only_texture()
        );
        assert_eq!(resource.schema, None);
        assert_eq!(
            resource.access_metadata,
            Some(RenderGraphResourceAccessMetadata::new(
                RenderGraphResourceAccessRange::Texture(range),
                intent,
            ))
        );
    }
}
