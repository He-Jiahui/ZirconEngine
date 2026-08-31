use crate::render_graph::{
    PassFlags, QueueLane, RenderGraphBufferRange, RenderGraphBuilder, RenderGraphError,
    RenderGraphExternalResourceBinding, RenderGraphResourceAccessIntent,
    RenderGraphResourceAccessKind, RenderGraphResourceAccessRange, RenderGraphShaderStages,
    RenderGraphTextureAspect, RenderGraphTextureSubresourceRange,
};
use crate::rhi::{BufferDesc, BufferUsage, TextureDesc, TextureFormat, TextureUsage};

mod texture_aspects;
mod texture_view_alias_roots;
mod typed_external_roots;

fn non_cullable() -> PassFlags {
    PassFlags {
        has_side_effects: true,
        ..PassFlags::default()
    }
}

fn two_mip_storage_texture() -> TextureDesc {
    TextureDesc::new(
        "mip-chain",
        32,
        32,
        TextureFormat::Rgba16Float,
        TextureUsage::STORAGE | TextureUsage::SAMPLED,
    )
    .with_mip_levels(2)
}

fn two_mip_range() -> RenderGraphTextureSubresourceRange {
    RenderGraphTextureSubresourceRange {
        base_mip_level: 0,
        mip_level_count: Some(2),
        base_array_layer: 0,
        array_layer_count: None,
        aspect: RenderGraphTextureAspect::All,
    }
}

#[test]
fn disjoint_texture_writes_do_not_create_a_waw_dependency() {
    let mut builder = RenderGraphBuilder::new("disjoint-texture-writes");
    let texture = builder.create_texture(two_mip_storage_texture());
    let first = builder.add_pass("write-mip-zero", QueueLane::AsyncCompute);
    let second = builder.add_pass("write-mip-one", QueueLane::AsyncCompute);
    let write_intent =
        RenderGraphResourceAccessIntent::storage_texture_write(RenderGraphShaderStages::COMPUTE);

    builder
        .access_texture(
            first,
            texture,
            RenderGraphResourceAccessKind::Write,
            RenderGraphTextureSubresourceRange::single_mip(0),
            write_intent,
            None,
        )
        .expect("write the first mip");
    builder
        .access_texture(
            second,
            texture,
            RenderGraphResourceAccessKind::Write,
            RenderGraphTextureSubresourceRange::single_mip(1),
            write_intent,
            None,
        )
        .expect("write the second mip");
    builder
        .set_pass_flags(first, non_cullable())
        .expect("first pass flags");
    builder
        .set_pass_flags(second, non_cullable())
        .expect("second pass flags");

    let graph = builder
        .compile()
        .expect("disjoint subresources must compile independently");

    assert_eq!(graph.passes()[0].id, first);
    assert_eq!(graph.passes()[1].id, second);
    assert!(graph.passes()[1].dependencies.is_empty());
}

#[test]
fn overlapping_texture_writes_keep_a_waw_dependency() {
    let mut builder = RenderGraphBuilder::new("overlapping-texture-writes");
    let texture = builder.create_texture(two_mip_storage_texture());
    let first = builder.add_pass("write-mip-chain", QueueLane::AsyncCompute);
    let second = builder.add_pass("rewrite-mip-one", QueueLane::AsyncCompute);
    let write_intent =
        RenderGraphResourceAccessIntent::storage_texture_write(RenderGraphShaderStages::COMPUTE);

    builder
        .access_texture(
            first,
            texture,
            RenderGraphResourceAccessKind::Write,
            two_mip_range(),
            write_intent,
            None,
        )
        .expect("write both mips");
    builder
        .access_texture(
            second,
            texture,
            RenderGraphResourceAccessKind::Write,
            RenderGraphTextureSubresourceRange::single_mip(1),
            write_intent,
            None,
        )
        .expect("rewrite the second mip");
    builder
        .set_pass_flags(first, non_cullable())
        .expect("first pass flags");
    builder
        .set_pass_flags(second, non_cullable())
        .expect("second pass flags");

    let graph = builder
        .compile()
        .expect("overlapping subresources must compile with a hazard edge");

    assert_eq!(graph.passes()[1].dependencies, vec![first]);
}

#[test]
fn texture_view_alias_accesses_share_parent_subresource_hazards() {
    let mut builder = RenderGraphBuilder::new("texture-view-alias-hazards");
    let parent = builder.create_texture(two_mip_storage_texture());
    let alias = builder
        .create_texture_view_alias(
            "mip-one-view",
            parent,
            RenderGraphTextureSubresourceRange::single_mip(1),
        )
        .expect("declare the parent mip view");
    let output = builder.import_external_resource("viewport-output");
    let write_parent = builder.add_pass("write-parent", QueueLane::Graphics);
    let write_alias = builder.add_pass("write-alias", QueueLane::Graphics);
    let read_parent_mip = builder.add_pass("read-parent-mip", QueueLane::Graphics);

    builder.write_texture(write_parent, parent).unwrap();
    builder.write_texture(write_alias, alias).unwrap();
    builder
        .read_texture_with_access(
            read_parent_mip,
            parent,
            RenderGraphTextureSubresourceRange::single_mip(1),
            RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::FRAGMENT),
        )
        .expect("read parent mip one");
    builder.write_external(read_parent_mip, output).unwrap();

    let graph = builder
        .compile()
        .expect("alias access must participate in parent mip hazards");
    let alias_pass = graph
        .passes()
        .iter()
        .find(|pass| pass.id == write_alias)
        .expect("alias writer pass");
    let reader_pass = graph
        .passes()
        .iter()
        .find(|pass| pass.id == read_parent_mip)
        .expect("parent reader pass");

    assert!(alias_pass.dependencies.contains(&write_parent));
    assert!(reader_pass.dependencies.contains(&write_alias));
}

#[test]
fn persistent_texture_cull_root_keeps_every_disjoint_final_writer() {
    let mut builder = RenderGraphBuilder::new("persistent-subresource-cull-root");
    let texture = builder.create_texture(two_mip_storage_texture());
    builder
        .mark_persistent(texture)
        .expect("mark the texture as a cull root");
    let first = builder.add_pass("write-mip-zero", QueueLane::AsyncCompute);
    let second = builder.add_pass("write-mip-one", QueueLane::AsyncCompute);
    let write_intent =
        RenderGraphResourceAccessIntent::storage_texture_write(RenderGraphShaderStages::COMPUTE);

    builder
        .access_texture(
            first,
            texture,
            RenderGraphResourceAccessKind::Write,
            RenderGraphTextureSubresourceRange::single_mip(0),
            write_intent,
            None,
        )
        .expect("write the first persistent mip");
    builder
        .access_texture(
            second,
            texture,
            RenderGraphResourceAccessKind::Write,
            RenderGraphTextureSubresourceRange::single_mip(1),
            write_intent,
            None,
        )
        .expect("write the second persistent mip");

    let graph = builder
        .compile()
        .expect("each final subresource writer must remain reachable from the root");

    assert!(graph.passes().iter().all(|pass| !pass.culled));
}

#[test]
fn persistent_texture_view_alias_cull_root_keeps_only_its_parent_scope() {
    let mut builder = RenderGraphBuilder::new("persistent-texture-view-alias-cull-root");
    let parent = builder.create_texture(two_mip_storage_texture());
    let alias = builder
        .create_texture_view_alias(
            "mip-one-view",
            parent,
            RenderGraphTextureSubresourceRange::single_mip(1),
        )
        .expect("declare the second-mip view");
    builder
        .mark_persistent(alias)
        .expect("the view is the only culling root");
    let write_parent_mip_zero = builder.add_pass("write-parent-mip-zero", QueueLane::AsyncCompute);
    let write_alias_mip_one = builder.add_pass("write-alias-mip-one", QueueLane::AsyncCompute);
    let write_intent =
        RenderGraphResourceAccessIntent::storage_texture_write(RenderGraphShaderStages::COMPUTE);

    builder
        .access_texture(
            write_parent_mip_zero,
            parent,
            RenderGraphResourceAccessKind::Write,
            RenderGraphTextureSubresourceRange::single_mip(0),
            write_intent,
            None,
        )
        .expect("write the unrooted parent mip");
    builder
        .access_texture(
            write_alias_mip_one,
            alias,
            RenderGraphResourceAccessKind::Write,
            RenderGraphTextureSubresourceRange::full(),
            write_intent,
            None,
        )
        .expect("write the rooted alias scope");

    let graph = builder
        .compile()
        .expect("the alias root must cull only its own parent scope");
    let parent_writer = graph
        .passes()
        .iter()
        .find(|pass| pass.id == write_parent_mip_zero)
        .expect("parent writer pass");
    let alias_writer = graph
        .passes()
        .iter()
        .find(|pass| pass.id == write_alias_mip_one)
        .expect("alias writer pass");

    assert!(parent_writer.culled);
    assert!(!alias_writer.culled);
}

#[test]
fn descriptor_backed_external_texture_scopes_do_not_globally_order_disjoint_mips() {
    let mut builder = RenderGraphBuilder::new("typed-external-texture-subresource-hazards");
    let texture = builder.import_external_texture_with_binding(
        "history-color",
        two_mip_storage_texture(),
        RenderGraphExternalResourceBinding::required_texture(),
    );
    let first = builder.add_pass("write-history-mip-zero", QueueLane::AsyncCompute);
    let second = builder.add_pass("write-history-mip-one", QueueLane::AsyncCompute);
    let write_intent =
        RenderGraphResourceAccessIntent::storage_texture_write(RenderGraphShaderStages::COMPUTE);

    builder
        .access_external(
            first,
            texture,
            RenderGraphResourceAccessKind::Write,
            RenderGraphResourceAccessRange::Texture(
                RenderGraphTextureSubresourceRange::single_mip(0),
            ),
            write_intent,
            None,
        )
        .expect("write external mip zero");
    builder
        .access_external(
            second,
            texture,
            RenderGraphResourceAccessKind::Write,
            RenderGraphResourceAccessRange::Texture(
                RenderGraphTextureSubresourceRange::single_mip(1),
            ),
            write_intent,
            None,
        )
        .expect("write external mip one");
    builder
        .set_pass_flags(first, non_cullable())
        .expect("first pass flags");
    builder
        .set_pass_flags(second, non_cullable())
        .expect("second pass flags");

    let graph = builder
        .compile()
        .expect("typed external mips must retain exact scopes");

    assert!(graph.passes()[1].dependencies.is_empty());
}

#[test]
fn descriptor_backed_present_external_texture_cull_root_uses_its_full_descriptor_scope() {
    let mut builder = RenderGraphBuilder::new("typed-present-external-texture-cull-root");
    let texture = builder.import_present_external_texture_with_binding(
        "present-history-color",
        two_mip_storage_texture(),
        RenderGraphExternalResourceBinding::required_texture(),
    );
    let write_mip_zero = builder.add_pass("write-present-mip-zero", QueueLane::AsyncCompute);
    let write_mip_one = builder.add_pass("write-present-mip-one", QueueLane::AsyncCompute);
    let write_intent =
        RenderGraphResourceAccessIntent::storage_texture_write(RenderGraphShaderStages::COMPUTE);

    for (pass, range) in [
        (
            write_mip_zero,
            RenderGraphTextureSubresourceRange::single_mip(0),
        ),
        (
            write_mip_one,
            RenderGraphTextureSubresourceRange::single_mip(1),
        ),
    ] {
        builder
            .access_external(
                pass,
                texture,
                RenderGraphResourceAccessKind::Write,
                RenderGraphResourceAccessRange::Texture(range),
                write_intent,
                None,
            )
            .expect("write one present external mip");
    }

    let graph = builder
        .compile()
        .expect("the typed present root must cover its full descriptor");

    assert!(graph.passes().iter().all(|pass| !pass.culled));
}

#[test]
fn descriptor_backed_external_buffer_scopes_do_not_globally_order_adjacent_ranges() {
    let mut builder = RenderGraphBuilder::new("typed-external-buffer-subresource-hazards");
    let buffer = builder.import_external_buffer_with_binding(
        "history-worklist",
        BufferDesc::new("history-worklist", 128, BufferUsage::STORAGE),
        RenderGraphExternalResourceBinding::required_buffer(),
    );
    let first = builder.add_pass("write-history-first-window", QueueLane::AsyncCompute);
    let second = builder.add_pass("write-history-second-window", QueueLane::AsyncCompute);
    let write_intent = RenderGraphResourceAccessIntent::storage_buffer_read_write(
        RenderGraphShaderStages::COMPUTE,
    );

    builder
        .access_external(
            first,
            buffer,
            RenderGraphResourceAccessKind::Write,
            RenderGraphResourceAccessRange::Buffer(RenderGraphBufferRange::new(0, Some(32))),
            write_intent,
            None,
        )
        .expect("write the first external window");
    builder
        .access_external(
            second,
            buffer,
            RenderGraphResourceAccessKind::Write,
            RenderGraphResourceAccessRange::Buffer(RenderGraphBufferRange::new(32, Some(32))),
            write_intent,
            None,
        )
        .expect("write the adjacent external window");
    builder
        .set_pass_flags(first, non_cullable())
        .expect("first pass flags");
    builder
        .set_pass_flags(second, non_cullable())
        .expect("second pass flags");

    let graph = builder
        .compile()
        .expect("typed external buffer ranges must retain exact scopes");

    assert!(graph.passes()[1].dependencies.is_empty());
}

#[test]
fn disjoint_buffer_writes_do_not_create_a_waw_dependency() {
    let mut builder = RenderGraphBuilder::new("disjoint-buffer-writes");
    let buffer = builder.create_buffer(BufferDesc::new("worklist", 128, BufferUsage::STORAGE));
    let first = builder.add_pass("write-first-window", QueueLane::AsyncCompute);
    let second = builder.add_pass("write-second-window", QueueLane::AsyncCompute);
    let write_intent = RenderGraphResourceAccessIntent::storage_buffer_read_write(
        RenderGraphShaderStages::COMPUTE,
    );

    builder
        .access_buffer(
            first,
            buffer,
            RenderGraphResourceAccessKind::Write,
            RenderGraphBufferRange::new(0, Some(32)),
            write_intent,
        )
        .expect("write first buffer window");
    builder
        .access_buffer(
            second,
            buffer,
            RenderGraphResourceAccessKind::Write,
            RenderGraphBufferRange::new(64, Some(32)),
            write_intent,
        )
        .expect("write second buffer window");
    builder
        .set_pass_flags(first, non_cullable())
        .expect("first pass flags");
    builder
        .set_pass_flags(second, non_cullable())
        .expect("second pass flags");

    let graph = builder
        .compile()
        .expect("disjoint buffer windows must compile independently");

    assert!(graph.passes()[1].dependencies.is_empty());
}

#[test]
fn adjacent_buffer_windows_do_not_overlap_across_or_within_passes() {
    let write_intent = RenderGraphResourceAccessIntent::storage_buffer_read_write(
        RenderGraphShaderStages::COMPUTE,
    );

    let mut cross_pass_builder = RenderGraphBuilder::new("adjacent-buffer-writes-cross-pass");
    let cross_pass_buffer =
        cross_pass_builder.create_buffer(BufferDesc::new("worklist", 128, BufferUsage::STORAGE));
    let first = cross_pass_builder.add_pass("write-first-window", QueueLane::AsyncCompute);
    let second = cross_pass_builder.add_pass("write-second-window", QueueLane::AsyncCompute);
    cross_pass_builder
        .access_buffer(
            first,
            cross_pass_buffer,
            RenderGraphResourceAccessKind::Write,
            RenderGraphBufferRange::new(0, Some(32)),
            write_intent,
        )
        .expect("write the first adjacent range");
    cross_pass_builder
        .access_buffer(
            second,
            cross_pass_buffer,
            RenderGraphResourceAccessKind::Write,
            RenderGraphBufferRange::new(32, Some(32)),
            write_intent,
        )
        .expect("write the second adjacent range");
    cross_pass_builder
        .set_pass_flags(second, non_cullable())
        .expect("cross-pass cull root");
    let cross_pass_graph = cross_pass_builder
        .compile()
        .expect("adjacent half-open buffer ranges must not create a WAW edge");
    assert!(cross_pass_graph.passes()[1].dependencies.is_empty());

    let mut same_pass_builder = RenderGraphBuilder::new("adjacent-buffer-writes-same-pass");
    let same_pass_buffer =
        same_pass_builder.create_buffer(BufferDesc::new("worklist", 128, BufferUsage::STORAGE));
    let pass = same_pass_builder.add_pass("write-adjacent-windows", QueueLane::AsyncCompute);
    for range in [
        RenderGraphBufferRange::new(0, Some(32)),
        RenderGraphBufferRange::new(32, Some(32)),
    ] {
        same_pass_builder
            .access_buffer(
                pass,
                same_pass_buffer,
                RenderGraphResourceAccessKind::Write,
                range,
                write_intent,
            )
            .expect("declare an adjacent same-pass range");
    }
    same_pass_builder
        .set_pass_flags(pass, non_cullable())
        .expect("same-pass cull root");
    let same_pass_graph = same_pass_builder
        .compile()
        .expect("adjacent same-pass ranges must not conflict");
    assert!(same_pass_graph.access_id_at(pass, 0).is_some());
    assert!(same_pass_graph.access_id_at(pass, 1).is_some());
}

#[test]
fn overlapping_buffer_writes_keep_a_waw_dependency() {
    let mut builder = RenderGraphBuilder::new("overlapping-buffer-writes");
    let buffer = builder.create_buffer(BufferDesc::new("worklist", 128, BufferUsage::STORAGE));
    let first = builder.add_pass("write-first-window", QueueLane::AsyncCompute);
    let second = builder.add_pass("rewrite-overlap", QueueLane::AsyncCompute);
    let write_intent = RenderGraphResourceAccessIntent::storage_buffer_read_write(
        RenderGraphShaderStages::COMPUTE,
    );

    builder
        .access_buffer(
            first,
            buffer,
            RenderGraphResourceAccessKind::Write,
            RenderGraphBufferRange::new(0, Some(64)),
            write_intent,
        )
        .expect("write the first buffer window");
    builder
        .access_buffer(
            second,
            buffer,
            RenderGraphResourceAccessKind::Write,
            RenderGraphBufferRange::new(32, Some(64)),
            write_intent,
        )
        .expect("rewrite an overlapping buffer window");
    builder
        .set_pass_flags(first, non_cullable())
        .expect("first pass flags");
    builder
        .set_pass_flags(second, non_cullable())
        .expect("second pass flags");

    let graph = builder
        .compile()
        .expect("overlapping buffer windows must keep their hazard edge");

    assert_eq!(graph.passes()[1].dependencies, vec![first]);
}

#[test]
fn versioned_texture_read_rejects_a_scope_not_fully_covered_by_its_producer() {
    let mut builder = RenderGraphBuilder::new("partial-versioned-texture-read");
    let texture = builder.create_texture(two_mip_storage_texture());
    let producer = builder.add_pass("write-mip-zero", QueueLane::AsyncCompute);
    let consumer = builder.add_pass("read-mip-one", QueueLane::AsyncCompute);
    let producer_version = builder
        .write_texture_with_access_versioned(
            producer,
            texture,
            RenderGraphTextureSubresourceRange::single_mip(0),
            RenderGraphResourceAccessIntent::storage_texture_write(
                RenderGraphShaderStages::COMPUTE,
            ),
            None,
        )
        .expect("produce the first mip version");
    builder
        .read_texture_with_access_from_version(
            consumer,
            producer_version,
            RenderGraphTextureSubresourceRange::single_mip(1),
            RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::COMPUTE),
        )
        .expect("declare the consumer scope before compile-time coverage validation");
    builder
        .set_pass_flags(consumer, non_cullable())
        .expect("consumer pass flags");

    let error = builder
        .compile()
        .expect_err("a version token cannot provide a mip it did not produce");

    assert!(matches!(
        error,
        RenderGraphError::ResourceVersionScopeNotCovered {
            ref pass,
            ref resource,
            ref producer,
            ..
        } if pass == "read-mip-one" && resource == "mip-chain" && producer == "write-mip-zero"
    ));
}

#[test]
fn versioned_texture_read_accepts_the_exact_producer_scope() {
    let mut builder = RenderGraphBuilder::new("exact-versioned-texture-read");
    let texture = builder.create_texture(two_mip_storage_texture());
    let producer = builder.add_pass("write-mip-zero", QueueLane::AsyncCompute);
    let consumer = builder.add_pass("read-mip-zero", QueueLane::AsyncCompute);
    let producer_version = builder
        .write_texture_with_access_versioned(
            producer,
            texture,
            RenderGraphTextureSubresourceRange::single_mip(0),
            RenderGraphResourceAccessIntent::storage_texture_write(
                RenderGraphShaderStages::COMPUTE,
            ),
            None,
        )
        .expect("produce the exact texture scope");
    builder
        .read_texture_with_access_from_version(
            consumer,
            producer_version,
            RenderGraphTextureSubresourceRange::single_mip(0),
            RenderGraphResourceAccessIntent::sampled_texture(RenderGraphShaderStages::COMPUTE),
        )
        .expect("consume the exact texture scope");
    builder
        .set_pass_flags(consumer, non_cullable())
        .expect("consumer cull root");

    let graph = builder
        .compile()
        .expect("exact producer texture scope must resolve");
    let producer_access = graph.access_id_at(producer, 0).expect("producer access id");
    let consumer_access = graph.access_id_at(consumer, 0).expect("consumer access id");
    let produced = graph
        .resource_version_for_id(producer_access)
        .expect("producer version");

    assert_eq!(graph.passes()[1].dependencies, vec![producer]);
    assert_eq!(graph.input_version_for_id(consumer_access), Some(produced));
}

#[test]
fn versioned_buffer_read_rejects_a_range_not_fully_covered_by_its_producer() {
    let mut builder = RenderGraphBuilder::new("partial-versioned-buffer-read");
    let buffer = builder.create_buffer(BufferDesc::new("worklist", 128, BufferUsage::STORAGE));
    let producer = builder.add_pass("write-first-window", QueueLane::AsyncCompute);
    let consumer = builder.add_pass("read-second-window", QueueLane::AsyncCompute);
    let producer_version = builder
        .write_buffer_with_access_versioned(
            producer,
            buffer,
            RenderGraphBufferRange::new(0, Some(32)),
            RenderGraphResourceAccessIntent::storage_buffer_read_write(
                RenderGraphShaderStages::COMPUTE,
            ),
        )
        .expect("produce the first buffer window version");
    builder
        .read_buffer_with_access_from_version(
            consumer,
            producer_version,
            RenderGraphBufferRange::new(64, Some(32)),
            RenderGraphResourceAccessIntent::storage_buffer_read(RenderGraphShaderStages::COMPUTE),
        )
        .expect("declare the consumer range before compile-time coverage validation");
    builder
        .set_pass_flags(consumer, non_cullable())
        .expect("consumer pass flags");

    let error = builder
        .compile()
        .expect_err("a version token cannot provide bytes it did not produce");

    assert!(matches!(
        error,
        RenderGraphError::ResourceVersionScopeNotCovered {
            ref pass,
            ref resource,
            ref producer,
        } if pass == "read-second-window" && resource == "worklist" && producer == "write-first-window"
    ));
}

#[test]
fn versioned_buffer_read_accepts_the_exact_producer_range() {
    let mut builder = RenderGraphBuilder::new("exact-versioned-buffer-read");
    let buffer = builder.create_buffer(BufferDesc::new("worklist", 128, BufferUsage::STORAGE));
    let producer = builder.add_pass("write-first-window", QueueLane::AsyncCompute);
    let consumer = builder.add_pass("read-first-window", QueueLane::AsyncCompute);
    let producer_version = builder
        .write_buffer_with_access_versioned(
            producer,
            buffer,
            RenderGraphBufferRange::new(0, Some(32)),
            RenderGraphResourceAccessIntent::storage_buffer_read_write(
                RenderGraphShaderStages::COMPUTE,
            ),
        )
        .expect("produce the exact buffer range");
    builder
        .read_buffer_with_access_from_version(
            consumer,
            producer_version,
            RenderGraphBufferRange::new(0, Some(32)),
            RenderGraphResourceAccessIntent::storage_buffer_read(RenderGraphShaderStages::COMPUTE),
        )
        .expect("consume the exact buffer range");
    builder
        .set_pass_flags(consumer, non_cullable())
        .expect("consumer cull root");

    let graph = builder
        .compile()
        .expect("exact producer buffer range must resolve");
    let producer_access = graph.access_id_at(producer, 0).expect("producer access id");
    let consumer_access = graph.access_id_at(consumer, 0).expect("consumer access id");
    let produced = graph
        .resource_version_for_id(producer_access)
        .expect("producer version");

    assert_eq!(graph.passes()[1].dependencies, vec![producer]);
    assert_eq!(graph.input_version_for_id(consumer_access), Some(produced));
}

#[test]
fn legacy_version_token_reports_the_writer_that_replaced_its_scope() {
    let mut builder = RenderGraphBuilder::new("legacy-version-token-scope-replacement");
    let texture = builder.create_texture(two_mip_storage_texture());
    let producer = builder.add_pass("write-full-chain", QueueLane::AsyncCompute);
    let replacement = builder.add_pass("rewrite-second-mip", QueueLane::AsyncCompute);
    let consumer = builder.add_pass("consume-legacy-token", QueueLane::AsyncCompute);
    let producer_version = builder
        .write_storage_texture_versioned(producer, texture)
        .expect("produce the legacy full texture version");
    builder
        .access_texture(
            replacement,
            texture,
            RenderGraphResourceAccessKind::Write,
            RenderGraphTextureSubresourceRange::single_mip(1),
            RenderGraphResourceAccessIntent::storage_texture_write(
                RenderGraphShaderStages::COMPUTE,
            ),
            None,
        )
        .expect("replace only the second mip");
    builder
        .read_texture_from_version(consumer, producer_version)
        .expect("declare legacy token consumption");
    builder
        .set_pass_flags(consumer, non_cullable())
        .expect("consumer cull root");

    let error = builder
        .compile()
        .expect_err("the legacy token must not identify the original writer as current");

    assert!(matches!(
        error,
        RenderGraphError::ResourceVersionNotCurrent {
            ref pass,
            ref resource,
            ref producer,
            ref latest_producer,
        } if pass == "consume-legacy-token"
            && resource == "mip-chain"
            && producer == "write-full-chain"
            && latest_producer == "rewrite-second-mip"
    ));
}
