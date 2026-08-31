use crate::render_graph::{
    PassFlags, QueueLane, RenderGraphAttachmentOps, RenderGraphBuilder, RenderGraphDump,
    RenderGraphError, RenderGraphResource, RenderGraphResourceAccessIntent,
    RenderGraphResourceAccessKind, RenderGraphResourceKind, RenderGraphShaderStages,
    RenderGraphTextureSubresourceRange, RgTextureHandle,
};
use crate::rhi::{BufferDesc, BufferUsage, TextureDesc, TextureFormat, TextureUsage};

#[test]
fn builder_rejects_foreign_resource_handles_before_index_lookup() {
    let mut builder = RenderGraphBuilder::new("invalid-resource-handle");
    let pass = builder.add_pass("write", QueueLane::Graphics);

    let error = builder
        .write_texture(pass, RgTextureHandle::from_index(usize::MAX, 0))
        .unwrap_err();

    assert!(matches!(
        error,
        RenderGraphError::ForeignResource {
            kind: RenderGraphResourceKind::TransientTexture,
            index: usize::MAX,
            ..
        }
    ));
}

#[test]
fn builder_resource_validation_uses_constant_time_handle_bounds() {
    let source = include_str!("../builder.rs");

    assert!(source.contains("if handle_generation != self.generation"));
    assert!(source.contains("handle.index() < self.next_texture"));
    assert!(source.contains("handle.index() < self.next_buffer"));
    assert!(source.contains("handle.index() < self.next_external_resource"));
    assert!(!source.contains("self.resources.iter().any(|node| node.resource == resource)"));
}

#[test]
fn builder_rejects_texture_view_alias_ranges_outside_the_parent_descriptor() {
    let mut builder = RenderGraphBuilder::new("invalid-texture-view-alias");
    let parent = builder.create_texture(test_color_desc().with_mip_levels(1));

    let error = builder
        .create_texture_view_alias(
            "invalid-alias",
            parent,
            RenderGraphTextureSubresourceRange::single_mip(1),
        )
        .expect_err("view aliases must be declared inside their parent mip range");

    assert!(matches!(
        error,
        RenderGraphError::TextureViewAliasRangeOutOfBounds {
            ref alias,
            ref parent_name,
            ..
        } if alias == "invalid-alias" && parent_name == "color"
    ));
}

#[test]
fn builder_preserves_duplicate_writes_by_exact_access_id_and_rejects_legacy_lookup() {
    let mut builder = RenderGraphBuilder::new("duplicate-resource-access");
    let color = builder.create_texture(
        TextureDesc::new(
            "color",
            16,
            16,
            TextureFormat::Rgba8Unorm,
            TextureUsage::STORAGE,
        )
        .with_mip_levels(2),
    );
    let pass = builder.add_pass("duplicate-write", QueueLane::Graphics);
    let write_intent =
        RenderGraphResourceAccessIntent::storage_texture_write(RenderGraphShaderStages::COMPUTE);
    builder
        .access_texture(
            pass,
            color,
            RenderGraphResourceAccessKind::Write,
            RenderGraphTextureSubresourceRange::single_mip(0),
            write_intent,
            None,
        )
        .expect("first disjoint write");
    builder
        .access_texture(
            pass,
            color,
            RenderGraphResourceAccessKind::Write,
            RenderGraphTextureSubresourceRange::single_mip(1),
            write_intent,
            None,
        )
        .expect("second disjoint write");
    builder
        .set_pass_flags(
            pass,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .expect("duplicate-write pass flags");

    let graph = builder
        .compile()
        .expect("duplicate writes remain addressable by exact compiled access IDs");
    let resource = RenderGraphResource::TransientTexture(color);

    assert!(graph.access_id_at(pass, 0).is_some());
    assert!(graph.access_id_at(pass, 1).is_some());
    assert_eq!(
        graph.access_id_for(pass, resource, RenderGraphResourceAccessKind::Write),
        None
    );
}

#[test]
fn builder_preserves_duplicate_reads_by_exact_access_id_and_rejects_legacy_lookup() {
    let mut builder = RenderGraphBuilder::new("duplicate-resource-read");
    let color = builder.create_texture(test_color_desc().with_mip_levels(2));
    let producer = builder.add_pass("producer", QueueLane::Graphics);
    let pass = builder.add_pass("duplicate-read", QueueLane::Graphics);
    builder.write_texture(producer, color).unwrap();
    let read_intent =
        RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::COMPUTE);
    builder
        .read_texture_with_access(
            pass,
            color,
            RenderGraphTextureSubresourceRange::single_mip(0),
            read_intent,
        )
        .expect("first disjoint read");
    builder
        .read_texture_with_access(
            pass,
            color,
            RenderGraphTextureSubresourceRange::single_mip(1),
            read_intent,
        )
        .expect("second disjoint read");
    builder
        .set_pass_flags(
            pass,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .expect("duplicate-read pass flags");

    let graph = builder
        .compile()
        .expect("duplicate reads remain addressable by exact compiled access IDs");
    let resource = RenderGraphResource::TransientTexture(color);

    assert!(graph.access_id_at(pass, 0).is_some());
    assert!(graph.access_id_at(pass, 1).is_some());
    assert_eq!(
        graph.access_id_for(pass, resource, RenderGraphResourceAccessKind::Read),
        None
    );
}

#[test]
fn builder_rejects_overlapping_same_kind_access_scopes_in_one_pass() {
    let mut builder = RenderGraphBuilder::new("overlapping-pass-accesses");
    let color = builder.create_texture(test_color_desc());
    let pass = builder.add_pass("overlap", QueueLane::Graphics);
    let attachment_ops = crate::render_graph::RenderGraphAttachmentOps::clear_store();

    for _ in 0..2 {
        builder
            .access_texture(
                pass,
                color,
                RenderGraphResourceAccessKind::Write,
                RenderGraphTextureSubresourceRange::single_mip(0),
                RenderGraphResourceAccessIntent::ColorAttachment,
                Some(attachment_ops),
            )
            .expect("declare the overlapping access before compile validation");
    }

    let error = builder
        .compile()
        .expect_err("same-pass writes to the same subresource have no graph ordering");

    assert!(matches!(
        error,
        RenderGraphError::OverlappingPassResourceAccessScope {
            ref pass,
            first_access: 0,
            second_access: 1,
            access: RenderGraphResourceAccessKind::Write,
            ..
        } if pass == "overlap"
    ));
}

#[test]
fn builder_preserves_distinct_read_and_write_accesses_in_one_pass() {
    let mut builder = RenderGraphBuilder::new("read-write-resource-access");
    let color = builder.create_texture(test_color_desc());
    let producer = builder.add_pass("producer", QueueLane::Graphics);
    let consumer = builder.add_pass("consumer", QueueLane::Graphics);
    builder.write_texture(producer, color).unwrap();
    builder.read_texture(consumer, color).unwrap();
    builder.write_texture(consumer, color).unwrap();
    builder
        .set_pass_flags(
            consumer,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .unwrap();

    let graph = builder
        .compile()
        .expect("same-pass read and write remain distinct access kinds");
    let resource = RenderGraphResource::TransientTexture(color);
    let read_version = graph
        .resource_version_for_access(consumer, resource, RenderGraphResourceAccessKind::Read)
        .expect("compiled read version");
    let write_version = graph
        .resource_version_for_access(consumer, resource, RenderGraphResourceAccessKind::Write)
        .expect("compiled write version");

    assert!(graph
        .pass_resource_access(consumer, resource, RenderGraphResourceAccessKind::Read)
        .is_some());
    assert!(graph
        .pass_resource_access(consumer, resource, RenderGraphResourceAccessKind::Write)
        .is_some());
    assert_eq!(read_version.ordinal(), 1);
    assert_eq!(write_version.ordinal(), 2);
}

fn test_color_desc() -> TextureDesc {
    TextureDesc::new(
        "color",
        16,
        16,
        TextureFormat::Rgba8Unorm,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    )
}

#[test]
fn explicit_texture_version_token_orders_consumer_and_reaches_compiled_access() {
    let mut builder = RenderGraphBuilder::new("explicit-version-token");
    let color = builder.create_texture(test_color_desc());
    let consumer = builder.add_pass("consumer", QueueLane::Graphics);
    let producer = builder.add_pass("producer", QueueLane::Graphics);
    let produced = builder
        .write_texture_versioned(producer, color)
        .expect("versioned texture write");
    builder
        .read_texture_from_version(consumer, produced)
        .expect("consume the declared producer value");
    builder
        .set_pass_flags(
            consumer,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .expect("consumer flags");

    let graph = builder
        .compile()
        .expect("compile explicit version dependency");
    let resource = RenderGraphResource::TransientTexture(color);
    let producer_version = graph
        .resource_version_for_access(producer, resource, RenderGraphResourceAccessKind::Write)
        .expect("compiled producer version");

    assert_eq!(
        graph
            .passes()
            .iter()
            .map(|pass| pass.name.as_str())
            .collect::<Vec<_>>(),
        vec!["producer", "consumer"]
    );
    assert_eq!(graph.passes()[1].dependencies, vec![producer]);
    assert_eq!(
        graph.input_version_for_access(consumer, resource, RenderGraphResourceAccessKind::Read),
        Some(producer_version)
    );
}

#[test]
fn explicit_version_token_preserves_a_nonzero_producer_access_ordinal() {
    let mut builder = RenderGraphBuilder::new("nonzero-producer-access-ordinal");
    let scratch = builder.create_texture(test_color_desc());
    let color = builder.create_texture(TextureDesc::new(
        "versioned-color",
        16,
        16,
        TextureFormat::Rgba8Unorm,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let consumer = builder.add_pass("consumer", QueueLane::Graphics);
    let producer = builder.add_pass("producer", QueueLane::Graphics);
    builder
        .write_texture(producer, scratch)
        .expect("first producer access");
    let produced = builder
        .write_texture_versioned(producer, color)
        .expect("second producer access creates the version token");
    builder
        .read_texture_from_version(consumer, produced)
        .expect("consumer reads the second producer access");
    builder
        .set_pass_flags(
            consumer,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .expect("consumer flags");

    let graph = builder
        .compile()
        .expect("compiler retains the nonzero producer access ordinal");
    let resource = RenderGraphResource::TransientTexture(color);
    let producer_version = graph
        .resource_version_for_access(producer, resource, RenderGraphResourceAccessKind::Write)
        .expect("compiled version for the second producer access");
    let producer_access = graph
        .access_id_at(producer, 1)
        .expect("stable identity for the second producer access");
    let consumer_access = graph
        .access_id_at(consumer, 0)
        .expect("stable identity for the consumer access");

    assert_eq!(
        graph.resource_version_for_id(producer_access),
        Some(producer_version)
    );
    assert_eq!(
        graph.input_version_for_id(consumer_access),
        Some(producer_version)
    );
    let producer_key = graph
        .versioned_access_key(producer_access)
        .expect("compiled producer binding key");
    let consumer_key = graph
        .versioned_access_key(consumer_access)
        .expect("compiled consumer binding key");
    assert_eq!(producer_key.access_id, producer_access);
    assert_eq!(producer_key.resource, resource);
    assert_eq!(producer_key.version, producer_version);
    assert_eq!(consumer_key.access_id, consumer_access);
    assert_eq!(consumer_key.resource, resource);
    assert_eq!(consumer_key.version, producer_version);

    let dump = RenderGraphDump::from_graph(&graph);
    let producer_row = dump
        .pass_rows
        .iter()
        .find(|row| row.id == producer)
        .expect("producer must remain present after explicit dependency reordering");
    let producer_access_row = producer_row
        .resources
        .get(1)
        .expect("second producer access must remain present in the dump");

    assert_eq!(
        graph
            .passes()
            .iter()
            .map(|pass| pass.id)
            .collect::<Vec<_>>(),
        vec![producer, consumer]
    );
    assert_eq!(producer_access_row.access_id, producer_access);
    assert_eq!(producer_access_row.version, producer_version.ordinal());
}

#[test]
fn loaded_write_binding_key_uses_the_version_it_produces() {
    let mut builder = RenderGraphBuilder::new("loaded-write-versioned-access-key");
    let color = builder.create_texture(TextureDesc::new(
        "scene-color",
        16,
        16,
        TextureFormat::Rgba8Unorm,
        TextureUsage::RENDER_ATTACHMENT,
    ));
    let producer = builder.add_pass("opaque", QueueLane::Graphics);
    let load = builder.add_pass("transparent", QueueLane::Graphics);
    let initial = builder
        .write_texture_with_ops_versioned(producer, color, RenderGraphAttachmentOps::clear_store())
        .expect("initial color version");
    builder
        .write_texture_with_ops_from_version(
            load,
            color,
            RenderGraphAttachmentOps::load_store(),
            initial,
        )
        .expect("loaded write creates a successor version");

    let graph = builder.compile().expect("compile loaded write");
    let producer_access = graph.access_id_at(producer, 0).expect("producer access");
    let load_access = graph.access_id_at(load, 0).expect("loaded write access");
    let initial_version = graph
        .resource_version_for_id(producer_access)
        .expect("initial compiled version");
    let loaded_write_version = graph
        .resource_version_for_id(load_access)
        .expect("loaded write compiled version");

    assert_eq!(
        graph.input_version_for_id(load_access),
        Some(initial_version)
    );
    assert_ne!(loaded_write_version, initial_version);
    assert_eq!(
        graph
            .versioned_access_key(load_access)
            .expect("loaded write binding key")
            .version,
        loaded_write_version
    );
}

#[test]
fn explicit_buffer_version_token_orders_consumer_and_reaches_compiled_access() {
    let mut builder = RenderGraphBuilder::new("explicit-buffer-version-token");
    let buffer = builder.create_buffer(BufferDesc::new(
        "visible-clusters",
        64,
        BufferUsage::STORAGE,
    ));
    let consumer = builder.add_pass("consumer", QueueLane::Graphics);
    let producer = builder.add_pass("producer", QueueLane::Graphics);
    let produced = builder
        .write_buffer_versioned(producer, buffer)
        .expect("versioned buffer write");
    builder
        .read_buffer_from_version(consumer, produced)
        .expect("consume the declared producer value");
    builder
        .set_pass_flags(
            consumer,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .expect("consumer flags");

    let graph = builder
        .compile()
        .expect("compile explicit buffer version dependency");
    let resource = RenderGraphResource::TransientBuffer(buffer);
    let producer_version = graph
        .resource_version_for_access(producer, resource, RenderGraphResourceAccessKind::Write)
        .expect("compiled producer version");

    assert_eq!(
        graph
            .passes()
            .iter()
            .map(|pass| pass.name.as_str())
            .collect::<Vec<_>>(),
        vec!["producer", "consumer"]
    );
    assert_eq!(graph.passes()[1].dependencies, vec![producer]);
    assert_eq!(
        graph.input_version_for_access(consumer, resource, RenderGraphResourceAccessKind::Read),
        Some(producer_version)
    );
}

#[test]
fn explicit_external_version_token_orders_consumer_and_reaches_compiled_access() {
    let mut builder = RenderGraphBuilder::new("explicit-external-version-token");
    let external = builder.import_present_external_resource("viewport-output");
    let consumer = builder.add_pass("consumer", QueueLane::Graphics);
    let producer = builder.add_pass("producer", QueueLane::Graphics);
    let produced = builder
        .write_external_versioned(producer, external)
        .expect("versioned external write");
    builder
        .read_external_from_version(consumer, produced)
        .expect("consume the declared producer value");
    builder
        .set_pass_flags(
            consumer,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .expect("consumer flags");

    let graph = builder
        .compile()
        .expect("compile explicit external version dependency");
    let resource = RenderGraphResource::External(external);
    let producer_version = graph
        .resource_version_for_access(producer, resource, RenderGraphResourceAccessKind::Write)
        .expect("compiled producer version");

    assert_eq!(
        graph
            .passes()
            .iter()
            .map(|pass| pass.name.as_str())
            .collect::<Vec<_>>(),
        vec!["producer", "consumer"]
    );
    assert_eq!(graph.passes()[1].dependencies, vec![producer]);
    assert_eq!(
        graph.input_version_for_access(consumer, resource, RenderGraphResourceAccessKind::Read),
        Some(producer_version)
    );
}

#[test]
fn explicit_texture_version_token_rejects_a_value_replaced_before_its_consumer() {
    let mut builder = RenderGraphBuilder::new("stale-version-token");
    let color = builder.create_texture(test_color_desc());
    let producer = builder.add_pass("producer", QueueLane::Graphics);
    let replacement = builder.add_pass("replacement", QueueLane::Graphics);
    let consumer = builder.add_pass("consumer", QueueLane::Graphics);
    let produced = builder
        .write_texture_versioned(producer, color)
        .expect("versioned texture write");
    builder
        .write_texture(replacement, color)
        .expect("replacement write");
    builder
        .read_texture_from_version(consumer, produced)
        .expect("declare the stale producer value for validation");
    builder
        .set_pass_flags(
            consumer,
            PassFlags {
                has_side_effects: true,
                ..PassFlags::default()
            },
        )
        .expect("consumer flags");

    let error = builder
        .compile()
        .expect_err("a replaced version must not silently resolve to latest");

    assert!(matches!(
        error,
        RenderGraphError::ResourceVersionNotCurrent {
            ref pass,
            ref resource,
            ref producer,
            ref latest_producer,
        } if pass == "consumer"
            && resource == "color"
            && producer == "producer"
            && latest_producer == "replacement"
    ));
}

#[test]
fn version_tokens_cannot_cross_render_graph_builder_generations() {
    let mut source = RenderGraphBuilder::new("source-version-token");
    let source_color = source.create_texture(test_color_desc());
    let source_pass = source.add_pass("source", QueueLane::Graphics);
    let produced = source
        .write_texture_versioned(source_pass, source_color)
        .expect("source versioned write");

    let mut destination = RenderGraphBuilder::new("destination-version-token");
    let destination_pass = destination.add_pass("destination", QueueLane::Graphics);
    let error = destination
        .read_texture_from_version(destination_pass, produced)
        .expect_err("foreign version token must fail before access registration");

    assert!(matches!(
        error,
        RenderGraphError::ForeignResourceVersion { .. }
    ));
}

#[test]
fn external_version_tokens_cannot_cross_render_graph_builder_generations() {
    let mut source = RenderGraphBuilder::new("source-external-version-token");
    let source_external = source.import_present_external_resource("source-output");
    let source_pass = source.add_pass("source", QueueLane::Graphics);
    let produced = source
        .write_external_versioned(source_pass, source_external)
        .expect("source versioned external write");

    let mut destination = RenderGraphBuilder::new("destination-external-version-token");
    let destination_pass = destination.add_pass("destination", QueueLane::Graphics);
    let error = destination
        .read_external_from_version(destination_pass, produced)
        .expect_err("foreign external version token must fail before access registration");

    assert!(matches!(
        error,
        RenderGraphError::ForeignResourceVersion { .. }
    ));
}
