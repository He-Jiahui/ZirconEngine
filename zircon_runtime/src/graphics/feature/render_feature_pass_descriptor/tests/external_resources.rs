use crate::graphics::feature::{
    RenderFeaturePassDescriptor, RenderFeatureResourceAccess, RenderFeatureResourceWriteMode,
};
use crate::graphics::pipeline::RenderPassStage;
use crate::render_graph::{
    QueueLane, RenderBufferSchema, RenderGraphBufferRange, RenderGraphExternalResourceBinding,
    RenderGraphResourceAccessIntent, RenderGraphResourceAccessMetadata,
    RenderGraphResourceAccessRange, RenderGraphShaderStages, RenderGraphTextureSubresourceRange,
    RenderResourceSchema, RenderTextureSchema,
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
fn required_external_buffer_access_retains_required_binding_and_exact_storage_intent() {
    let range = RenderGraphBufferRange::full();
    let intent =
        RenderGraphResourceAccessIntent::storage_buffer_read(RenderGraphShaderStages::COMPUTE);
    let pass = RenderFeaturePassDescriptor::new(
        RenderPassStage::Lighting,
        "required-scene-light-data",
        QueueLane::AsyncCompute,
    )
    .read_required_external_buffer_with_access("scene-light-data", range, intent);

    assert_eq!(pass.resources.len(), 1);
    let resource = &pass.resources[0];
    assert_eq!(resource.access, RenderFeatureResourceAccess::Read);
    assert_eq!(resource.schema, None);
    assert_eq!(
        resource.external_binding,
        RenderGraphExternalResourceBinding::required_buffer()
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
    assert!(pass
        .resources
        .iter()
        .all(|resource| resource.usage.persistent));
    assert!(pass.resources.iter().all(|resource| {
        resource.external_binding == RenderGraphExternalResourceBinding::report_only_buffer()
    }));
    assert_eq!(pass.resources[0].access, RenderFeatureResourceAccess::Read);
    assert_eq!(pass.resources[1].access, RenderFeatureResourceAccess::Write);
    assert!(pass
        .resources
        .iter()
        .all(|resource| resource.schema == Some(schema)));
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
    let intent = RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::COMPUTE);
    let pass = RenderFeaturePassDescriptor::new(
        RenderPassStage::DepthPrepass,
        "catalog-backed-history",
        QueueLane::AsyncCompute,
    )
    .read_persistent_external_texture_with_access("history.previous.hzb", range, intent);

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
