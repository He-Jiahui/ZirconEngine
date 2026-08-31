use crate::render_graph::{
    RenderGraphAttachmentOps, RenderGraphExternalResourceBinding, RenderGraphResourceUsageFlags,
    RenderResourceSchema,
};

use super::render_feature_pass_descriptor::{
    RenderFeaturePassDescriptor, RenderFeatureResourceAccess, RenderFeatureResourceKind,
    RenderFeatureResourceWriteMode,
};

impl RenderFeaturePassDescriptor {
    /// Writes a terminal external texture whose physical shape is part of the
    /// compiled graph contract.
    pub fn write_present_external_texture_with_schema(
        self,
        name: impl Into<String>,
        schema: RenderResourceSchema,
    ) -> Self {
        self.with_resource_with_input_version_and_usage(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            None,
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only_texture(),
            Some(schema),
            None,
            RenderGraphResourceUsageFlags::present(),
        )
    }

    /// Writes a terminal external texture with explicit attachment operations
    /// and a physical graph contract.
    pub fn write_present_external_texture_with_ops_and_schema(
        self,
        name: impl Into<String>,
        attachment_ops: RenderGraphAttachmentOps,
        schema: RenderResourceSchema,
    ) -> Self {
        self.with_resource_with_input_version_and_usage(
            name,
            RenderFeatureResourceKind::External,
            RenderFeatureResourceAccess::Write,
            Some(attachment_ops),
            RenderFeatureResourceWriteMode::Attachment,
            RenderGraphExternalResourceBinding::report_only_texture(),
            Some(schema),
            None,
            RenderGraphResourceUsageFlags::present(),
        )
    }
}
