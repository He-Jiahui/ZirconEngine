use zircon_runtime_interface::resource::{AssetReference, ResourceLocator};

use super::*;
use crate::graphics::shader::invocation::{
    ComputeDispatchBuilder, ComputeKernelRef, FullscreenPassBuilder, FullscreenShaderRef,
    RenderShaderEntryPointDescriptor, RenderShaderStage, ShaderAssetKind,
    ShaderDispatchBuildDiagnostic, ShaderDispatchExtent, ShaderResourceDescriptor,
};

#[test]
fn feature_pass_descriptor_consumes_shader_compute_dispatch_plan_resources() {
    let shader = AssetReference::from_locator(
        ResourceLocator::parse("builtin://shaders/compute/clustered_lighting").unwrap(),
    );
    let dispatch = ComputeDispatchBuilder::new(ComputeKernelRef::new(shader, "cs_main"))
        .with_pipeline_label("zircon-cluster-pipeline")
        .with_workgroup_size([8, 8, 1])
        .bind_storage_write("light-list")
        .bind_sampler("linear_sampler")
        .dispatch_extent(ShaderDispatchExtent::ClusterGrid);
    let dispatch = dispatch
        .build(
            ShaderAssetKind::Compute,
            &[RenderShaderEntryPointDescriptor {
                name: "cs_main".to_string(),
                stage: RenderShaderStage::Compute,
            }],
            &[
                ShaderResourceDescriptor {
                    name: "light-list".to_string(),
                    kind: ShaderResourceKind::StorageBuffer,
                    access: Some(ShaderResourceAccess::Write),
                },
                ShaderResourceDescriptor {
                    name: "linear_sampler".to_string(),
                    kind: ShaderResourceKind::Sampler,
                    access: Some(ShaderResourceAccess::Read),
                },
            ],
        )
        .unwrap();

    let pass = RenderFeaturePassDescriptor::new(
        RenderPassStage::Lighting,
        "light-grid-build",
        QueueLane::AsyncCompute,
    )
    .with_compute_dispatch_plan(&dispatch);

    assert_eq!(
        pass.compute_workload.as_ref().unwrap().pipeline_label,
        "zircon-cluster-pipeline"
    );
    assert_eq!(pass.resources.len(), 1);
    assert_eq!(pass.resources[0].name, "light-list");
    assert_eq!(pass.resources[0].kind, RenderFeatureResourceKind::Buffer);
    assert_eq!(pass.resources[0].access, RenderFeatureResourceAccess::Write);
    assert_eq!(
        pass.resources[0].write_mode,
        RenderFeatureResourceWriteMode::Storage
    );
}

#[test]
fn storage_texture_schema_is_preserved_by_feature_authoring() {
    let schema = crate::render_graph::RenderResourceSchema::texture(
        crate::render_graph::RenderTextureSchema::new(
            crate::rhi::TextureFormat::Rgba8Unorm,
            crate::rhi::TextureUsage::SAMPLED
                | crate::rhi::TextureUsage::STORAGE
                | crate::rhi::TextureUsage::COPY_DST,
        ),
    );
    let pass = RenderFeaturePassDescriptor::new(
        RenderPassStage::PostProcess,
        "typed-storage-output",
        QueueLane::AsyncCompute,
    )
    .write_storage_texture_with_schema("typed-storage-output", schema);

    assert_eq!(pass.resources.len(), 1);
    assert_eq!(pass.resources[0].schema, Some(schema));
}

#[test]
fn persistent_texture_write_is_an_explicit_graph_extraction_root() {
    let pass = RenderFeaturePassDescriptor::new(
        RenderPassStage::PostProcess,
        "history-source",
        QueueLane::Graphics,
    )
    .write_persistent_texture("history-source");

    assert_eq!(pass.resources.len(), 1);
    assert!(pass.resources[0].usage.persistent);
    assert!(pass.resources[0].usage.is_cull_root());
}

#[test]
fn persistent_storage_texture_write_retains_the_storage_contract() {
    let pass = RenderFeaturePassDescriptor::new(
        RenderPassStage::PostProcess,
        "persistent-storage-history-source",
        QueueLane::AsyncCompute,
    )
    .write_persistent_storage_texture("persistent-storage-history-source");

    assert_eq!(pass.resources.len(), 1);
    assert_eq!(
        pass.resources[0].write_mode,
        RenderFeatureResourceWriteMode::Storage
    );
    assert!(pass.resources[0].usage.persistent);
}

#[test]
fn persistent_attachment_write_preserves_explicit_attachment_ops() {
    let pass = RenderFeaturePassDescriptor::new(
        RenderPassStage::PostProcess,
        "persistent-attachment-history-source",
        QueueLane::Graphics,
    )
    .write_persistent_texture_with_ops(
        "persistent-attachment-history-source",
        RenderGraphAttachmentOps::clear_store(),
    );

    assert_eq!(
        pass.resources[0].attachment_ops,
        Some(RenderGraphAttachmentOps::clear_store())
    );
    assert!(pass.resources[0].usage.persistent);
}

#[test]
fn persistent_external_storage_write_retains_its_external_binding() {
    let pass = RenderFeaturePassDescriptor::new(
        RenderPassStage::PostProcess,
        "persistent-external-storage-history-source",
        QueueLane::AsyncCompute,
    )
    .write_persistent_storage_external_texture("persistent-external-storage-history-source");

    assert_eq!(pass.resources[0].kind, RenderFeatureResourceKind::External);
    assert_eq!(
        pass.resources[0].external_binding,
        RenderGraphExternalResourceBinding::report_only_texture()
    );
    assert_eq!(
        pass.resources[0].write_mode,
        RenderFeatureResourceWriteMode::Storage
    );
    assert!(pass.resources[0].usage.persistent);
}

#[test]
fn attachment_and_read_resource_schemas_are_preserved_by_feature_authoring() {
    let texture_schema = crate::render_graph::RenderResourceSchema::texture(
        crate::render_graph::RenderTextureSchema::new(
            crate::rhi::TextureFormat::Rgba16Float,
            crate::rhi::TextureUsage::RENDER_ATTACHMENT
                | crate::rhi::TextureUsage::SAMPLED
                | crate::rhi::TextureUsage::COPY_SRC,
        ),
    );
    let buffer_schema = crate::render_graph::RenderResourceSchema::buffer(
        crate::render_graph::RenderBufferSchema::new(
            256,
            crate::rhi::BufferUsage::STORAGE | crate::rhi::BufferUsage::COPY_DST,
        ),
    );
    let pass = RenderFeaturePassDescriptor::new(
        RenderPassStage::PostProcess,
        "typed-attachment-and-read-resources",
        QueueLane::Graphics,
    )
    .read_texture_with_schema("typed-input", texture_schema)
    .write_texture_with_ops_and_schema(
        "typed-output",
        RenderGraphAttachmentOps::clear_store(),
        texture_schema,
    )
    .read_buffer_with_schema("typed-input-buffer", buffer_schema)
    .write_buffer_with_schema("typed-output-buffer", buffer_schema);

    assert_eq!(pass.resources.len(), 4);
    assert_eq!(pass.resources[0].schema, Some(texture_schema));
    assert_eq!(pass.resources[1].schema, Some(texture_schema));
    assert_eq!(pass.resources[2].schema, Some(buffer_schema));
    assert_eq!(pass.resources[3].schema, Some(buffer_schema));
}

#[test]
fn feature_pass_descriptor_compute_and_fullscreen_contracts_report_named_resource_errors() {
    let compute_shader = AssetReference::from_locator(
        ResourceLocator::parse("res://shaders/simulation.zshader").unwrap(),
    );
    let compute = ComputeDispatchBuilder::new(ComputeKernelRef::new(compute_shader, "cs_main"))
        .bind_texture("particle_state")
        .dispatch_groups([1, 1, 1]);
    let compute_diagnostics = compute
        .build(
            ShaderAssetKind::Compute,
            &[RenderShaderEntryPointDescriptor {
                name: "cs_main".to_string(),
                stage: RenderShaderStage::Compute,
            }],
            &[ShaderResourceDescriptor {
                name: "particle_state".to_string(),
                kind: ShaderResourceKind::StorageBuffer,
                access: Some(ShaderResourceAccess::ReadWrite),
            }],
        )
        .expect_err("compute resource type mismatch should be diagnosed");
    assert!(
        compute_diagnostics.contains(&ShaderDispatchBuildDiagnostic::ResourceKindMismatch {
            name: "particle_state".to_string(),
            expected: ShaderResourceKind::StorageBuffer,
            actual: ShaderResourceKind::Texture,
        })
    );

    let fullscreen_shader = AssetReference::from_locator(
        ResourceLocator::parse("res://shaders/postprocess.zshader").unwrap(),
    );
    let fullscreen =
        FullscreenPassBuilder::new(FullscreenShaderRef::new(fullscreen_shader, "fs_main"))
            .bind_texture("scene_color");
    let fullscreen_plan = fullscreen
        .build(
            ShaderAssetKind::Fullscreen,
            &[RenderShaderEntryPointDescriptor {
                name: "fs_main".to_string(),
                stage: RenderShaderStage::Fragment,
            }],
            &[ShaderResourceDescriptor {
                name: "scene_color".to_string(),
                kind: ShaderResourceKind::Texture,
                access: Some(ShaderResourceAccess::Read),
            }],
        )
        .expect("fullscreen resources should match the authored contract");
    let pass = RenderFeaturePassDescriptor::new(
        RenderPassStage::PostProcess,
        "authoring-fullscreen",
        QueueLane::Graphics,
    )
    .with_fullscreen_pass_plan(&fullscreen_plan);

    assert_eq!(pass.resources.len(), 1);
    assert_eq!(pass.resources[0].name, "scene_color");
    assert_eq!(pass.resources[0].kind, RenderFeatureResourceKind::Texture);
    assert_eq!(pass.resources[0].access, RenderFeatureResourceAccess::Read);
}
