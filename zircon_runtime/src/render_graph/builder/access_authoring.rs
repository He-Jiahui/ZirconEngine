use super::super::access::{
    RenderGraphBufferRange, RenderGraphResourceAccessIntent, RenderGraphResourceAccessMetadata,
    RenderGraphResourceAccessRange, RenderGraphTextureSubresourceRange,
};
use super::super::error::RenderGraphError;
use super::super::types::{
    ExternalResource, RenderGraphAttachmentLoadOp, RenderGraphAttachmentOps, RenderGraphResource,
    RenderGraphResourceAccessKind, RenderGraphResourceKind, RenderGraphResourceVersionToken,
    RenderPassId, RgBufferHandle, RgTextureHandle,
};
use super::{resource_access_kind, RenderGraphBuilder, ResourceAccess, ResourceAccessKind};

impl RenderGraphBuilder {
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

    /// Declares a texture read with the exact logical view and backend-neutral use intent.
    pub fn read_texture_with_access(
        &mut self,
        pass: RenderPassId,
        texture: RgTextureHandle,
        range: RenderGraphTextureSubresourceRange,
        intent: RenderGraphResourceAccessIntent,
    ) -> Result<(), RenderGraphError> {
        self.access_texture(
            pass,
            texture,
            RenderGraphResourceAccessKind::Read,
            range,
            intent,
            None,
        )
    }

    /// Declares a texture read or write with exact logical scope and use intent.
    pub fn access_texture(
        &mut self,
        pass: RenderPassId,
        texture: RgTextureHandle,
        access: RenderGraphResourceAccessKind,
        range: RenderGraphTextureSubresourceRange,
        intent: RenderGraphResourceAccessIntent,
        attachment_ops: Option<RenderGraphAttachmentOps>,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access_with_metadata(
            pass,
            RenderGraphResource::TransientTexture(texture),
            resource_access_kind(access),
            attachment_ops,
            RenderGraphResourceAccessMetadata::new(
                RenderGraphResourceAccessRange::Texture(range),
                intent,
            ),
        )
    }

    pub fn read_texture_from_version(
        &mut self,
        pass: RenderPassId,
        version: RenderGraphResourceVersionToken,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access_from_version(
            pass,
            RenderGraphResourceKind::TransientTexture,
            ResourceAccessKind::Read,
            None,
            version,
        )
    }

    /// Declares an exact texture scope consumed from one exact producer access.
    pub fn read_texture_with_access_from_version(
        &mut self,
        pass: RenderPassId,
        version: RenderGraphResourceVersionToken,
        range: RenderGraphTextureSubresourceRange,
        intent: RenderGraphResourceAccessIntent,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access_from_version_with_metadata(
            pass,
            RenderGraphResourceKind::TransientTexture,
            ResourceAccessKind::Read,
            None,
            version,
            RenderGraphResourceAccessMetadata::new(
                RenderGraphResourceAccessRange::Texture(range),
                intent,
            ),
        )
    }

    pub fn write_texture(
        &mut self,
        pass: RenderPassId,
        texture: RgTextureHandle,
    ) -> Result<(), RenderGraphError> {
        self.write_texture_with_ops(pass, texture, RenderGraphAttachmentOps::clear_store())
    }

    pub fn write_texture_versioned(
        &mut self,
        pass: RenderPassId,
        texture: RgTextureHandle,
    ) -> Result<RenderGraphResourceVersionToken, RenderGraphError> {
        self.write_texture_with_ops_versioned(
            pass,
            texture,
            RenderGraphAttachmentOps::clear_store(),
        )
    }

    /// Declares an exact texture write and returns its stable producer token.
    pub fn write_texture_with_access_versioned(
        &mut self,
        pass: RenderPassId,
        texture: RgTextureHandle,
        range: RenderGraphTextureSubresourceRange,
        intent: RenderGraphResourceAccessIntent,
        attachment_ops: Option<RenderGraphAttachmentOps>,
    ) -> Result<RenderGraphResourceVersionToken, RenderGraphError> {
        self.add_versioned_resource_write_with_metadata(
            pass,
            RenderGraphResource::TransientTexture(texture),
            attachment_ops,
            RenderGraphResourceAccessMetadata::new(
                RenderGraphResourceAccessRange::Texture(range),
                intent,
            ),
        )
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

    pub fn write_storage_texture_versioned(
        &mut self,
        pass: RenderPassId,
        texture: RgTextureHandle,
    ) -> Result<RenderGraphResourceVersionToken, RenderGraphError> {
        self.add_versioned_resource_write(
            pass,
            RenderGraphResource::TransientTexture(texture),
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

    pub fn write_texture_with_ops_versioned(
        &mut self,
        pass: RenderPassId,
        texture: RgTextureHandle,
        ops: RenderGraphAttachmentOps,
    ) -> Result<RenderGraphResourceVersionToken, RenderGraphError> {
        self.add_versioned_resource_write(
            pass,
            RenderGraphResource::TransientTexture(texture),
            Some(ops),
        )
    }

    pub fn write_texture_with_ops_from_version(
        &mut self,
        pass: RenderPassId,
        texture: RgTextureHandle,
        ops: RenderGraphAttachmentOps,
        input_version: RenderGraphResourceVersionToken,
    ) -> Result<RenderGraphResourceVersionToken, RenderGraphError> {
        self.add_versioned_attachment_write_from_version(
            pass,
            RenderGraphResource::TransientTexture(texture),
            ops,
            input_version,
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

    /// Declares a buffer read with the exact static byte window and use intent.
    pub fn read_buffer_with_access(
        &mut self,
        pass: RenderPassId,
        buffer: RgBufferHandle,
        range: RenderGraphBufferRange,
        intent: RenderGraphResourceAccessIntent,
    ) -> Result<(), RenderGraphError> {
        self.access_buffer(
            pass,
            buffer,
            RenderGraphResourceAccessKind::Read,
            range,
            intent,
        )
    }

    /// Declares a buffer read or write with an exact static byte window and use intent.
    pub fn access_buffer(
        &mut self,
        pass: RenderPassId,
        buffer: RgBufferHandle,
        access: RenderGraphResourceAccessKind,
        range: RenderGraphBufferRange,
        intent: RenderGraphResourceAccessIntent,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access_with_metadata(
            pass,
            RenderGraphResource::TransientBuffer(buffer),
            resource_access_kind(access),
            None,
            RenderGraphResourceAccessMetadata::new(
                RenderGraphResourceAccessRange::Buffer(range),
                intent,
            ),
        )
    }

    pub fn read_buffer_from_version(
        &mut self,
        pass: RenderPassId,
        version: RenderGraphResourceVersionToken,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access_from_version(
            pass,
            RenderGraphResourceKind::TransientBuffer,
            ResourceAccessKind::Read,
            None,
            version,
        )
    }

    /// Declares an exact buffer range consumed from one exact producer access.
    pub fn read_buffer_with_access_from_version(
        &mut self,
        pass: RenderPassId,
        version: RenderGraphResourceVersionToken,
        range: RenderGraphBufferRange,
        intent: RenderGraphResourceAccessIntent,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access_from_version_with_metadata(
            pass,
            RenderGraphResourceKind::TransientBuffer,
            ResourceAccessKind::Read,
            None,
            version,
            RenderGraphResourceAccessMetadata::new(
                RenderGraphResourceAccessRange::Buffer(range),
                intent,
            ),
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

    pub fn write_buffer_versioned(
        &mut self,
        pass: RenderPassId,
        buffer: RgBufferHandle,
    ) -> Result<RenderGraphResourceVersionToken, RenderGraphError> {
        self.add_versioned_resource_write(pass, RenderGraphResource::TransientBuffer(buffer), None)
    }

    /// Declares an exact buffer write and returns its stable producer token.
    pub fn write_buffer_with_access_versioned(
        &mut self,
        pass: RenderPassId,
        buffer: RgBufferHandle,
        range: RenderGraphBufferRange,
        intent: RenderGraphResourceAccessIntent,
    ) -> Result<RenderGraphResourceVersionToken, RenderGraphError> {
        self.add_versioned_resource_write_with_metadata(
            pass,
            RenderGraphResource::TransientBuffer(buffer),
            None,
            RenderGraphResourceAccessMetadata::new(
                RenderGraphResourceAccessRange::Buffer(range),
                intent,
            ),
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

    /// Declares an imported-resource read with a typed physical scope.
    ///
    /// Report-only external imports cannot consume a typed range because no
    /// texture or buffer descriptor exists to validate or later materialize it.
    pub fn read_external_with_access(
        &mut self,
        pass: RenderPassId,
        external: ExternalResource,
        range: RenderGraphResourceAccessRange,
        intent: RenderGraphResourceAccessIntent,
    ) -> Result<(), RenderGraphError> {
        self.access_external(
            pass,
            external,
            RenderGraphResourceAccessKind::Read,
            range,
            intent,
            None,
        )
    }

    /// Reads one exact imported-resource value through its producer token.
    pub fn read_external_with_access_from_version(
        &mut self,
        pass: RenderPassId,
        version: RenderGraphResourceVersionToken,
        range: RenderGraphResourceAccessRange,
        intent: RenderGraphResourceAccessIntent,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access_from_version_with_metadata(
            pass,
            RenderGraphResourceKind::External,
            ResourceAccessKind::Read,
            None,
            version,
            RenderGraphResourceAccessMetadata::new(range, intent),
        )
    }

    /// Declares an imported-resource read or write with a typed physical scope.
    ///
    /// Report-only external imports cannot consume a typed range because no
    /// texture or buffer descriptor exists to validate or later materialize it.
    pub fn access_external(
        &mut self,
        pass: RenderPassId,
        external: ExternalResource,
        access: RenderGraphResourceAccessKind,
        range: RenderGraphResourceAccessRange,
        intent: RenderGraphResourceAccessIntent,
        attachment_ops: Option<RenderGraphAttachmentOps>,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access_with_metadata(
            pass,
            RenderGraphResource::External(external),
            resource_access_kind(access),
            attachment_ops,
            RenderGraphResourceAccessMetadata::new(range, intent),
        )
    }

    pub fn read_external_from_version(
        &mut self,
        pass: RenderPassId,
        version: RenderGraphResourceVersionToken,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access_from_version(
            pass,
            RenderGraphResourceKind::External,
            ResourceAccessKind::Read,
            None,
            version,
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

    pub fn write_external_versioned(
        &mut self,
        pass: RenderPassId,
        external: ExternalResource,
    ) -> Result<RenderGraphResourceVersionToken, RenderGraphError> {
        self.write_external_with_ops_versioned(
            pass,
            external,
            RenderGraphAttachmentOps::load_store(),
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

    pub fn write_storage_external_versioned(
        &mut self,
        pass: RenderPassId,
        external: ExternalResource,
    ) -> Result<RenderGraphResourceVersionToken, RenderGraphError> {
        self.add_versioned_resource_write(pass, RenderGraphResource::External(external), None)
    }

    /// Writes an exact imported-resource scope and returns its stable producer token.
    pub fn write_external_with_access_versioned(
        &mut self,
        pass: RenderPassId,
        external: ExternalResource,
        range: RenderGraphResourceAccessRange,
        intent: RenderGraphResourceAccessIntent,
        attachment_ops: Option<RenderGraphAttachmentOps>,
    ) -> Result<RenderGraphResourceVersionToken, RenderGraphError> {
        self.add_versioned_resource_write_with_metadata(
            pass,
            RenderGraphResource::External(external),
            attachment_ops,
            RenderGraphResourceAccessMetadata::new(range, intent),
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

    pub fn write_external_with_ops_versioned(
        &mut self,
        pass: RenderPassId,
        external: ExternalResource,
        ops: RenderGraphAttachmentOps,
    ) -> Result<RenderGraphResourceVersionToken, RenderGraphError> {
        self.add_versioned_resource_write(pass, RenderGraphResource::External(external), Some(ops))
    }

    pub fn write_external_with_ops_from_version(
        &mut self,
        pass: RenderPassId,
        external: ExternalResource,
        ops: RenderGraphAttachmentOps,
        input_version: RenderGraphResourceVersionToken,
    ) -> Result<RenderGraphResourceVersionToken, RenderGraphError> {
        self.add_versioned_attachment_write_from_version(
            pass,
            RenderGraphResource::External(external),
            ops,
            input_version,
        )
    }

    /// Loads one exact imported attachment scope from a producer and returns the next value.
    pub fn write_external_with_access_from_version(
        &mut self,
        pass: RenderPassId,
        external: ExternalResource,
        range: RenderGraphResourceAccessRange,
        intent: RenderGraphResourceAccessIntent,
        ops: RenderGraphAttachmentOps,
        input_version: RenderGraphResourceVersionToken,
    ) -> Result<RenderGraphResourceVersionToken, RenderGraphError> {
        self.add_versioned_attachment_write_from_version_with_metadata(
            pass,
            RenderGraphResource::External(external),
            ops,
            input_version,
            RenderGraphResourceAccessMetadata::new(range, intent),
        )
    }

    fn add_resource_access(
        &mut self,
        pass: RenderPassId,
        resource: RenderGraphResource,
        kind: ResourceAccessKind,
        attachment_ops: Option<RenderGraphAttachmentOps>,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access_with_metadata(
            pass,
            resource,
            kind,
            attachment_ops,
            Self::legacy_resource_access_metadata(resource),
        )
    }

    fn add_resource_access_with_metadata(
        &mut self,
        pass: RenderPassId,
        resource: RenderGraphResource,
        kind: ResourceAccessKind,
        attachment_ops: Option<RenderGraphAttachmentOps>,
        metadata: RenderGraphResourceAccessMetadata,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access_with_metadata_and_input_version(
            pass,
            resource,
            kind,
            attachment_ops,
            metadata,
            None,
        )
        .map(|_| ())
    }

    fn add_resource_access_with_metadata_and_input_version(
        &mut self,
        pass: RenderPassId,
        resource: RenderGraphResource,
        kind: ResourceAccessKind,
        attachment_ops: Option<RenderGraphAttachmentOps>,
        metadata: RenderGraphResourceAccessMetadata,
        input_version: Option<RenderGraphResourceVersionToken>,
    ) -> Result<usize, RenderGraphError> {
        self.ensure_pass(pass)?;
        self.ensure_resource(resource)?;
        let access_index = self.passes[pass.0].resources.len();
        self.passes[pass.0].resources.push(ResourceAccess {
            resource,
            kind,
            input_version,
            attachment_ops,
            metadata,
        });
        Ok(access_index)
    }

    fn legacy_resource_access_metadata(
        resource: RenderGraphResource,
    ) -> RenderGraphResourceAccessMetadata {
        let range = match resource {
            RenderGraphResource::TransientTexture(_) => {
                RenderGraphResourceAccessRange::Texture(RenderGraphTextureSubresourceRange::full())
            }
            RenderGraphResource::TransientBuffer(_) => {
                RenderGraphResourceAccessRange::Buffer(RenderGraphBufferRange::full())
            }
            RenderGraphResource::External(_) => RenderGraphResourceAccessRange::UnresolvedExternal,
        };
        RenderGraphResourceAccessMetadata::new(range, RenderGraphResourceAccessIntent::Legacy)
    }

    fn add_versioned_resource_write(
        &mut self,
        pass: RenderPassId,
        resource: RenderGraphResource,
        attachment_ops: Option<RenderGraphAttachmentOps>,
    ) -> Result<RenderGraphResourceVersionToken, RenderGraphError> {
        self.add_versioned_resource_write_with_metadata(
            pass,
            resource,
            attachment_ops,
            Self::legacy_resource_access_metadata(resource),
        )
    }

    fn add_versioned_resource_write_with_metadata(
        &mut self,
        pass: RenderPassId,
        resource: RenderGraphResource,
        attachment_ops: Option<RenderGraphAttachmentOps>,
        metadata: RenderGraphResourceAccessMetadata,
    ) -> Result<RenderGraphResourceVersionToken, RenderGraphError> {
        let access_index = self.add_resource_access_with_metadata_and_input_version(
            pass,
            resource,
            ResourceAccessKind::Write,
            attachment_ops,
            metadata,
            None,
        )?;
        Ok(RenderGraphResourceVersionToken::new(
            resource,
            pass,
            access_index,
            self.generation,
        ))
    }

    fn add_versioned_attachment_write_from_version(
        &mut self,
        pass: RenderPassId,
        resource: RenderGraphResource,
        ops: RenderGraphAttachmentOps,
        input_version: RenderGraphResourceVersionToken,
    ) -> Result<RenderGraphResourceVersionToken, RenderGraphError> {
        self.add_versioned_attachment_write_from_version_with_metadata(
            pass,
            resource,
            ops,
            input_version,
            Self::legacy_resource_access_metadata(resource),
        )
    }

    fn add_versioned_attachment_write_from_version_with_metadata(
        &mut self,
        pass: RenderPassId,
        resource: RenderGraphResource,
        ops: RenderGraphAttachmentOps,
        input_version: RenderGraphResourceVersionToken,
        metadata: RenderGraphResourceAccessMetadata,
    ) -> Result<RenderGraphResourceVersionToken, RenderGraphError> {
        self.ensure_pass(pass)?;
        if ops.load != RenderGraphAttachmentLoadOp::Load {
            return Err(RenderGraphError::ResourceVersionRequiresAttachmentLoad {
                pass: self.passes[pass.0].name.clone(),
                resource: self.resource_name(resource),
            });
        }
        self.validate_resource_version_token(pass, resource, input_version)?;
        self.ensure_resource(resource)?;
        let access_index = self.add_resource_access_with_metadata_and_input_version(
            pass,
            resource,
            ResourceAccessKind::Write,
            Some(ops),
            metadata,
            Some(input_version),
        )?;
        Ok(RenderGraphResourceVersionToken::new(
            resource,
            pass,
            access_index,
            self.generation,
        ))
    }

    fn add_resource_access_from_version(
        &mut self,
        pass: RenderPassId,
        expected_kind: RenderGraphResourceKind,
        kind: ResourceAccessKind,
        attachment_ops: Option<RenderGraphAttachmentOps>,
        input_version: RenderGraphResourceVersionToken,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access_from_version_with_metadata(
            pass,
            expected_kind,
            kind,
            attachment_ops,
            input_version,
            Self::legacy_resource_access_metadata(input_version.resource()),
        )
    }

    fn add_resource_access_from_version_with_metadata(
        &mut self,
        pass: RenderPassId,
        expected_kind: RenderGraphResourceKind,
        kind: ResourceAccessKind,
        attachment_ops: Option<RenderGraphAttachmentOps>,
        input_version: RenderGraphResourceVersionToken,
        metadata: RenderGraphResourceAccessMetadata,
    ) -> Result<(), RenderGraphError> {
        self.ensure_pass(pass)?;
        let resource = input_version.resource();
        if resource.kind() != expected_kind {
            return Err(RenderGraphError::ResourceVersionResourceMismatch {
                pass: self.passes[pass.0].name.clone(),
                expected_resource: format!("{expected_kind:?}"),
                producer_resource: self.resource_name(resource),
            });
        }
        self.validate_resource_version_token(pass, resource, input_version)?;
        self.ensure_resource(resource)?;
        self.add_resource_access_with_metadata_and_input_version(
            pass,
            resource,
            kind,
            attachment_ops,
            metadata,
            Some(input_version),
        )
        .map(|_| ())
    }
}
