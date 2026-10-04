//! 外部资源构造器把纹理/缓冲的读写、绑定强度、持久性和访问 metadata 保持为图编译契约。
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

    /// Reads a required frame-scoped external buffer with exact access metadata.
    /// The external producer remains responsible for publishing its physical descriptor.
    pub fn read_required_external_buffer_with_access(
        self,
        name: impl Into<String>,
        range: RenderGraphBufferRange,
        intent: RenderGraphResourceAccessIntent,
    ) -> Self {
        self.with_resource_contract(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Read,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::required_buffer(),
            None,
            None,
            RenderGraphResourceUsageFlags::default(),
            Some(RenderGraphResourceAccessMetadata::new(
                RenderGraphResourceAccessRange::Buffer(range),
                intent,
            )),
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
#[path = "tests/external_resources.rs"]
mod tests;
