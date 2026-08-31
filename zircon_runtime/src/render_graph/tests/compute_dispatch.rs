use crate::render_graph::{
    BindingSchemaEntry, ComputeBindingKind, PassFlags, QueueLane, RenderGraphBufferRange,
    RenderGraphBuilder, RenderGraphComputeDispatchExtent, RenderGraphComputePassMetadata,
    RenderGraphComputeShaderSource, RenderGraphComputeWorkload, RenderGraphError,
    RenderGraphExternalResourceBinding, RenderGraphResource, RenderGraphResourceAccessIntent,
    RenderGraphResourceAccessKind, RenderGraphResourceAccessRange, RenderGraphShaderStages,
    RenderGraphTextureSubresourceRange,
};
use crate::rhi::{BufferDesc, BufferUsage, TextureDesc, TextureFormat, TextureUsage};

#[test]
fn compute_dispatch_from_buffer_requires_a_declared_buffer_read() {
    let mut graph = RenderGraphBuilder::new("compute-dispatch-indirect");
    let _indirect = graph.import_present_external_resource_with_binding(
        "dispatch-args",
        RenderGraphExternalResourceBinding::required_buffer(),
    );
    let pass = graph.add_pass("reduce", QueueLane::AsyncCompute);
    graph
        .set_pass_flags(
            pass,
            PassFlags {
                allow_culling: false,
                has_side_effects: true,
            },
        )
        .unwrap();
    graph
        .set_compute_workload(
            pass,
            RenderGraphComputeWorkload::new(
                "reduce",
                [8, 1, 1],
                RenderGraphComputeDispatchExtent::FromBuffer {
                    buffer: "dispatch-args".to_string(),
                    offset: 0,
                },
            ),
        )
        .unwrap();

    assert_eq!(
        graph.compile().unwrap_err(),
        RenderGraphError::ComputeDispatchResourceNotDeclared {
            pass: "reduce".to_string(),
            resource: "dispatch-args".to_string(),
            required_access: "read buffer",
        }
    );

    let mut valid_graph = RenderGraphBuilder::new("compute-dispatch-indirect-valid");
    let indirect = valid_graph.import_present_external_buffer_with_binding(
        "dispatch-args",
        BufferDesc::new(
            "dispatch-args",
            64,
            BufferUsage::STORAGE | BufferUsage::INDIRECT,
        ),
        RenderGraphExternalResourceBinding::required_buffer(),
    );
    let pass = valid_graph.add_pass("reduce", QueueLane::AsyncCompute);
    valid_graph
        .read_external_with_access(
            pass,
            indirect,
            RenderGraphResourceAccessRange::Buffer(RenderGraphBufferRange::new(16, Some(12))),
            RenderGraphResourceAccessIntent::storage_buffer_read(RenderGraphShaderStages::COMPUTE),
        )
        .unwrap();
    valid_graph
        .set_pass_flags(
            pass,
            PassFlags {
                allow_culling: false,
                has_side_effects: true,
            },
        )
        .unwrap();
    valid_graph
        .set_compute_workload(
            pass,
            RenderGraphComputeWorkload::new(
                "reduce",
                [8, 1, 1],
                RenderGraphComputeDispatchExtent::FromBuffer {
                    buffer: "dispatch-args".to_string(),
                    offset: 16,
                },
            ),
        )
        .unwrap();
    valid_graph
        .set_compute_pass_metadata(
            pass,
            RenderGraphComputePassMetadata::new(
                RenderGraphComputeShaderSource::wgsl("reduce", "@compute fn cs_main() {}"),
                "cs_main",
                vec![BindingSchemaEntry::new(
                    0,
                    "dispatch-args",
                    ComputeBindingKind::StorageBufferRead,
                )
                .with_buffer_range(16, Some(12))],
            ),
        )
        .unwrap();

    let compiled = valid_graph
        .compile()
        .expect("typed indirect dispatch target compiles");
    let packet = compiled
        .compute_dispatch_access_packet(pass)
        .expect("dynamic dispatch owns an exact compiler packet");
    assert!(matches!(
        packet.dispatch,
        crate::render_graph::CompiledRenderGraphComputeDispatchAccess::Indirect {
            access,
            offset: 16,
        } if access.range == RenderGraphResourceAccessRange::Buffer(RenderGraphBufferRange::new(16, Some(12)))
    ));
}

#[test]
fn compute_dispatch_from_buffer_rejects_a_declared_target_without_indirect_usage() {
    let mut graph = RenderGraphBuilder::new("compute-dispatch-indirect-usage");
    let dispatch_args = graph.create_buffer(BufferDesc::new(
        "dispatch-args",
        64,
        BufferUsage::STORAGE | BufferUsage::COPY_DST,
    ));
    let pass = graph.add_pass("reduce", QueueLane::AsyncCompute);
    graph
        .read_buffer_with_access(
            pass,
            dispatch_args,
            RenderGraphBufferRange::new(0, Some(12)),
            RenderGraphResourceAccessIntent::storage_buffer_read(RenderGraphShaderStages::COMPUTE),
        )
        .unwrap();
    graph
        .set_pass_flags(
            pass,
            PassFlags {
                allow_culling: false,
                has_side_effects: true,
            },
        )
        .unwrap();
    graph
        .set_compute_workload(
            pass,
            RenderGraphComputeWorkload::from_buffer("reduce", [1, 1, 1], "dispatch-args", 0),
        )
        .unwrap();
    graph
        .set_compute_pass_metadata(
            pass,
            RenderGraphComputePassMetadata::new(
                RenderGraphComputeShaderSource::wgsl("reduce", "@compute fn cs_main() {}"),
                "cs_main",
                vec![BindingSchemaEntry::new(
                    0,
                    "dispatch-args",
                    ComputeBindingKind::StorageBufferRead,
                )
                .with_buffer_range(0, Some(12))],
            ),
        )
        .unwrap();

    assert_eq!(
        graph.compile().unwrap_err(),
        RenderGraphError::ComputeIndirectDispatchUsageMissing {
            pass: "reduce".to_string(),
            resource: "dispatch-args".to_string(),
            required: BufferUsage::INDIRECT,
            actual: BufferUsage::STORAGE | BufferUsage::COPY_DST,
        }
    );
}

#[test]
fn compute_dispatch_from_buffer_requires_the_exact_indirect_argument_window() {
    let mut graph = RenderGraphBuilder::new("compute-dispatch-indirect-window");
    let dispatch_args = graph.create_buffer(BufferDesc::new(
        "dispatch-args",
        64,
        BufferUsage::STORAGE | BufferUsage::INDIRECT,
    ));
    let pass = graph.add_pass("reduce", QueueLane::AsyncCompute);
    graph
        .read_buffer_with_access(
            pass,
            dispatch_args,
            RenderGraphBufferRange::new(0, Some(64)),
            RenderGraphResourceAccessIntent::storage_buffer_read(RenderGraphShaderStages::COMPUTE),
        )
        .unwrap();
    graph
        .set_pass_flags(
            pass,
            PassFlags {
                allow_culling: false,
                has_side_effects: true,
            },
        )
        .unwrap();
    graph
        .set_compute_workload(
            pass,
            RenderGraphComputeWorkload::from_buffer("reduce", [1, 1, 1], "dispatch-args", 16),
        )
        .unwrap();
    graph
        .set_compute_pass_metadata(
            pass,
            RenderGraphComputePassMetadata::new(
                RenderGraphComputeShaderSource::wgsl("reduce", "@compute fn cs_main() {}"),
                "cs_main",
                vec![BindingSchemaEntry::new(
                    0,
                    "dispatch-args",
                    ComputeBindingKind::StorageBufferRead,
                )
                .with_buffer_range(0, Some(64))],
            ),
        )
        .unwrap();

    assert_eq!(
        graph.compile().unwrap_err(),
        RenderGraphError::ComputeIndirectDispatchRangeNotExact {
            pass: "reduce".to_string(),
            resource: "dispatch-args".to_string(),
            offset: 16,
            range_start: 0,
            range_end: 64,
        }
    );
}

#[test]
fn compute_dispatch_per_pixel_packet_selects_the_exact_written_texture_access() {
    let mut graph = RenderGraphBuilder::new("compute-dispatch-per-pixel-packet");
    let target = graph.create_texture(TextureDesc::new(
        "compute-output",
        64,
        32,
        TextureFormat::Rgba8Unorm,
        TextureUsage::STORAGE,
    ));
    let pass = graph.add_pass("reduce", QueueLane::AsyncCompute);
    graph
        .write_texture_with_access_versioned(
            pass,
            target,
            RenderGraphTextureSubresourceRange::full(),
            RenderGraphResourceAccessIntent::storage_texture_write(
                RenderGraphShaderStages::COMPUTE,
            ),
            None,
        )
        .unwrap();
    graph
        .set_pass_flags(
            pass,
            PassFlags {
                allow_culling: false,
                has_side_effects: true,
            },
        )
        .unwrap();
    graph
        .set_compute_workload(
            pass,
            RenderGraphComputeWorkload::per_pixel("reduce", [8, 8, 1], "compute-output", [8, 8]),
        )
        .unwrap();
    graph
        .set_compute_pass_metadata(
            pass,
            RenderGraphComputePassMetadata::new(
                RenderGraphComputeShaderSource::wgsl("reduce", "@compute fn cs_main() {}"),
                "cs_main",
                vec![BindingSchemaEntry::new(
                    0,
                    "compute-output",
                    ComputeBindingKind::StorageTextureWrite,
                )],
            ),
        )
        .unwrap();

    let compiled = graph.compile().expect("per-pixel dispatch target compiles");
    let packet = compiled
        .compute_dispatch_access_packet(pass)
        .expect("dynamic dispatch owns an exact compiler packet");
    assert!(matches!(
        packet.dispatch,
        crate::render_graph::CompiledRenderGraphComputeDispatchAccess::PerPixel {
            access,
            target_extent: [64, 32],
            local_size: [8, 8],
        } if access.access == RenderGraphResourceAccessKind::Write
            && access.range == RenderGraphResourceAccessRange::Texture(RenderGraphTextureSubresourceRange::full())
    ));
}

#[test]
fn compute_dispatch_per_pixel_packet_uses_the_local_texture_view_extent() {
    let mut graph = RenderGraphBuilder::new("compute-dispatch-per-pixel-alias");
    let pyramid = graph.create_texture(
        TextureDesc::new(
            "pyramid",
            64,
            32,
            TextureFormat::Rgba16Float,
            TextureUsage::SAMPLED | TextureUsage::STORAGE,
        )
        .with_mip_levels(4),
    );
    let coarse = graph
        .create_texture_view_alias(
            "pyramid-coarse",
            pyramid,
            RenderGraphTextureSubresourceRange::single_mip(2),
        )
        .expect("valid graph texture alias");
    let pass = graph.add_pass("reduce", QueueLane::AsyncCompute);
    graph
        .write_texture_with_access_versioned(
            pass,
            coarse,
            RenderGraphTextureSubresourceRange::full(),
            RenderGraphResourceAccessIntent::storage_texture_write(
                RenderGraphShaderStages::COMPUTE,
            ),
            None,
        )
        .unwrap();
    graph
        .set_pass_flags(
            pass,
            PassFlags {
                allow_culling: false,
                has_side_effects: true,
            },
        )
        .unwrap();
    graph
        .set_compute_workload(
            pass,
            RenderGraphComputeWorkload::per_pixel("reduce", [8, 8, 1], "pyramid-coarse", [8, 8]),
        )
        .unwrap();
    graph
        .set_compute_pass_metadata(
            pass,
            RenderGraphComputePassMetadata::new(
                RenderGraphComputeShaderSource::wgsl("reduce", "@compute fn cs_main() {}"),
                "cs_main",
                vec![BindingSchemaEntry::new(
                    0,
                    "pyramid-coarse",
                    ComputeBindingKind::StorageTextureWrite,
                )],
            ),
        )
        .unwrap();

    let compiled = graph.compile().expect("per-pixel alias target compiles");
    let packet = compiled
        .compute_dispatch_access_packet(pass)
        .expect("dynamic dispatch owns an exact compiler packet");
    assert!(matches!(
        packet.dispatch,
        crate::render_graph::CompiledRenderGraphComputeDispatchAccess::PerPixel {
            target_extent: [16, 8],
            local_size: [8, 8],
            ..
        }
    ));
}

#[test]
fn compute_dispatch_per_pixel_packet_uses_the_selected_texture_access_mip() {
    let mut graph = RenderGraphBuilder::new("compute-dispatch-per-pixel-storage-mip");
    let target = graph.create_texture(
        TextureDesc::new(
            "hzb",
            65,
            33,
            TextureFormat::Rgba16Float,
            TextureUsage::SAMPLED | TextureUsage::STORAGE,
        )
        .with_mip_levels(7),
    );
    let pass = graph.add_pass("hzb-reduce", QueueLane::AsyncCompute);
    graph
        .write_texture_with_access_versioned(
            pass,
            target,
            RenderGraphTextureSubresourceRange::single_mip(2),
            RenderGraphResourceAccessIntent::storage_texture_write(
                RenderGraphShaderStages::COMPUTE,
            ),
            None,
        )
        .unwrap();
    graph
        .set_pass_flags(
            pass,
            PassFlags {
                allow_culling: false,
                has_side_effects: true,
            },
        )
        .unwrap();
    graph
        .set_compute_workload(
            pass,
            RenderGraphComputeWorkload::per_pixel("hzb-reduce", [8, 8, 1], "hzb", [8, 8]),
        )
        .unwrap();
    graph
        .set_compute_pass_metadata(
            pass,
            RenderGraphComputePassMetadata::new(
                RenderGraphComputeShaderSource::wgsl("hzb-reduce", "@compute fn cs_main() {}"),
                "cs_main",
                Vec::new(),
            ),
        )
        .unwrap();

    let compiled = graph.compile().expect("per-pixel mip target compiles");
    let packet = compiled
        .compute_dispatch_access_packet(pass)
        .expect("dynamic dispatch owns an exact compiler packet");
    assert!(matches!(
        packet.dispatch,
        crate::render_graph::CompiledRenderGraphComputeDispatchAccess::PerPixel {
            target_extent: [16, 8],
            local_size: [8, 8],
            ..
        }
    ));
}

#[test]
fn compute_dispatch_per_pixel_packet_falls_back_to_the_exact_read_texture_access() {
    let mut graph = RenderGraphBuilder::new("compute-dispatch-per-pixel-read");
    let target = graph.create_texture(
        TextureDesc::new(
            "source",
            65,
            33,
            TextureFormat::Rgba16Float,
            TextureUsage::SAMPLED | TextureUsage::STORAGE,
        )
        .with_mip_levels(7),
    );
    let pass = graph.add_pass("hzb-reduce", QueueLane::AsyncCompute);
    graph
        .read_texture_with_access(
            pass,
            target,
            RenderGraphTextureSubresourceRange::single_mip(2),
            RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::COMPUTE),
        )
        .unwrap();
    graph
        .set_pass_flags(
            pass,
            PassFlags {
                allow_culling: false,
                has_side_effects: true,
            },
        )
        .unwrap();
    graph
        .set_compute_workload(
            pass,
            RenderGraphComputeWorkload::per_pixel("hzb-reduce", [8, 8, 1], "source", [8, 8]),
        )
        .unwrap();
    graph
        .set_compute_pass_metadata(
            pass,
            RenderGraphComputePassMetadata::new(
                RenderGraphComputeShaderSource::wgsl("hzb-reduce", "@compute fn cs_main() {}"),
                "cs_main",
                vec![
                    BindingSchemaEntry::new(0, "source", ComputeBindingKind::SampledTexture)
                        .with_texture_mip_level(2),
                ],
            ),
        )
        .unwrap();

    let compiled = graph.compile().expect("per-pixel read target compiles");
    let packet = compiled
        .compute_dispatch_access_packet(pass)
        .expect("dynamic dispatch owns an exact compiler packet");
    assert!(matches!(
        packet.dispatch,
        crate::render_graph::CompiledRenderGraphComputeDispatchAccess::PerPixel {
            access,
            target_extent: [16, 8],
            local_size: [8, 8],
        } if access.access == RenderGraphResourceAccessKind::Read
            && access.range == RenderGraphResourceAccessRange::Texture(RenderGraphTextureSubresourceRange::single_mip(2))
    ));
}

#[test]
fn compute_binding_packet_rejects_a_legacy_access_with_a_different_buffer_window() {
    let mut graph = RenderGraphBuilder::new("compute-binding-packet-scope-mismatch");
    let buffer = graph.create_buffer(BufferDesc::new(
        "compute-data",
        256,
        BufferUsage::STORAGE | BufferUsage::COPY_SRC,
    ));
    let pass = graph.add_pass("reduce", QueueLane::AsyncCompute);
    graph.read_buffer(pass, buffer).unwrap();
    graph
        .set_pass_flags(
            pass,
            PassFlags {
                allow_culling: false,
                has_side_effects: true,
            },
        )
        .unwrap();
    graph
        .set_compute_workload(
            pass,
            RenderGraphComputeWorkload::fixed("reduce", [1, 1, 1], [1, 1, 1]),
        )
        .unwrap();
    graph
        .set_compute_pass_metadata(
            pass,
            RenderGraphComputePassMetadata::new(
                RenderGraphComputeShaderSource::wgsl("reduce", "@compute fn cs_main() {}"),
                "cs_main",
                vec![BindingSchemaEntry::new(
                    0,
                    "compute-data",
                    ComputeBindingKind::StorageBufferRead,
                )
                .with_buffer_range(16, Some(64))],
            ),
        )
        .unwrap();

    assert!(matches!(
        graph.compile(),
        Err(RenderGraphError::ComputeBindingAccessScopeMismatch {
            pass,
            binding: 0,
            resource,
            access: RenderGraphResourceAccessKind::Read,
            ..
        }) if pass == "reduce" && resource == "compute-data"
    ));
}

#[test]
fn compute_binding_packet_canonicalizes_a_full_transient_buffer_binding() {
    let mut graph = RenderGraphBuilder::new("compute-binding-packet-full-buffer");
    let buffer = graph.create_buffer(BufferDesc::new(
        "compute-data",
        256,
        BufferUsage::STORAGE | BufferUsage::COPY_SRC,
    ));
    let pass = graph.add_pass("reduce", QueueLane::AsyncCompute);
    graph
        .read_buffer_with_access(
            pass,
            buffer,
            RenderGraphBufferRange::full(),
            RenderGraphResourceAccessIntent::storage_buffer_read(RenderGraphShaderStages::COMPUTE),
        )
        .unwrap();
    graph
        .set_pass_flags(
            pass,
            PassFlags {
                allow_culling: false,
                has_side_effects: true,
            },
        )
        .unwrap();
    graph
        .set_compute_workload(
            pass,
            RenderGraphComputeWorkload::fixed("reduce", [1, 1, 1], [1, 1, 1]),
        )
        .unwrap();
    graph
        .set_compute_pass_metadata(
            pass,
            RenderGraphComputePassMetadata::new(
                RenderGraphComputeShaderSource::wgsl("reduce", "@compute fn cs_main() {}"),
                "cs_main",
                vec![BindingSchemaEntry::new(
                    0,
                    "compute-data",
                    ComputeBindingKind::StorageBufferRead,
                )],
            ),
        )
        .unwrap();

    let compiled = graph
        .compile()
        .expect("full scoped buffer binding compiles");
    let access = compiled
        .compute_binding_access_packet(pass)
        .expect("live compute pass has a binding packet")
        .binding(0)
        .and_then(|binding| binding.read_access)
        .expect("read-only storage binding has an exact access key");

    assert_eq!(
        access.range,
        RenderGraphResourceAccessRange::Buffer(RenderGraphBufferRange::new(0, Some(256)))
    );
}

#[test]
fn compute_binding_packet_projects_a_texture_view_alias_to_the_parent_scope() {
    let mut graph = RenderGraphBuilder::new("compute-binding-packet-alias");
    let pyramid = graph.create_texture(
        TextureDesc::new(
            "pyramid",
            64,
            32,
            TextureFormat::Rgba16Float,
            TextureUsage::SAMPLED | TextureUsage::STORAGE,
        )
        .with_mip_levels(4),
    );
    let coarse = graph
        .create_texture_view_alias(
            "pyramid-coarse",
            pyramid,
            RenderGraphTextureSubresourceRange::single_mip(2),
        )
        .expect("valid graph texture alias");
    let pass = graph.add_pass("reduce", QueueLane::AsyncCompute);
    graph
        .read_texture_with_access(
            pass,
            coarse,
            RenderGraphTextureSubresourceRange::full(),
            RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::COMPUTE),
        )
        .unwrap();
    graph
        .set_pass_flags(
            pass,
            PassFlags {
                allow_culling: false,
                has_side_effects: true,
            },
        )
        .unwrap();
    graph
        .set_compute_workload(
            pass,
            RenderGraphComputeWorkload::fixed("reduce", [1, 1, 1], [1, 1, 1]),
        )
        .unwrap();
    graph
        .set_compute_pass_metadata(
            pass,
            RenderGraphComputePassMetadata::new(
                RenderGraphComputeShaderSource::wgsl("reduce", "@compute fn cs_main() {}"),
                "cs_main",
                vec![BindingSchemaEntry::new(
                    0,
                    "pyramid-coarse",
                    ComputeBindingKind::SampledTexture,
                )],
            ),
        )
        .unwrap();

    let compiled = graph
        .compile()
        .expect("alias-scoped compute binding compiles");
    let access = compiled
        .compute_binding_access_packet(pass)
        .expect("live compute pass has a binding packet")
        .binding(0)
        .and_then(|binding| binding.read_access)
        .expect("sampled binding has an exact access key");

    assert_eq!(
        access.range,
        RenderGraphResourceAccessRange::Texture(RenderGraphTextureSubresourceRange {
            base_mip_level: 2,
            mip_level_count: Some(1),
            base_array_layer: 0,
            array_layer_count: Some(1),
            aspect: crate::render_graph::RenderGraphTextureAspect::All,
        })
    );
}

#[test]
fn compute_dispatch_per_pixel_requires_a_declared_texture() {
    let mut graph = RenderGraphBuilder::new("compute-dispatch-per-pixel");
    let _output = graph.import_present_external_resource_with_binding(
        "plugin-output",
        RenderGraphExternalResourceBinding::required_texture(),
    );
    let pass = graph.add_pass("plugin-reduce", QueueLane::AsyncCompute);
    graph
        .set_pass_flags(
            pass,
            PassFlags {
                allow_culling: false,
                has_side_effects: true,
            },
        )
        .unwrap();
    graph
        .set_compute_workload(
            pass,
            RenderGraphComputeWorkload::per_pixel(
                "plugin-reduce",
                [8, 8, 1],
                "plugin-output",
                [8, 8],
            ),
        )
        .unwrap();

    assert_eq!(
        graph.compile().unwrap_err(),
        RenderGraphError::ComputeDispatchResourceNotDeclared {
            pass: "plugin-reduce".to_string(),
            resource: "plugin-output".to_string(),
            required_access: "read or write texture",
        }
    );

    let mut valid_graph = RenderGraphBuilder::new("compute-dispatch-per-pixel-valid");
    let output = valid_graph.import_present_external_resource_with_binding(
        "plugin-output",
        RenderGraphExternalResourceBinding::required_texture(),
    );
    let pass = valid_graph.add_pass("plugin-reduce", QueueLane::AsyncCompute);
    valid_graph.read_external(pass, output).unwrap();
    valid_graph
        .set_pass_flags(
            pass,
            PassFlags {
                allow_culling: false,
                has_side_effects: true,
            },
        )
        .unwrap();
    valid_graph
        .set_compute_workload(
            pass,
            RenderGraphComputeWorkload::per_pixel(
                "plugin-reduce",
                [8, 8, 1],
                "plugin-output",
                [8, 8],
            ),
        )
        .unwrap();

    let compiled = valid_graph
        .compile()
        .expect("custom compute workload keeps its scheduling metadata");
    assert_eq!(compiled.compute_dispatch_access_packet(pass), None);
}

#[test]
fn compute_metadata_requires_a_workload_and_unique_bindings() {
    let metadata = RenderGraphComputePassMetadata::new(
        RenderGraphComputeShaderSource::wgsl("reduce", "@compute fn cs_main() {}"),
        "cs_main",
        vec![
            BindingSchemaEntry::new(0, "output", ComputeBindingKind::StorageBufferReadWrite),
            BindingSchemaEntry::new(0, "output", ComputeBindingKind::StorageBufferReadWrite),
        ],
    );
    let mut missing_workload = RenderGraphBuilder::new("compute-metadata-missing-workload");
    let pass = missing_workload.add_pass("reduce", QueueLane::AsyncCompute);
    missing_workload
        .set_pass_flags(
            pass,
            PassFlags {
                allow_culling: false,
                has_side_effects: true,
            },
        )
        .unwrap();
    missing_workload
        .set_compute_pass_metadata(pass, metadata.clone())
        .unwrap();
    assert_eq!(
        missing_workload.compile().unwrap_err(),
        RenderGraphError::ComputePassMetadataMissingWorkload {
            pass: "reduce".to_string(),
        }
    );

    let mut duplicate_binding = RenderGraphBuilder::new("compute-metadata-duplicate-binding");
    let pass = duplicate_binding.add_pass("reduce", QueueLane::AsyncCompute);
    duplicate_binding
        .set_pass_flags(
            pass,
            PassFlags {
                allow_culling: false,
                has_side_effects: true,
            },
        )
        .unwrap();
    duplicate_binding
        .set_compute_workload(
            pass,
            RenderGraphComputeWorkload::fixed("reduce", [1, 1, 1], [1, 1, 1]),
        )
        .unwrap();
    duplicate_binding
        .set_compute_pass_metadata(pass, metadata)
        .unwrap();
    assert_eq!(
        duplicate_binding.compile().unwrap_err(),
        RenderGraphError::DuplicateComputeBinding {
            pass: "reduce".to_string(),
            binding: 0,
        }
    );
}

#[test]
fn compute_metadata_bindings_must_match_declared_graph_resources() {
    let mut graph = RenderGraphBuilder::new("compute-metadata-binding-resource");
    let output = graph.import_present_external_resource_with_binding(
        "output",
        RenderGraphExternalResourceBinding::required_buffer(),
    );
    let pass = graph.add_pass("reduce", QueueLane::AsyncCompute);
    graph.write_storage_external(pass, output).unwrap();
    graph
        .set_pass_flags(
            pass,
            PassFlags {
                allow_culling: false,
                has_side_effects: true,
            },
        )
        .unwrap();
    graph
        .set_compute_workload(
            pass,
            RenderGraphComputeWorkload::fixed("reduce", [1, 1, 1], [1, 1, 1]),
        )
        .unwrap();
    graph
        .set_compute_pass_metadata(
            pass,
            RenderGraphComputePassMetadata::new(
                RenderGraphComputeShaderSource::wgsl("reduce", "@compute fn cs_main() {}"),
                "cs_main",
                vec![BindingSchemaEntry::new(
                    0,
                    "output",
                    ComputeBindingKind::StorageBufferReadWrite,
                )],
            ),
        )
        .unwrap();

    assert_eq!(
        graph.compile().unwrap_err(),
        RenderGraphError::ComputeBindingResourceNotDeclared {
            pass: "reduce".to_string(),
            binding: 0,
            resource: "output".to_string(),
            required_access: "read/write buffer",
        }
    );
}

#[test]
fn compute_metadata_texture_mip_bindings_require_a_texture_and_a_valid_transient_mip() {
    let mut valid_graph = RenderGraphBuilder::new("compute-metadata-texture-mip-valid");
    let hzb = valid_graph.create_texture(
        TextureDesc::new(
            "hzb",
            64,
            32,
            TextureFormat::Rgba16Float,
            TextureUsage::SAMPLED | TextureUsage::STORAGE,
        )
        .with_mip_levels(3),
    );
    let pass = valid_graph.add_pass("hzb-mip-two", QueueLane::AsyncCompute);
    valid_graph
        .access_texture(
            pass,
            hzb,
            RenderGraphResourceAccessKind::Write,
            RenderGraphTextureSubresourceRange::single_mip(2),
            RenderGraphResourceAccessIntent::storage_texture_write(
                RenderGraphShaderStages::COMPUTE,
            ),
            None,
        )
        .unwrap();
    valid_graph
        .set_pass_flags(
            pass,
            PassFlags {
                allow_culling: false,
                has_side_effects: true,
            },
        )
        .unwrap();
    valid_graph
        .set_compute_workload(
            pass,
            RenderGraphComputeWorkload::fixed("hzb-mip-two", [1, 1, 1], [1, 1, 1]),
        )
        .unwrap();
    valid_graph
        .set_compute_pass_metadata(
            pass,
            RenderGraphComputePassMetadata::new(
                RenderGraphComputeShaderSource::wgsl("hzb-mip-two", "@compute fn cs_main() {}"),
                "cs_main",
                vec![
                    BindingSchemaEntry::new(0, "hzb", ComputeBindingKind::StorageTextureWrite)
                        .with_texture_mip_level(2),
                ],
            ),
        )
        .unwrap();
    assert!(valid_graph.clone().compile().is_ok());

    let mut out_of_range = valid_graph;
    out_of_range
        .set_compute_pass_metadata(
            pass,
            RenderGraphComputePassMetadata::new(
                RenderGraphComputeShaderSource::wgsl("hzb-mip-three", "@compute fn cs_main() {}"),
                "cs_main",
                vec![
                    BindingSchemaEntry::new(0, "hzb", ComputeBindingKind::StorageTextureWrite)
                        .with_texture_mip_level(3),
                ],
            ),
        )
        .unwrap();
    assert_eq!(
        out_of_range.compile().unwrap_err(),
        RenderGraphError::ComputeTextureMipOutOfRange {
            pass: "hzb-mip-two".to_string(),
            binding: 0,
            resource: "hzb".to_string(),
            mip_level: 3,
            mip_levels: 3,
        }
    );

    let mut buffer_graph = RenderGraphBuilder::new("compute-metadata-texture-mip-buffer");
    let output = buffer_graph.import_present_external_resource_with_binding(
        "output",
        RenderGraphExternalResourceBinding::required_buffer(),
    );
    let pass = buffer_graph.add_pass("buffer-mip", QueueLane::AsyncCompute);
    buffer_graph.read_external(pass, output).unwrap();
    buffer_graph
        .set_pass_flags(
            pass,
            PassFlags {
                allow_culling: false,
                has_side_effects: true,
            },
        )
        .unwrap();
    buffer_graph
        .set_compute_workload(
            pass,
            RenderGraphComputeWorkload::fixed("buffer-mip", [1, 1, 1], [1, 1, 1]),
        )
        .unwrap();
    buffer_graph
        .set_compute_pass_metadata(
            pass,
            RenderGraphComputePassMetadata::new(
                RenderGraphComputeShaderSource::wgsl("buffer-mip", "@compute fn cs_main() {}"),
                "cs_main",
                vec![
                    BindingSchemaEntry::new(0, "output", ComputeBindingKind::StorageBufferRead)
                        .with_texture_mip_level(0),
                ],
            ),
        )
        .unwrap();
    assert_eq!(
        buffer_graph.compile().unwrap_err(),
        RenderGraphError::ComputeTextureMipBindingNotTexture {
            pass: "buffer-mip".to_string(),
            binding: 0,
            mip_level: 0,
        }
    );

    let mut external_graph = RenderGraphBuilder::new("compute-metadata-texture-mip-external");
    let output = external_graph.import_present_external_resource_with_binding(
        "output",
        RenderGraphExternalResourceBinding::required_texture(),
    );
    let pass = external_graph.add_pass("external-mip", QueueLane::AsyncCompute);
    external_graph.write_external(pass, output).unwrap();
    external_graph
        .set_pass_flags(
            pass,
            PassFlags {
                allow_culling: false,
                has_side_effects: true,
            },
        )
        .unwrap();
    external_graph
        .set_compute_workload(
            pass,
            RenderGraphComputeWorkload::fixed("external-mip", [1, 1, 1], [1, 1, 1]),
        )
        .unwrap();
    external_graph
        .set_compute_pass_metadata(
            pass,
            RenderGraphComputePassMetadata::new(
                RenderGraphComputeShaderSource::wgsl("external-mip", "@compute fn cs_main() {}"),
                "cs_main",
                vec![
                    BindingSchemaEntry::new(0, "output", ComputeBindingKind::StorageTextureWrite)
                        .with_texture_mip_level(0),
                ],
            ),
        )
        .unwrap();
    assert_eq!(
        external_graph.compile().unwrap_err(),
        RenderGraphError::ComputeTextureMipRequiresTransientTexture {
            pass: "external-mip".to_string(),
            binding: 0,
            resource: "output".to_string(),
            mip_level: 0,
        }
    );
}

#[test]
fn compute_metadata_buffer_ranges_require_a_buffer_binding() {
    let mut valid_graph = RenderGraphBuilder::new("compute-metadata-buffer-range-valid");
    let weights = valid_graph.import_present_external_resource_with_binding(
        "weights",
        RenderGraphExternalResourceBinding::required_buffer(),
    );
    let pass = valid_graph.add_pass("weight-slab", QueueLane::AsyncCompute);
    valid_graph.read_external(pass, weights).unwrap();
    valid_graph
        .set_pass_flags(
            pass,
            PassFlags {
                allow_culling: false,
                has_side_effects: true,
            },
        )
        .unwrap();
    valid_graph
        .set_compute_workload(
            pass,
            RenderGraphComputeWorkload::fixed("weight-slab", [1, 1, 1], [1, 1, 1]),
        )
        .unwrap();
    valid_graph
        .set_compute_pass_metadata(
            pass,
            RenderGraphComputePassMetadata::new(
                RenderGraphComputeShaderSource::wgsl("weight-slab", "@compute fn cs_main() {}"),
                "cs_main",
                vec![
                    BindingSchemaEntry::new(0, "weights", ComputeBindingKind::StorageBufferRead)
                        .with_buffer_range(256, Some(512)),
                ],
            ),
        )
        .unwrap();
    assert!(valid_graph.compile().is_ok());

    let mut texture_graph = RenderGraphBuilder::new("compute-metadata-buffer-range-texture");
    let input = texture_graph.import_present_external_resource_with_binding(
        "input",
        RenderGraphExternalResourceBinding::required_texture(),
    );
    let pass = texture_graph.add_pass("texture-offset", QueueLane::AsyncCompute);
    texture_graph.read_external(pass, input).unwrap();
    texture_graph
        .set_pass_flags(
            pass,
            PassFlags {
                allow_culling: false,
                has_side_effects: true,
            },
        )
        .unwrap();
    texture_graph
        .set_compute_workload(
            pass,
            RenderGraphComputeWorkload::fixed("texture-offset", [1, 1, 1], [1, 1, 1]),
        )
        .unwrap();
    texture_graph
        .set_compute_pass_metadata(
            pass,
            RenderGraphComputePassMetadata::new(
                RenderGraphComputeShaderSource::wgsl("texture-offset", "@compute fn cs_main() {}"),
                "cs_main",
                vec![
                    BindingSchemaEntry::new(0, "input", ComputeBindingKind::SampledTexture)
                        .with_buffer_range(256, Some(512)),
                ],
            ),
        )
        .unwrap();
    assert_eq!(
        texture_graph.compile().unwrap_err(),
        RenderGraphError::ComputeBufferRangeBindingNotBuffer {
            pass: "texture-offset".to_string(),
            binding: 0,
            offset: 256,
            size: Some(512),
        }
    );
}

#[test]
fn compute_metadata_buffer_ranges_reject_empty_and_transient_overrun() {
    let mut empty_range = RenderGraphBuilder::new("compute-metadata-empty-buffer-range");
    let params = empty_range.create_buffer(BufferDesc::new(
        "params",
        512,
        BufferUsage::UNIFORM | BufferUsage::COPY_DST,
    ));
    let pass = empty_range.add_pass("empty-range", QueueLane::AsyncCompute);
    empty_range.read_buffer(pass, params).unwrap();
    empty_range
        .mark_readback(RenderGraphResource::TransientBuffer(params))
        .unwrap();
    empty_range
        .set_compute_workload(
            pass,
            RenderGraphComputeWorkload::fixed("empty-range", [1, 1, 1], [1, 1, 1]),
        )
        .unwrap();
    empty_range
        .set_compute_pass_metadata(
            pass,
            RenderGraphComputePassMetadata::new(
                RenderGraphComputeShaderSource::wgsl("empty-range", "@compute fn cs_main() {}"),
                "cs_main",
                vec![
                    BindingSchemaEntry::new(0, "params", ComputeBindingKind::UniformBuffer)
                        .with_buffer_range(0, Some(0)),
                ],
            ),
        )
        .unwrap();
    assert_eq!(
        empty_range.compile().unwrap_err(),
        RenderGraphError::ComputeBufferBindingRangeEmpty {
            pass: "empty-range".to_string(),
            binding: 0,
            resource: "params".to_string(),
        }
    );

    let mut overrunning_range = RenderGraphBuilder::new("compute-metadata-overrun-buffer-range");
    let output = overrunning_range.create_buffer(BufferDesc::new(
        "output",
        512,
        BufferUsage::STORAGE | BufferUsage::COPY_SRC,
    ));
    let pass = overrunning_range.add_pass("overrun-range", QueueLane::AsyncCompute);
    overrunning_range.read_buffer(pass, output).unwrap();
    overrunning_range
        .mark_readback(RenderGraphResource::TransientBuffer(output))
        .unwrap();
    overrunning_range
        .set_compute_workload(
            pass,
            RenderGraphComputeWorkload::fixed("overrun-range", [1, 1, 1], [1, 1, 1]),
        )
        .unwrap();
    overrunning_range
        .set_compute_pass_metadata(
            pass,
            RenderGraphComputePassMetadata::new(
                RenderGraphComputeShaderSource::wgsl("overrun-range", "@compute fn cs_main() {}"),
                "cs_main",
                vec![
                    BindingSchemaEntry::new(0, "output", ComputeBindingKind::StorageBufferRead)
                        .with_buffer_range(256, Some(257)),
                ],
            ),
        )
        .unwrap();
    assert_eq!(
        overrunning_range.compile().unwrap_err(),
        RenderGraphError::ComputeBufferBindingRangeOutOfBounds {
            pass: "overrun-range".to_string(),
            binding: 0,
            resource: "output".to_string(),
            offset: 256,
            size: Some(257),
            buffer_size: 512,
        }
    );
}

#[test]
fn compute_metadata_buffer_bindings_require_declared_usage() {
    let mut graph = RenderGraphBuilder::new("compute-metadata-buffer-usage");
    let params = graph.create_buffer(BufferDesc::new("params", 256, BufferUsage::COPY_SRC));
    let pass = graph.add_pass("storage-read", QueueLane::AsyncCompute);
    graph.read_buffer(pass, params).unwrap();
    graph
        .mark_readback(RenderGraphResource::TransientBuffer(params))
        .unwrap();
    graph
        .set_compute_workload(
            pass,
            RenderGraphComputeWorkload::fixed("storage-read", [1, 1, 1], [1, 1, 1]),
        )
        .unwrap();
    graph
        .set_compute_pass_metadata(
            pass,
            RenderGraphComputePassMetadata::new(
                RenderGraphComputeShaderSource::wgsl("storage-read", "@compute fn cs_main() {}"),
                "cs_main",
                vec![BindingSchemaEntry::new(
                    0,
                    "params",
                    ComputeBindingKind::StorageBufferRead,
                )],
            ),
        )
        .unwrap();

    assert_eq!(
        graph.compile().unwrap_err(),
        RenderGraphError::ComputeBufferBindingUsageMissing {
            pass: "storage-read".to_string(),
            binding: 0,
            resource: "params".to_string(),
            required: BufferUsage::STORAGE,
            actual: BufferUsage::COPY_SRC,
        }
    );
}

#[test]
fn compute_metadata_rejects_empty_entry_points_and_wgsl_sources() {
    let mut empty_entry_point = RenderGraphBuilder::new("compute-metadata-empty-entry-point");
    let pass = empty_entry_point.add_pass("reduce", QueueLane::AsyncCompute);
    empty_entry_point
        .set_pass_flags(
            pass,
            PassFlags {
                allow_culling: false,
                has_side_effects: true,
            },
        )
        .unwrap();
    empty_entry_point
        .set_compute_workload(
            pass,
            RenderGraphComputeWorkload::fixed("reduce", [1, 1, 1], [1, 1, 1]),
        )
        .unwrap();
    empty_entry_point
        .set_compute_pass_metadata(
            pass,
            RenderGraphComputePassMetadata::new(
                RenderGraphComputeShaderSource::wgsl("reduce", "@compute fn cs_main() {}"),
                " ",
                Vec::new(),
            ),
        )
        .unwrap();
    assert_eq!(
        empty_entry_point.compile().unwrap_err(),
        RenderGraphError::ComputePassEntryPointEmpty {
            pass: "reduce".to_string(),
        }
    );

    let mut empty_source = RenderGraphBuilder::new("compute-metadata-empty-source");
    let pass = empty_source.add_pass("reduce", QueueLane::AsyncCompute);
    empty_source
        .set_pass_flags(
            pass,
            PassFlags {
                allow_culling: false,
                has_side_effects: true,
            },
        )
        .unwrap();
    empty_source
        .set_compute_workload(
            pass,
            RenderGraphComputeWorkload::fixed("reduce", [1, 1, 1], [1, 1, 1]),
        )
        .unwrap();
    empty_source
        .set_compute_pass_metadata(
            pass,
            RenderGraphComputePassMetadata::new(
                RenderGraphComputeShaderSource::wgsl("reduce", "\n\t"),
                "cs_main",
                Vec::new(),
            ),
        )
        .unwrap();
    assert_eq!(
        empty_source.compile().unwrap_err(),
        RenderGraphError::ComputePassShaderSourceEmpty {
            pass: "reduce".to_string(),
        }
    );
}
