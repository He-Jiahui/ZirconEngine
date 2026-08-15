use crate::render_graph::{
    BindingSchemaEntry, ComputeBindingKind, PassFlags, QueueLane, RenderGraphBuilder,
    RenderGraphComputeDispatchExtent, RenderGraphComputePassMetadata,
    RenderGraphComputeShaderSource, RenderGraphComputeWorkload, RenderGraphError,
    RenderGraphExternalResourceBinding,
};
use crate::rhi::{TextureDesc, TextureFormat, TextureUsage};

#[test]
fn compute_dispatch_from_buffer_requires_a_declared_buffer_read() {
    let mut graph = RenderGraphBuilder::new("compute-dispatch-indirect");
    let _indirect = graph.import_external_resource_with_binding(
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
    let indirect = valid_graph.import_external_resource_with_binding(
        "dispatch-args",
        RenderGraphExternalResourceBinding::required_buffer(),
    );
    let pass = valid_graph.add_pass("reduce", QueueLane::AsyncCompute);
    valid_graph.read_external(pass, indirect).unwrap();
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
                    offset: 0,
                },
            ),
        )
        .unwrap();

    assert!(valid_graph.compile().is_ok());
}

#[test]
fn compute_dispatch_per_pixel_requires_a_declared_texture() {
    let mut graph = RenderGraphBuilder::new("compute-dispatch-per-pixel");
    let _output = graph.import_external_resource_with_binding(
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
    let output = valid_graph.import_external_resource_with_binding(
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

    assert!(valid_graph.compile().is_ok());
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
    let output = graph.import_external_resource_with_binding(
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
    valid_graph.write_storage_texture(pass, hzb).unwrap();
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
    let output = buffer_graph.import_external_resource_with_binding(
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
    let output = external_graph.import_external_resource_with_binding(
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
fn compute_metadata_buffer_offsets_require_a_buffer_binding() {
    let mut valid_graph = RenderGraphBuilder::new("compute-metadata-buffer-offset-valid");
    let weights = valid_graph.import_external_resource_with_binding(
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
                        .with_buffer_offset(256),
                ],
            ),
        )
        .unwrap();
    assert!(valid_graph.compile().is_ok());

    let mut texture_graph = RenderGraphBuilder::new("compute-metadata-buffer-offset-texture");
    let input = texture_graph.import_external_resource_with_binding(
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
                        .with_buffer_offset(256),
                ],
            ),
        )
        .unwrap();
    assert_eq!(
        texture_graph.compile().unwrap_err(),
        RenderGraphError::ComputeBufferOffsetBindingNotBuffer {
            pass: "texture-offset".to_string(),
            binding: 0,
            offset: 256,
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
