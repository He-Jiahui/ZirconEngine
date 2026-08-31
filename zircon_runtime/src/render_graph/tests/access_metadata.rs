use crate::render_graph::{
    PassFlags, QueueLane, RenderGraphBufferRange, RenderGraphBuilder, RenderGraphError,
    RenderGraphResource, RenderGraphResourceAccessIntent, RenderGraphResourceAccessKind,
    RenderGraphResourceAccessRange, RenderGraphShaderStages, RenderGraphTextureSubresourceRange,
};
use crate::rhi::{BufferDesc, BufferUsage, TextureDesc, TextureFormat, TextureUsage};

#[test]
fn compiled_access_metadata_retains_a_texture_mip_range_and_shader_intent() {
    let mut builder = RenderGraphBuilder::new("compiled-texture-access-metadata");
    let texture = builder.create_texture(
        TextureDesc::new(
            "depth-pyramid",
            64,
            64,
            TextureFormat::R32Float,
            TextureUsage::SAMPLED | TextureUsage::STORAGE,
        )
        .with_mip_levels(4),
    );
    let producer = builder.add_pass("producer", QueueLane::AsyncCompute);
    let consumer = builder.add_pass("consumer", QueueLane::AsyncCompute);
    let range = RenderGraphTextureSubresourceRange::single_mip(2);
    let compiled_range = RenderGraphTextureSubresourceRange {
        base_mip_level: 2,
        mip_level_count: Some(1),
        base_array_layer: 0,
        array_layer_count: Some(1),
        aspect: crate::render_graph::RenderGraphTextureAspect::All,
    };
    let intent = RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::COMPUTE);
    let write_intent =
        RenderGraphResourceAccessIntent::storage_texture_write(RenderGraphShaderStages::COMPUTE);

    builder
        .access_texture(
            producer,
            texture,
            RenderGraphResourceAccessKind::Write,
            range,
            write_intent,
            None,
        )
        .unwrap();
    builder
        .read_texture_with_access(consumer, texture, range, intent)
        .unwrap();
    builder
        .set_pass_flags(
            consumer,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();

    let graph = builder.compile().unwrap();
    let resource = RenderGraphResource::TransientTexture(texture);
    let access_id = graph
        .access_id_for(consumer, resource, RenderGraphResourceAccessKind::Read)
        .expect("compiled access id");
    let metadata = graph
        .access_metadata(access_id)
        .expect("compiled access metadata");
    let producer_access_id = graph
        .access_id_for(producer, resource, RenderGraphResourceAccessKind::Write)
        .expect("compiled producer access id");
    let producer_metadata = graph
        .access_metadata(producer_access_id)
        .expect("compiled producer metadata");

    assert_eq!(
        metadata.range,
        RenderGraphResourceAccessRange::Texture(compiled_range)
    );
    assert_eq!(metadata.intent, intent);
    assert_eq!(
        producer_metadata.range,
        RenderGraphResourceAccessRange::Texture(compiled_range)
    );
    assert_eq!(producer_metadata.intent, write_intent);
}

#[test]
fn compiled_access_metadata_retains_a_buffer_byte_window() {
    let mut builder = RenderGraphBuilder::new("compiled-buffer-access-metadata");
    let buffer = builder.create_buffer(BufferDesc::new(
        "visible-clusters",
        128,
        BufferUsage::STORAGE,
    ));
    let producer = builder.add_pass("producer", QueueLane::AsyncCompute);
    let consumer = builder.add_pass("consumer", QueueLane::AsyncCompute);
    let range = RenderGraphBufferRange::new(32, Some(64));
    let intent =
        RenderGraphResourceAccessIntent::storage_buffer_read(RenderGraphShaderStages::COMPUTE);
    let write_intent = RenderGraphResourceAccessIntent::storage_buffer_read_write(
        RenderGraphShaderStages::COMPUTE,
    );

    builder
        .access_buffer(
            producer,
            buffer,
            RenderGraphResourceAccessKind::Write,
            range,
            write_intent,
        )
        .unwrap();
    builder
        .read_buffer_with_access(consumer, buffer, range, intent)
        .unwrap();
    builder
        .set_pass_flags(
            consumer,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();

    let graph = builder.compile().unwrap();
    let resource = RenderGraphResource::TransientBuffer(buffer);
    let access_id = graph
        .access_id_for(consumer, resource, RenderGraphResourceAccessKind::Read)
        .expect("compiled access id");
    let metadata = graph
        .access_metadata(access_id)
        .expect("compiled access metadata");
    let producer_access_id = graph
        .access_id_for(producer, resource, RenderGraphResourceAccessKind::Write)
        .expect("compiled producer access id");
    let producer_metadata = graph
        .access_metadata(producer_access_id)
        .expect("compiled producer metadata");

    assert_eq!(
        metadata.range,
        RenderGraphResourceAccessRange::Buffer(range)
    );
    assert_eq!(metadata.intent, intent);
    assert_eq!(
        producer_metadata.range,
        RenderGraphResourceAccessRange::Buffer(range)
    );
    assert_eq!(producer_metadata.intent, write_intent);
}

#[test]
fn compiled_access_metadata_resolves_full_texture_scope_to_finite_counts() {
    let mut builder = RenderGraphBuilder::new("compiled-full-texture-scope");
    let texture = builder.create_texture(
        TextureDesc::new(
            "scene-color",
            64,
            32,
            TextureFormat::Rgba16Float,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        )
        .with_mip_levels(4),
    );
    let pass = builder.add_pass("produce", QueueLane::Graphics);
    builder.write_texture(pass, texture).unwrap();
    builder
        .set_pass_flags(
            pass,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();

    let graph = builder.compile().expect("compile full texture access");
    let access = graph
        .access_id_for(
            pass,
            RenderGraphResource::TransientTexture(texture),
            RenderGraphResourceAccessKind::Write,
        )
        .expect("texture access id");

    assert_eq!(
        graph.versioned_access_key(access).unwrap().range,
        RenderGraphResourceAccessRange::Texture(RenderGraphTextureSubresourceRange {
            base_mip_level: 0,
            mip_level_count: Some(4),
            base_array_layer: 0,
            array_layer_count: Some(1),
            aspect: crate::render_graph::RenderGraphTextureAspect::All,
        })
    );
}

#[test]
fn compiled_access_metadata_resolves_full_buffer_scope_to_finite_window() {
    let mut builder = RenderGraphBuilder::new("compiled-full-buffer-scope");
    let buffer = builder.create_buffer(BufferDesc::new(
        "visible-clusters",
        128,
        BufferUsage::STORAGE,
    ));
    let pass = builder.add_pass("produce", QueueLane::AsyncCompute);
    builder.write_buffer(pass, buffer).unwrap();
    builder
        .set_pass_flags(
            pass,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();

    let graph = builder.compile().expect("compile full buffer access");
    let access = graph
        .access_id_for(
            pass,
            RenderGraphResource::TransientBuffer(buffer),
            RenderGraphResourceAccessKind::Write,
        )
        .expect("buffer access id");

    assert_eq!(
        graph.versioned_access_key(access).unwrap().range,
        RenderGraphResourceAccessRange::Buffer(RenderGraphBufferRange::new(0, Some(128)))
    );
}

#[test]
fn compiled_access_metadata_keeps_the_authoring_access_id_after_pass_reordering() {
    let mut builder = RenderGraphBuilder::new("reordered-access-metadata");
    let texture = builder.create_texture(TextureDesc::new(
        "depth-pyramid",
        64,
        64,
        TextureFormat::R32Float,
        TextureUsage::SAMPLED | TextureUsage::STORAGE,
    ));
    let consumer = builder.add_pass("consumer", QueueLane::AsyncCompute);
    let producer = builder.add_pass("producer", QueueLane::AsyncCompute);
    let range = RenderGraphTextureSubresourceRange::single_mip(0);
    let compiled_range = RenderGraphTextureSubresourceRange {
        base_mip_level: 0,
        mip_level_count: Some(1),
        base_array_layer: 0,
        array_layer_count: Some(1),
        aspect: crate::render_graph::RenderGraphTextureAspect::All,
    };
    let intent = RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::COMPUTE);

    builder
        .read_texture_with_access(consumer, texture, range, intent)
        .unwrap();
    builder
        .access_texture(
            producer,
            texture,
            RenderGraphResourceAccessKind::Write,
            range,
            RenderGraphResourceAccessIntent::storage_texture_write(
                RenderGraphShaderStages::COMPUTE,
            ),
            None,
        )
        .unwrap();
    builder.add_dependency(producer, consumer).unwrap();
    builder
        .set_pass_flags(
            consumer,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();

    let graph = builder.compile().unwrap();
    let resource = RenderGraphResource::TransientTexture(texture);
    let access_id = graph
        .access_id_for(consumer, resource, RenderGraphResourceAccessKind::Read)
        .expect("compiled consumer access id");
    let metadata = graph
        .access_metadata(access_id)
        .expect("compiled consumer metadata");

    assert_eq!(
        graph
            .passes()
            .iter()
            .map(|pass| pass.name.as_str())
            .collect::<Vec<_>>(),
        vec!["producer", "consumer"]
    );
    assert_eq!(access_id.pass(), consumer);
    assert_eq!(access_id.access_index(), 0);
    assert_eq!(
        metadata.range,
        RenderGraphResourceAccessRange::Texture(compiled_range)
    );
    assert_eq!(metadata.intent, intent);
}

#[test]
fn builder_rejects_an_access_range_outside_the_texture_description() {
    let mut builder = RenderGraphBuilder::new("texture-access-range-validation");
    let texture = builder.create_texture(
        TextureDesc::new(
            "scene-color",
            32,
            32,
            TextureFormat::Rgba16Float,
            TextureUsage::SAMPLED | TextureUsage::STORAGE,
        )
        .with_mip_levels(2),
    );
    let producer = builder.add_pass("producer", QueueLane::AsyncCompute);
    let consumer = builder.add_pass("consumer", QueueLane::AsyncCompute);

    builder.write_storage_texture(producer, texture).unwrap();
    builder
        .read_texture_with_access(
            consumer,
            texture,
            RenderGraphTextureSubresourceRange::single_mip(2),
            RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::COMPUTE),
        )
        .unwrap();
    builder
        .set_pass_flags(
            consumer,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();

    let error = builder
        .compile()
        .expect_err("mip-range validation must reject an unavailable mip");

    assert!(matches!(
        error,
        RenderGraphError::TextureAccessMipRangeOutOfBounds {
            ref pass,
            ref resource,
            base_mip_level: 2,
            ..
        } if pass == "consumer" && resource == "scene-color"
    ));
}

#[test]
fn builder_rejects_a_write_only_intent_on_a_read_access() {
    let mut builder = RenderGraphBuilder::new("access-intent-kind-validation");
    let texture = builder.create_texture(TextureDesc::new(
        "storage-output",
        32,
        32,
        TextureFormat::Rgba16Float,
        TextureUsage::STORAGE,
    ));
    let producer = builder.add_pass("producer", QueueLane::AsyncCompute);
    let consumer = builder.add_pass("consumer", QueueLane::AsyncCompute);

    builder.write_storage_texture(producer, texture).unwrap();
    builder
        .read_texture_with_access(
            consumer,
            texture,
            RenderGraphTextureSubresourceRange::full(),
            RenderGraphResourceAccessIntent::storage_texture_write(
                RenderGraphShaderStages::COMPUTE,
            ),
        )
        .unwrap();
    builder
        .set_pass_flags(
            consumer,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();

    let error = builder
        .compile()
        .expect_err("a write-only storage intent cannot be declared as a graph read");

    assert!(matches!(
        error,
        RenderGraphError::ResourceAccessIntentKindMismatch {
            ref pass,
            ref resource,
            ..
        } if pass == "consumer" && resource == "storage-output"
    ));
}

#[test]
fn builder_rejects_an_empty_buffer_access_range() {
    let mut builder = RenderGraphBuilder::new("buffer-access-range-validation");
    let buffer = builder.create_buffer(BufferDesc::new("cluster-counts", 64, BufferUsage::STORAGE));
    let producer = builder.add_pass("producer", QueueLane::AsyncCompute);
    let consumer = builder.add_pass("consumer", QueueLane::AsyncCompute);

    builder.write_buffer(producer, buffer).unwrap();
    builder
        .read_buffer_with_access(
            consumer,
            buffer,
            RenderGraphBufferRange::new(0, Some(0)),
            RenderGraphResourceAccessIntent::storage_buffer_read(RenderGraphShaderStages::COMPUTE),
        )
        .unwrap();
    builder
        .set_pass_flags(
            consumer,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();

    let error = builder
        .compile()
        .expect_err("an empty buffer window cannot reach dependency inference");

    assert!(matches!(
        error,
        RenderGraphError::BufferAccessRangeEmpty {
            ref pass,
            ref resource,
        } if pass == "consumer" && resource == "cluster-counts"
    ));
}

#[test]
fn builder_rejects_a_sampled_texture_intent_without_sampled_usage() {
    let mut builder = RenderGraphBuilder::new("sampled-texture-intent-validation");
    let texture = builder.create_texture(TextureDesc::new(
        "storage-only",
        32,
        32,
        TextureFormat::Rgba16Float,
        TextureUsage::STORAGE,
    ));
    let producer = builder.add_pass("producer", QueueLane::AsyncCompute);
    let consumer = builder.add_pass("consumer", QueueLane::AsyncCompute);

    builder.write_storage_texture(producer, texture).unwrap();
    builder
        .read_texture_with_access(
            consumer,
            texture,
            RenderGraphTextureSubresourceRange::full(),
            RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::COMPUTE),
        )
        .unwrap();
    builder
        .set_pass_flags(
            consumer,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();

    let error = builder
        .compile()
        .expect_err("sampled intent must validate the texture usage contract");

    assert!(matches!(
        error,
        RenderGraphError::TextureAccessIntentUsageMissing {
            ref pass,
            ref resource,
            ..
        } if pass == "consumer" && resource == "storage-only"
    ));
}

#[test]
fn builder_rejects_a_typed_intent_on_a_report_only_external_resource() {
    let mut builder = RenderGraphBuilder::new("external-access-intent-validation");
    let external = builder.import_external_resource("report-only-output");
    let pass = builder.add_pass("consumer", QueueLane::Graphics);

    builder
        .read_external_with_access(
            pass,
            external,
            RenderGraphResourceAccessRange::UnresolvedExternal,
            RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::FRAGMENT),
        )
        .unwrap();

    let error = builder
        .compile()
        .expect_err("report-only external resources cannot claim typed intent");

    assert!(matches!(
        error,
        RenderGraphError::UnresolvedExternalAccessMetadata {
            ref pass,
            ref resource,
        } if pass == "consumer" && resource == "report-only-output"
    ));
}

#[test]
fn builder_rejects_shader_visible_intent_without_shader_stages() {
    let mut builder = RenderGraphBuilder::new("empty-shader-stage-validation");
    let texture = builder.create_texture(TextureDesc::new(
        "scene-color",
        32,
        32,
        TextureFormat::Rgba16Float,
        TextureUsage::SAMPLED | TextureUsage::STORAGE,
    ));
    let producer = builder.add_pass("producer", QueueLane::AsyncCompute);
    let consumer = builder.add_pass("consumer", QueueLane::AsyncCompute);

    builder.write_storage_texture(producer, texture).unwrap();
    builder
        .read_texture_with_access(
            consumer,
            texture,
            RenderGraphTextureSubresourceRange::full(),
            RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::NONE),
        )
        .unwrap();

    let error = builder
        .compile()
        .expect_err("shader-visible intent requires a stage");

    assert!(matches!(
        error,
        RenderGraphError::ResourceAccessIntentShaderStagesEmpty {
            ref pass,
            ref resource,
            ..
        } if pass == "consumer" && resource == "scene-color"
    ));
}

#[test]
fn builder_rejects_texture_intent_for_a_buffer_access() {
    let mut builder = RenderGraphBuilder::new("buffer-texture-intent-validation");
    let buffer = builder.create_buffer(BufferDesc::new(
        "visible-clusters",
        64,
        BufferUsage::STORAGE,
    ));
    let producer = builder.add_pass("producer", QueueLane::AsyncCompute);
    let consumer = builder.add_pass("consumer", QueueLane::AsyncCompute);

    builder.write_buffer(producer, buffer).unwrap();
    builder
        .read_buffer_with_access(
            consumer,
            buffer,
            RenderGraphBufferRange::full(),
            RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::COMPUTE),
        )
        .unwrap();

    let error = builder
        .compile()
        .expect_err("sampled texture intent cannot target a buffer");

    assert!(matches!(
        error,
        RenderGraphError::ResourceAccessIntentRequiresTexture {
            ref pass,
            ref resource,
            ..
        } if pass == "consumer" && resource == "visible-clusters"
    ));
}

#[test]
fn builder_rejects_uniform_buffer_intent_without_uniform_usage() {
    let mut builder = RenderGraphBuilder::new("uniform-buffer-intent-validation");
    let buffer = builder.create_buffer(BufferDesc::new(
        "visible-clusters",
        64,
        BufferUsage::STORAGE,
    ));
    let producer = builder.add_pass("producer", QueueLane::AsyncCompute);
    let consumer = builder.add_pass("consumer", QueueLane::AsyncCompute);

    builder.write_buffer(producer, buffer).unwrap();
    builder
        .read_buffer_with_access(
            consumer,
            buffer,
            RenderGraphBufferRange::full(),
            RenderGraphResourceAccessIntent::UniformBuffer {
                stages: RenderGraphShaderStages::COMPUTE,
            },
        )
        .unwrap();

    let error = builder
        .compile()
        .expect_err("uniform buffer intent must validate buffer usage");

    assert!(matches!(
        error,
        RenderGraphError::BufferAccessIntentUsageMissing {
            ref pass,
            ref resource,
            ..
        } if pass == "consumer" && resource == "visible-clusters"
    ));
}

#[test]
fn builder_rejects_copy_source_and_destination_without_copy_usage() {
    let mut source_builder = RenderGraphBuilder::new("copy-source-intent-validation");
    let source = source_builder.create_texture(TextureDesc::new(
        "source-color",
        32,
        32,
        TextureFormat::Rgba16Float,
        TextureUsage::STORAGE,
    ));
    let producer = source_builder.add_pass("producer", QueueLane::AsyncCompute);
    let consumer = source_builder.add_pass("consumer", QueueLane::AsyncCopy);
    source_builder
        .write_storage_texture(producer, source)
        .unwrap();
    source_builder
        .access_texture(
            consumer,
            source,
            RenderGraphResourceAccessKind::Read,
            RenderGraphTextureSubresourceRange::full(),
            RenderGraphResourceAccessIntent::CopySource,
            None,
        )
        .unwrap();

    let source_error = source_builder
        .compile()
        .expect_err("copy source must validate texture copy-source usage");
    assert!(matches!(
        source_error,
        RenderGraphError::TextureAccessIntentUsageMissing {
            ref pass,
            ref resource,
            ..
        } if pass == "consumer" && resource == "source-color"
    ));

    let mut destination_builder = RenderGraphBuilder::new("copy-destination-intent-validation");
    let destination =
        destination_builder.create_buffer(BufferDesc::new("copy-target", 64, BufferUsage::STORAGE));
    let pass = destination_builder.add_pass("copy", QueueLane::AsyncCopy);
    destination_builder
        .access_buffer(
            pass,
            destination,
            RenderGraphResourceAccessKind::Write,
            RenderGraphBufferRange::full(),
            RenderGraphResourceAccessIntent::CopyDestination,
        )
        .unwrap();

    let destination_error = destination_builder
        .compile()
        .expect_err("copy destination must validate buffer copy-destination usage");
    assert!(matches!(
        destination_error,
        RenderGraphError::BufferAccessIntentUsageMissing {
            ref pass,
            ref resource,
            ..
        } if pass == "copy" && resource == "copy-target"
    ));
}

#[test]
fn versioned_external_buffer_accesses_preserve_exact_scopes_and_provenance() {
    let mut builder = RenderGraphBuilder::new("versioned-external-buffer-access");
    let buffer = builder.import_present_external_buffer_with_binding(
        "exposure.current",
        BufferDesc::new("exposure.current", 16, BufferUsage::STORAGE),
        crate::render_graph::RenderGraphExternalResourceBinding::report_only_buffer(),
    );
    let writer = builder.add_pass("exposure-resolve", QueueLane::AsyncCompute);
    let reader = builder.add_pass("scene-composite", QueueLane::Graphics);
    let range = RenderGraphResourceAccessRange::Buffer(RenderGraphBufferRange::full());
    let write_intent = RenderGraphResourceAccessIntent::storage_buffer_read_write(
        RenderGraphShaderStages::COMPUTE,
    );
    let read_intent =
        RenderGraphResourceAccessIntent::storage_buffer_read(RenderGraphShaderStages::FRAGMENT);

    let version = builder
        .write_external_with_access_versioned(writer, buffer, range, write_intent, None)
        .unwrap();
    builder
        .read_external_with_access_from_version(reader, version, range, read_intent)
        .unwrap();
    builder
        .set_pass_flags(
            reader,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();

    let graph = builder.compile().unwrap();
    let resource = RenderGraphResource::External(buffer);
    let write_access = graph
        .access_id_for(writer, resource, RenderGraphResourceAccessKind::Write)
        .unwrap();
    let read_access = graph
        .access_id_for(reader, resource, RenderGraphResourceAccessKind::Read)
        .unwrap();
    let compiled_range =
        RenderGraphResourceAccessRange::Buffer(RenderGraphBufferRange::new(0, Some(16)));

    assert_eq!(
        graph.access_metadata(write_access).unwrap().range,
        compiled_range
    );
    assert_eq!(
        graph.access_metadata(write_access).unwrap().intent,
        write_intent
    );
    assert_eq!(
        graph.access_metadata(read_access).unwrap().range,
        compiled_range
    );
    assert_eq!(
        graph.access_metadata(read_access).unwrap().intent,
        read_intent
    );
    assert!(graph.input_version_for_id(read_access).is_some());
    assert!(graph
        .external_access_packet()
        .access(write_access)
        .is_some());
    assert!(graph.external_access_packet().access(read_access).is_some());
}
