use super::{ComputePassDescriptor, ComputeShaderSource};
use crate::core::framework::render::{RenderFrameExtract, RenderWorldSnapshotHandle};
use crate::graphics::feature::RenderFeaturePassDescriptor;
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::{
    RenderBufferSchema, RenderFeatureCapabilityRequirement, RenderFeatureDescriptor,
    RenderFeatureResourceAccess, RenderFeatureResourceKind, RenderPipelineAsset,
    RenderPipelineCompileOptions, RenderResourceSchema, RenderTextureSchema,
};
use crate::render_graph::{
    BindingSchemaEntry, ComputeBindingKind, PassFlags, QueueLane, RenderGraphBufferRange,
    RenderGraphComputeDispatchExtent, RenderGraphComputePipelineFallbackPolicy,
    RenderGraphExternalResourceBinding, RenderGraphResourceAccessIntent,
    RenderGraphResourceAccessKind, RenderGraphResourceAccessMetadata,
    RenderGraphResourceAccessRange, RenderGraphShaderStages, RenderGraphTextureSubresourceRange,
};
use crate::rhi::{BufferUsage, TextureFormat, TextureUsage};
use crate::scene::world::World;

#[test]
fn compute_pass_lowering_preserves_bindings_dispatch_and_generic_executor() {
    let pass = ComputePassDescriptor::new(
        "plugin-reduce",
        RenderPassStage::PostProcess,
        QueueLane::AsyncCompute,
        ComputeShaderSource::inline_wgsl(
            "plugin-reduce",
            "@compute @workgroup_size(8, 8, 1) fn cs_main() {}",
        ),
        "cs_main",
        [8, 8, 1],
        vec![
            BindingSchemaEntry::new(0, "scene-color", ComputeBindingKind::SampledTexture),
            BindingSchemaEntry::new(1, "plugin-output", ComputeBindingKind::StorageTextureWrite),
        ],
        RenderGraphComputeDispatchExtent::PerPixel {
            target: "plugin-output".to_string(),
            local_size: [8, 8],
        },
        PassFlags {
            allow_culling: false,
            has_side_effects: true,
        },
    )
    .with_last_good_pipeline("plugin.reduce", 4)
    .into_feature_pass();

    assert_eq!(pass.executor_id.as_str(), "compute.generic");
    assert_eq!(
        pass.compute_workload.as_ref().unwrap().pipeline_label,
        "plugin-reduce"
    );
    assert!(matches!(
        &pass
            .compute_workload
            .as_ref()
            .expect("compute workload")
            .pipeline_fallback_policy,
        RenderGraphComputePipelineFallbackPolicy::LastGood(family)
            if family.name == "plugin.reduce" && family.interface_generation == 4
    ));
    assert_eq!(pass.resources.len(), 2);
    let compute_metadata = pass.compute_pass.as_ref().unwrap().graph_metadata();
    assert_eq!(compute_metadata.entry_point, "cs_main");
    assert_eq!(compute_metadata.bindings.len(), 2);
    assert!(matches!(
        compute_metadata.shader,
        crate::render_graph::RenderGraphComputeShaderSource::Wgsl { label, .. }
            if label == "plugin-reduce"
    ));
    assert!(pass.resources.iter().any(|resource| {
        resource.name == "scene-color"
            && resource.kind == RenderFeatureResourceKind::Texture
            && resource.access == RenderFeatureResourceAccess::Read
            && resource.external_binding == RenderGraphExternalResourceBinding::required_texture()
    }));
    assert!(pass.resources.iter().any(|resource| {
        resource.name == "plugin-output"
            && resource.kind == RenderFeatureResourceKind::Texture
            && resource.access == RenderFeatureResourceAccess::Write
            && resource.external_binding == RenderGraphExternalResourceBinding::required_texture()
    }));
}

#[test]
fn compute_binding_can_request_an_owned_full_mip_chain() {
    let binding = BindingSchemaEntry::new(5, "hzb-furthest", ComputeBindingKind::SampledTexture)
        .with_texture_full_mip_chain();

    assert_eq!(binding.texture_mip_level, None);
    assert!(binding.texture_full_mip_chain);
}

#[test]
fn compute_pass_lowering_preserves_canonical_binding_scope_and_intent() {
    let pass = ComputePassDescriptor::new(
        "scoped-compute",
        RenderPassStage::PostProcess,
        QueueLane::AsyncCompute,
        ComputeShaderSource::inline_wgsl(
            "scoped-compute",
            "@compute @workgroup_size(1) fn cs_main() {}",
        ),
        "cs_main",
        [1, 1, 1],
        vec![
            BindingSchemaEntry::new(0, "input-mip", ComputeBindingKind::SampledTexture)
                .with_texture_mip_level(2),
            BindingSchemaEntry::new(
                1,
                "read-write-window",
                ComputeBindingKind::StorageBufferReadWrite,
            )
            .with_buffer_range(16, Some(64)),
        ],
        RenderGraphComputeDispatchExtent::Fixed([1, 1, 1]),
        PassFlags {
            allow_culling: false,
            has_side_effects: true,
        },
    )
    .into_feature_pass();

    let input = pass
        .resources
        .iter()
        .find(|resource| {
            resource.name == "input-mip" && resource.access == RenderFeatureResourceAccess::Read
        })
        .expect("sampled binding lowers to a read resource");
    assert_eq!(
        input.access_metadata,
        Some(RenderGraphResourceAccessMetadata::new(
            RenderGraphResourceAccessRange::Texture(
                RenderGraphTextureSubresourceRange::single_mip(2,)
            ),
            RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::COMPUTE),
        ))
    );

    let read_window = pass
        .resources
        .iter()
        .find(|resource| {
            resource.name == "read-write-window"
                && resource.access == RenderFeatureResourceAccess::Read
        })
        .expect("read-write binding retains its read provenance");
    assert_eq!(
        read_window.access_metadata,
        Some(RenderGraphResourceAccessMetadata::new(
            RenderGraphResourceAccessRange::Buffer(RenderGraphBufferRange::new(16, Some(64))),
            RenderGraphResourceAccessIntent::storage_buffer_read(RenderGraphShaderStages::COMPUTE),
        ))
    );

    let write_window = pass
        .resources
        .iter()
        .find(|resource| {
            resource.name == "read-write-window"
                && resource.access == RenderFeatureResourceAccess::Write
        })
        .expect("read-write binding retains its write provenance");
    assert_eq!(
        write_window.access_metadata,
        Some(RenderGraphResourceAccessMetadata::new(
            RenderGraphResourceAccessRange::Buffer(RenderGraphBufferRange::new(16, Some(64))),
            RenderGraphResourceAccessIntent::storage_buffer_read_write(
                RenderGraphShaderStages::COMPUTE,
            ),
        ))
    );
}

#[test]
fn compute_pass_lowering_declares_unbound_per_pixel_dispatch_target() {
    let pass = ComputePassDescriptor::new(
        "plugin-dispatch-size",
        RenderPassStage::PostProcess,
        QueueLane::AsyncCompute,
        ComputeShaderSource::inline_wgsl(
            "plugin-dispatch-size",
            "@compute @workgroup_size(8, 8, 1) fn cs_main() {}",
        ),
        "cs_main",
        [8, 8, 1],
        Vec::new(),
        RenderGraphComputeDispatchExtent::PerPixel {
            target: "dispatch-size".to_string(),
            local_size: [8, 8],
        },
        PassFlags {
            allow_culling: false,
            has_side_effects: true,
        },
    )
    .into_feature_pass();

    assert_eq!(pass.resources.len(), 1);
    assert!(pass.resources.iter().any(|resource| {
        resource.name == "dispatch-size"
            && resource.kind == RenderFeatureResourceKind::Texture
            && resource.access == RenderFeatureResourceAccess::Read
            && resource.external_binding == RenderGraphExternalResourceBinding::required_texture()
    }));
}

#[test]
fn plugin_compute_buffer_schema_reaches_the_compiled_transient_buffer_descriptor() {
    let schema = RenderResourceSchema::buffer(RenderBufferSchema::new(
        4_096,
        BufferUsage::STORAGE | BufferUsage::COPY_SRC,
    ));
    let compute_pass = ComputePassDescriptor::new(
        "plugin-buffer-schema",
        RenderPassStage::PostProcess,
        QueueLane::AsyncCompute,
        ComputeShaderSource::inline_wgsl(
            "plugin-buffer-schema",
            "@group(1) @binding(0) var<storage, read_write> plugin_output: array<u32>;\n@compute @workgroup_size(1) fn cs_main() { plugin_output[0] = 1u; }",
        ),
        "cs_main",
        [1, 1, 1],
        vec![BindingSchemaEntry::new(
            0,
            "plugin-buffer-output",
            ComputeBindingKind::StorageBufferReadWrite,
        )
        .with_buffer_range(16, Some(256))],
        RenderGraphComputeDispatchExtent::Fixed([1, 1, 1]),
        PassFlags {
            allow_culling: false,
            has_side_effects: true,
        },
    )
    .with_resource_schema("plugin-buffer-output", schema)
    .into_feature_pass();
    let pipeline = RenderPipelineAsset::default_forward_plus().with_plugin_render_features([
        RenderFeatureDescriptor::new(
            "plugin.buffer-schema",
            Vec::new(),
            Vec::new(),
            vec![compute_pass],
        ),
    ]);

    let compiled = pipeline
        .compile(&test_extract())
        .expect("typed plugin buffer descriptor should compile");
    let output = compiled
        .graph()
        .resource_lifetime_by_name("plugin-buffer-output")
        .expect("typed plugin buffer lifetime");
    assert!(matches!(
        &output.desc,
        crate::render_graph::RenderGraphResourceDesc::Buffer(desc)
            if desc.size_bytes == 4_096
                && desc.usage == (BufferUsage::STORAGE | BufferUsage::COPY_SRC)
    ));
    let pass = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "plugin-buffer-schema")
        .expect("compiled compute pass");
    let read_access = compiled
        .graph()
        .access_id_at(pass.id, 0)
        .expect("read access ID");
    let write_access = compiled
        .graph()
        .access_id_at(pass.id, 1)
        .expect("write access ID");
    assert_ne!(read_access, write_access);
    let packet = compiled
        .graph()
        .compute_binding_access_packet(pass.id)
        .expect("live compute pass receives an immutable binding packet");
    let binding = packet
        .binding(0)
        .expect("compute binding slot has one packet row");
    assert_eq!(binding.kind, ComputeBindingKind::StorageBufferReadWrite);
    assert_eq!(
        binding.read_access.map(|key| key.access_id),
        Some(read_access)
    );
    assert_eq!(
        binding.write_access.map(|key| key.access_id),
        Some(write_access)
    );
    assert_eq!(
        compiled.graph().access_metadata(read_access),
        Some(RenderGraphResourceAccessMetadata::new(
            RenderGraphResourceAccessRange::Buffer(RenderGraphBufferRange::new(16, Some(256))),
            RenderGraphResourceAccessIntent::storage_buffer_read(RenderGraphShaderStages::COMPUTE),
        ))
    );
    assert_eq!(
        compiled.graph().access_metadata(write_access),
        Some(RenderGraphResourceAccessMetadata::new(
            RenderGraphResourceAccessRange::Buffer(RenderGraphBufferRange::new(16, Some(256))),
            RenderGraphResourceAccessIntent::storage_buffer_read_write(
                RenderGraphShaderStages::COMPUTE,
            ),
        ))
    );
    assert_eq!(
        compiled
            .graph()
            .versioned_access_key(read_access)
            .map(|key| key.access),
        Some(RenderGraphResourceAccessKind::Read)
    );
    assert_eq!(
        compiled
            .graph()
            .versioned_access_key(write_access)
            .map(|key| key.access),
        Some(RenderGraphResourceAccessKind::Write)
    );
}

#[test]
fn plugin_compute_external_buffer_schema_reaches_the_compiled_physical_contract() {
    let compute_pass = ComputePassDescriptor::new(
        "plugin-external-buffer-schema",
        RenderPassStage::PostProcess,
        QueueLane::AsyncCompute,
        ComputeShaderSource::inline_wgsl(
            "plugin-external-buffer-schema",
            "@group(1) @binding(0) var<uniform> plugin_input: vec4<u32>;\n@compute @workgroup_size(1) fn cs_main() {}",
        ),
        "cs_main",
        [1, 1, 1],
        vec![BindingSchemaEntry::new(
            0,
            "plugin-external-buffer",
            ComputeBindingKind::UniformBuffer,
        )],
        RenderGraphComputeDispatchExtent::Fixed([1, 1, 1]),
        PassFlags {
            allow_culling: false,
            has_side_effects: true,
        },
    )
    .with_resource_schema(
        "plugin-external-buffer",
        RenderResourceSchema::buffer(RenderBufferSchema::new(
            16,
            BufferUsage::UNIFORM | BufferUsage::COPY_DST,
        )),
    )
    .into_feature_pass();
    let pipeline = RenderPipelineAsset::default_forward_plus().with_plugin_render_features([
        RenderFeatureDescriptor::new(
            "plugin.external-buffer-schema",
            Vec::new(),
            Vec::new(),
            vec![compute_pass],
        ),
    ]);

    let compiled = pipeline
        .compile(&test_extract())
        .expect("typed external buffers should retain a graph-to-physical contract");
    let external = compiled
        .graph()
        .resource_lifetime_by_name("plugin-external-buffer")
        .expect("typed external buffer lifetime");
    assert!(matches!(
        &external.desc,
        crate::render_graph::RenderGraphResourceDesc::External
    ));
    assert!(matches!(
        external.external_buffer_desc.as_ref(),
        Some(desc)
            if desc.size_bytes == 16
                && desc.usage == (BufferUsage::UNIFORM | BufferUsage::COPY_DST)
    ));
}

#[test]
fn plugin_compute_descriptor_compiles_into_the_render_graph_with_an_explicit_storage_schema() {
    let compute_pass = ComputePassDescriptor::new(
        "plugin-compute-output",
        RenderPassStage::PostProcess,
        QueueLane::AsyncCompute,
        ComputeShaderSource::inline_wgsl(
            "plugin-compute-output",
            "@group(1) @binding(0) var plugin_compute_output: texture_storage_2d<rgba8unorm, write>;\n@compute @workgroup_size(8, 8, 1) fn cs_main() { textureStore(plugin_compute_output, vec2<i32>(0), vec4<f32>(0.0)); }",
        ),
        "cs_main",
        [8, 8, 1],
        vec![BindingSchemaEntry::new(
            0,
            "plugin-compute-output",
            ComputeBindingKind::StorageTextureWrite,
        )],
        RenderGraphComputeDispatchExtent::PerPixel {
            target: "plugin-compute-output".to_string(),
            local_size: [8, 8],
        },
        PassFlags {
            allow_culling: false,
            has_side_effects: true,
        },
    )
    .with_resource_schema(
        "plugin-compute-output",
        RenderResourceSchema::texture(RenderTextureSchema::new(
            TextureFormat::Rgba8Unorm,
            TextureUsage::SAMPLED | TextureUsage::STORAGE | TextureUsage::COPY_DST,
        )),
    )
    .into_feature_pass();
    let pipeline = RenderPipelineAsset::default_forward_plus().with_plugin_render_features([
        RenderFeatureDescriptor::new(
            "plugin.compute-descriptor",
            Vec::new(),
            Vec::new(),
            vec![compute_pass],
        )
        .with_capability_requirement(RenderFeatureCapabilityRequirement::NeuralCompute),
    ]);

    let disabled = pipeline
        .compile(&test_extract())
        .expect("disabled plugin compute descriptor should not enter the render graph");
    assert!(
        !disabled
            .graph()
            .passes()
            .iter()
            .any(|pass| pass.name == "plugin-compute-output")
    );

    let compiled = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default()
                .with_capability_enabled(RenderFeatureCapabilityRequirement::NeuralCompute),
        )
        .expect("plugin compute descriptor should compile into the render graph");
    let pass = compiled
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == "plugin-compute-output")
        .expect("compiled graph should include the plugin compute pass");

    assert_eq!(pass.executor_id.as_deref(), Some("compute.generic"));
    assert_eq!(
        pass.compute_workload
            .as_ref()
            .map(|workload| workload.workgroup_size),
        Some([8, 8, 1])
    );
    assert!(pass.compute_pass_metadata.is_some());
    let output = compiled
        .graph()
        .resource_lifetime_by_name("plugin-compute-output")
        .expect("typed plugin output lifetime");
    assert!(matches!(
        &output.desc,
        crate::render_graph::RenderGraphResourceDesc::Texture(desc)
            if desc.format == TextureFormat::Rgba8Unorm
                && desc.usage.contains(TextureUsage::STORAGE)
    ));
}

#[test]
fn plugin_compute_descriptor_rejects_untyped_storage_texture_output() {
    let compute_pass = ComputePassDescriptor::new(
        "plugin-untyped-compute-output",
        RenderPassStage::PostProcess,
        QueueLane::AsyncCompute,
        ComputeShaderSource::inline_wgsl(
            "plugin-untyped-compute-output",
            "@group(1) @binding(0) var plugin_compute_output: texture_storage_2d<rgba8unorm, write>;\n@compute @workgroup_size(8, 8, 1) fn cs_main() { textureStore(plugin_compute_output, vec2<i32>(0), vec4<f32>(0.0)); }",
        ),
        "cs_main",
        [8, 8, 1],
        vec![BindingSchemaEntry::new(
            0,
            "plugin-untyped-compute-output",
            ComputeBindingKind::StorageTextureWrite,
        )],
        RenderGraphComputeDispatchExtent::PerPixel {
            target: "plugin-untyped-compute-output".to_string(),
            local_size: [8, 8],
        },
        PassFlags {
            allow_culling: false,
            has_side_effects: true,
        },
    )
    .into_feature_pass();
    let pipeline = RenderPipelineAsset::default_forward_plus().with_plugin_render_features([
        RenderFeatureDescriptor::new(
            "plugin.untyped-compute-descriptor",
            Vec::new(),
            Vec::new(),
            vec![compute_pass],
        )
        .with_capability_requirement(RenderFeatureCapabilityRequirement::NeuralCompute),
    ]);

    let error = pipeline
        .compile_with_options(
            &test_extract(),
            &RenderPipelineCompileOptions::default()
                .with_capability_enabled(RenderFeatureCapabilityRequirement::NeuralCompute),
        )
        .expect_err("storage texture plugin outputs require a typed schema");

    assert!(
        error.contains(
            "transient texture resource `plugin-untyped-compute-output` requires an explicit RenderResourceSchema"
        ),
        "unexpected error: {error}"
    );
}

#[test]
fn compute_pass_lowering_keeps_caller_declared_external_resources() {
    let descriptor = ComputePassDescriptor::new(
        "plugin-output",
        RenderPassStage::PostProcess,
        QueueLane::AsyncCompute,
        ComputeShaderSource::inline_wgsl(
            "plugin-output",
            "@compute @workgroup_size(1) fn cs_main() {}",
        ),
        "cs_main",
        [1, 1, 1],
        vec![BindingSchemaEntry::new(
            0,
            "plugin-output",
            ComputeBindingKind::StorageBufferReadWrite,
        )],
        RenderGraphComputeDispatchExtent::Fixed([1, 1, 1]),
        PassFlags {
            allow_culling: false,
            has_side_effects: true,
        },
    );
    let pass = RenderFeaturePassDescriptor::new(
        RenderPassStage::PostProcess,
        "plugin-output",
        QueueLane::AsyncCompute,
    )
    .read_external_buffer("plugin-output")
    .write_storage_external_buffer("plugin-output")
    .with_compute_pass(descriptor);

    assert_eq!(pass.resources.len(), 2);
    assert!(pass.resources.iter().all(|resource| {
        resource.name == "plugin-output"
            && resource.kind == RenderFeatureResourceKind::External
            && resource.access_metadata.is_none()
    }));
}

#[test]
fn compute_pass_lowering_preserves_external_binding_when_adding_missing_read_access() {
    let descriptor = ComputePassDescriptor::new(
        "plugin-output",
        RenderPassStage::PostProcess,
        QueueLane::AsyncCompute,
        ComputeShaderSource::inline_wgsl(
            "plugin-output",
            "@compute @workgroup_size(1) fn cs_main() {}",
        ),
        "cs_main",
        [1, 1, 1],
        vec![BindingSchemaEntry::new(
            0,
            "plugin-output",
            ComputeBindingKind::StorageBufferReadWrite,
        )],
        RenderGraphComputeDispatchExtent::Fixed([1, 1, 1]),
        PassFlags {
            allow_culling: false,
            has_side_effects: true,
        },
    );
    let pass = RenderFeaturePassDescriptor::new(
        RenderPassStage::PostProcess,
        "plugin-output",
        QueueLane::AsyncCompute,
    )
    .write_storage_external_buffer("plugin-output")
    .with_compute_pass(descriptor);

    assert_eq!(pass.resources.len(), 2);
    assert!(pass.resources.iter().all(|resource| {
        resource.name == "plugin-output"
            && resource.kind == RenderFeatureResourceKind::External
            && resource.access_metadata.is_none()
    }));
    assert!(
        pass.resources
            .iter()
            .any(|resource| resource.access == RenderFeatureResourceAccess::Read)
    );
    assert!(
        pass.resources
            .iter()
            .any(|resource| resource.access == RenderFeatureResourceAccess::Write)
    );
}

fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    )
}
