use super::*;

#[test]
fn transient_allocation_indexes_buckets_slots_and_reservations() {
    let graph_source = include_str!("../../graph.rs");
    let allocation_source = include_str!("../../graph/transient_allocation.rs");

    assert!(graph_source.contains("mod transient_allocation;"));
    assert!(graph_source.contains("build_transient_allocation_plan"));
    assert!(allocation_source.contains("HashMap::<TransientAllocationBucketKey"));
    assert!(allocation_source.contains("BTreeSet::<(usize, usize)>::new()"));
    assert!(allocation_source.contains("CompiledRenderGraphTransientAllocationId,"));
    assert!(
        !allocation_source.contains("slot_last_passes\n            .iter()\n            .position")
    );
    assert!(!allocation_source
        .contains("lifetimes_by_bucket\n            .iter_mut()\n            .find"));
}

#[test]
fn transient_materialization_uses_compiler_allocation_ids_not_bucket_hashes() {
    let source = include_str!(
        "../../../graphics/scene/scene_renderer/graph_execution/transient_materialization.rs"
    );

    assert!(source.contains("allocation_id: CompiledRenderGraphTransientAllocationId"));
    assert!(source.contains("TransientMaterializationSlotKey::from_allocation"));
    assert!(source.contains("validate_transient_allocation_intervals"));
    assert!(!source.contains("if let Some(desc) = compatible_texture_slot_desc"));
    assert!(!source.contains("bucket_key_hash"));
}

#[test]
fn graph_builds_transient_aliasing_plan_for_non_overlapping_lifetimes() {
    let mut builder = RenderGraphBuilder::new("aliasing");
    let history = builder.create_texture(TextureDesc::new(
        "history",
        16,
        16,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let lighting = builder.create_texture(TextureDesc::new(
        "lighting",
        16,
        16,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let resolved = builder.create_texture(TextureDesc::new(
        "resolved",
        16,
        16,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let output = builder.import_present_external_resource("viewport-output");

    let write_history = builder.add_pass("write-history", QueueLane::Graphics);
    let light = builder.add_pass("lighting", QueueLane::Graphics);
    let resolve = builder.add_pass("resolve", QueueLane::Graphics);
    let present = builder.add_pass("present", QueueLane::Graphics);

    builder.write_texture(write_history, history).unwrap();
    builder.read_texture(light, history).unwrap();
    builder.write_texture(light, lighting).unwrap();
    builder.read_texture(resolve, lighting).unwrap();
    builder.write_texture(resolve, resolved).unwrap();
    builder.read_texture(present, resolved).unwrap();
    builder.write_external(present, output).unwrap();

    let graph = builder.compile().unwrap();
    let plan: &crate::render_graph::CompiledRenderGraphTransientAllocationPlan =
        graph.transient_allocation_plan();
    let same_plan: &crate::render_graph::CompiledRenderGraphTransientAllocationPlan =
        graph.transient_allocation_plan();

    assert!(std::ptr::eq(plan, same_plan));
    assert_eq!(plan.texture_slot_count, 2);
    assert_eq!(plan.buffer_slot_count, 0);
    assert_eq!(plan.slot_for("history"), Some(0));
    assert_eq!(plan.slot_for("lighting"), Some(1));
    assert_eq!(plan.slot_for("resolved"), Some(0));
    assert_eq!(plan.slot_for("viewport-output"), None);
    let history_allocation = plan
        .allocations
        .iter()
        .find(|allocation| allocation.resource_name == "history")
        .expect("history allocation");
    let resolved_allocation = plan
        .allocations
        .iter()
        .find(|allocation| allocation.resource_name == "resolved")
        .expect("resolved allocation");
    assert_eq!(
        history_allocation.allocation_id,
        resolved_allocation.allocation_id
    );
    assert!(history_allocation.last_pass < resolved_allocation.first_pass);
    plan.validate_transient_allocation_intervals()
        .expect("compiler plan proves shared allocation intervals are disjoint");
    for allocation in &plan.allocations {
        assert_eq!(
            graph
                .resource_lifetime(allocation.resource)
                .map(|lifetime| lifetime.name.as_str()),
            Some(allocation.resource_name.as_str())
        );
        let lifetime = graph
            .resource_lifetime(allocation.resource)
            .expect("allocation lifetime");
        assert_eq!(allocation.first_pass, lifetime.first_pass);
        assert_eq!(allocation.last_pass, lifetime.last_pass);
    }
}

#[test]
fn graph_transient_allocation_plan_bypasses_persistent_textures() {
    let mut builder = RenderGraphBuilder::new("persistent-bypass");
    let history = builder.create_texture(TextureDesc::new(
        "history.current.scene-color",
        16,
        16,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    builder.mark_persistent(history).unwrap();

    let write_history = builder.add_pass("write-history", QueueLane::Graphics);
    builder.write_texture(write_history, history).unwrap();

    let graph = builder.compile().unwrap();
    let plan = graph.transient_allocation_plan();

    assert_eq!(plan.slot_for("history.current.scene-color"), None);
    assert_eq!(plan.texture_slot_count, 0);
    assert!(
        graph
            .resource_lifetime_by_name("history.current.scene-color")
            .unwrap()
            .usage
            .persistent
    );
}

#[test]
fn graph_declared_texture_view_alias_keeps_its_logical_lifetime_without_a_physical_slot() {
    let mut builder = RenderGraphBuilder::new("texture-view-alias");
    let pyramid = builder.create_texture(
        TextureDesc::new(
            "reflection-pyramid",
            64,
            32,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        )
        .with_mip_levels(3),
    );
    let coarse = builder
        .create_texture_view_alias(
            "reflection-pyramid-coarse-view",
            pyramid,
            crate::render_graph::RenderGraphTextureSubresourceRange::single_mip(1),
        )
        .expect("alias declaration should be valid for the parent mip range");
    let output = builder.import_present_external_resource("viewport-output");
    let build_coarse = builder.add_pass("build-coarse", QueueLane::Graphics);
    let present = builder.add_pass("present", QueueLane::Graphics);

    builder.write_texture(build_coarse, coarse).unwrap();
    builder.read_texture(present, coarse).unwrap();
    builder.write_external(present, output).unwrap();

    let graph = builder
        .compile()
        .expect("compile declared texture view alias");
    let alias = graph
        .resource_lifetime_by_name("reflection-pyramid-coarse-view")
        .expect("live alias lifetime");
    let view = alias
        .texture_view_alias
        .expect("alias lifetime retains its graph declaration");

    assert_eq!(view.parent, pyramid);
    assert_eq!(
        view.range,
        crate::render_graph::RenderGraphTextureSubresourceRange::single_mip(1)
    );
    assert_eq!(
        graph
            .transient_allocation_plan()
            .slot_for("reflection-pyramid"),
        Some(0)
    );
    assert_eq!(
        graph
            .transient_allocation_plan()
            .slot_for("reflection-pyramid-coarse-view"),
        None
    );
}

#[test]
fn graph_physical_allocation_identity_projects_texture_view_aliases_to_parent_backing() {
    let mut builder = RenderGraphBuilder::new("physical-allocation-identity");
    let pyramid = builder.create_texture(
        TextureDesc::new(
            "reflection-pyramid",
            64,
            32,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        )
        .with_mip_levels(3),
    );
    let coarse = builder
        .create_texture_view_alias(
            "reflection-pyramid-coarse-view",
            pyramid,
            crate::render_graph::RenderGraphTextureSubresourceRange::single_mip(1),
        )
        .expect("alias declaration should be valid for the parent mip range");
    let output = builder.import_present_external_resource("viewport-output");
    let build_coarse = builder.add_pass("build-coarse", QueueLane::Graphics);
    let present = builder.add_pass("present", QueueLane::Graphics);

    builder.write_texture(build_coarse, coarse).unwrap();
    builder.read_texture(present, coarse).unwrap();
    builder.write_external(present, output).unwrap();

    let graph = builder.compile().expect("compile texture view alias graph");
    let parent_resource = RenderGraphResource::TransientTexture(pyramid);
    let alias_resource = RenderGraphResource::TransientTexture(coarse);
    let alias_access = graph
        .access_id_for(
            build_coarse,
            alias_resource,
            RenderGraphResourceAccessKind::Write,
        )
        .expect("alias write access id");
    let present_access = graph
        .access_id_for(
            present,
            RenderGraphResource::External(output),
            RenderGraphResourceAccessKind::Write,
        )
        .expect("present access id");

    let parent_identity = graph
        .physical_allocation_id_for_resource(parent_resource)
        .expect("parent transient physical identity");
    assert_eq!(
        graph.physical_allocation_id_for_resource(alias_resource),
        Some(parent_identity)
    );
    assert_eq!(
        graph.physical_allocation_id_for_access(alias_access),
        Some(parent_identity)
    );
    assert_eq!(
        graph.physical_allocation_id_for_access(present_access),
        None
    );
    assert_eq!(
        Some(parent_identity.allocation_id()),
        graph
            .transient_allocation_plan()
            .allocation_id_for("reflection-pyramid")
    );
}

#[test]
fn compiled_access_allocation_table_is_dense_and_keeps_external_leases_unresolved() {
    let mut builder = RenderGraphBuilder::new("compiled-access-allocation-table");
    let pyramid = builder.create_texture(
        TextureDesc::new(
            "reflection-pyramid",
            64,
            32,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        )
        .with_mip_levels(3),
    );
    let coarse = builder
        .create_texture_view_alias(
            "reflection-pyramid-coarse-view",
            pyramid,
            crate::render_graph::RenderGraphTextureSubresourceRange::single_mip(1),
        )
        .expect("alias declaration should be valid for the parent mip range");
    let output = builder.import_present_external_resource("viewport-output");
    let build_coarse = builder.add_pass("build-coarse", QueueLane::Graphics);
    let present = builder.add_pass("present", QueueLane::Graphics);

    builder.write_texture(build_coarse, coarse).unwrap();
    builder.read_texture(present, coarse).unwrap();
    builder.write_external(present, output).unwrap();

    let graph = builder
        .compile()
        .expect("compile access allocation table graph");
    let alias_write = graph
        .access_id_for(
            build_coarse,
            RenderGraphResource::TransientTexture(coarse),
            RenderGraphResourceAccessKind::Write,
        )
        .expect("alias write access id");
    let present_write = graph
        .access_id_for(
            present,
            RenderGraphResource::External(output),
            RenderGraphResourceAccessKind::Write,
        )
        .expect("present access id");

    let bindings = graph.access_allocation_bindings();
    assert_eq!(bindings.len(), 3);
    let alias_binding = graph
        .access_allocation_binding(alias_write)
        .expect("alias access allocation binding");
    assert_eq!(
        alias_binding.key,
        graph.versioned_access_key(alias_write).unwrap()
    );
    assert_eq!(
        alias_binding.key.range,
        crate::render_graph::RenderGraphResourceAccessRange::Texture(
            crate::render_graph::RenderGraphTextureSubresourceRange {
                base_mip_level: 1,
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: Some(1),
                aspect: crate::render_graph::RenderGraphTextureAspect::All,
            }
        )
    );
    assert_eq!(
        alias_binding.physical_allocation,
        graph.physical_allocation_id_for_access(alias_write)
    );
    assert!(alias_binding.physical_allocation.is_some());
    let present_binding = graph
        .access_allocation_binding(present_write)
        .expect("present access allocation binding");
    assert_eq!(
        present_binding.key,
        graph.versioned_access_key(present_write).unwrap()
    );
    assert_eq!(present_binding.physical_allocation, None);
}

#[test]
fn persistent_texture_view_alias_has_no_transient_physical_allocation_identity() {
    let mut builder = RenderGraphBuilder::new("persistent-alias-allocation-identity");
    let history = builder.create_texture(
        TextureDesc::new(
            "history",
            64,
            32,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        )
        .with_mip_levels(2),
    );
    builder.mark_persistent(history).unwrap();
    let history_mip = builder
        .create_texture_view_alias(
            "history-mip",
            history,
            crate::render_graph::RenderGraphTextureSubresourceRange::single_mip(1),
        )
        .expect("persistent texture alias declaration should be valid");
    let output = builder.import_present_external_resource("viewport-output");
    let produce = builder.add_pass("produce", QueueLane::Graphics);
    let present = builder.add_pass("present", QueueLane::Graphics);

    builder.write_texture(produce, history_mip).unwrap();
    builder.read_texture(present, history_mip).unwrap();
    builder.write_external(present, output).unwrap();

    let graph = builder.compile().expect("compile persistent alias graph");
    let persistent_alias_access = graph
        .access_id_for(
            produce,
            RenderGraphResource::TransientTexture(history_mip),
            RenderGraphResourceAccessKind::Write,
        )
        .expect("persistent alias write access id");
    assert_eq!(
        graph.physical_allocation_id_for_resource(RenderGraphResource::TransientTexture(history)),
        None
    );
    assert_eq!(
        graph.physical_allocation_id_for_resource(RenderGraphResource::TransientTexture(
            history_mip
        )),
        None
    );
    assert_eq!(
        graph
            .access_allocation_binding(persistent_alias_access)
            .map(|binding| binding.physical_allocation),
        Some(None)
    );
}

#[test]
fn graph_readback_lifetimes_extend_to_graph_end_and_do_not_alias() {
    let mut builder = RenderGraphBuilder::new("readback-lifetime");
    let first = builder.create_buffer(BufferDesc::new("readback.first", 64, BufferUsage::STORAGE));
    let second =
        builder.create_buffer(BufferDesc::new("readback.second", 64, BufferUsage::STORAGE));
    builder
        .mark_readback(RenderGraphResource::TransientBuffer(first))
        .unwrap();
    builder
        .mark_readback(RenderGraphResource::TransientBuffer(second))
        .unwrap();

    let write_first = builder.add_pass("write-first", QueueLane::AsyncCompute);
    let write_second = builder.add_pass("write-second", QueueLane::AsyncCompute);
    builder.write_buffer(write_first, first).unwrap();
    builder.write_buffer(write_second, second).unwrap();
    builder.add_dependency(write_second, write_first).unwrap();

    let graph = builder.compile().unwrap();
    let plan = graph.transient_allocation_plan();
    let graph_last_pass = graph.passes().len() - 1;

    assert_eq!(
        graph
            .resource_lifetime_by_name("readback.first")
            .unwrap()
            .last_pass,
        graph_last_pass,
    );
    assert_eq!(
        graph
            .resource_lifetime_by_name("readback.second")
            .unwrap()
            .last_pass,
        graph_last_pass,
    );
    assert_ne!(
        plan.slot_for("readback.first"),
        plan.slot_for("readback.second"),
    );
    assert_eq!(plan.buffer_slot_count, 2);
}

#[test]
fn graph_transient_allocation_plan_reports_slot_reserved_bytes() {
    let mut builder = RenderGraphBuilder::new("byte-aware-aliasing");
    let large_color = builder.create_texture(TextureDesc::new(
        "large-color",
        16,
        16,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let small_color = builder.create_texture(TextureDesc::new(
        "small-color",
        8,
        8,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
    ));
    let small_buffer =
        builder.create_buffer(BufferDesc::new("small-buffer", 64, BufferUsage::STORAGE));
    let large_buffer =
        builder.create_buffer(BufferDesc::new("large-buffer", 128, BufferUsage::STORAGE));
    let sparse_pages = builder.create_texture(
        TextureDesc::new(
            "sparse-pages",
            32,
            32,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::SAMPLED | TextureUsage::STORAGE,
        )
        .with_sparse_residency(),
    );
    let output = builder.import_present_external_resource("viewport-output");

    let write_large = builder.add_pass("write-large", QueueLane::Graphics);
    let present_large = builder.add_pass("present-large", QueueLane::Graphics);
    let write_small = builder.add_pass("write-small", QueueLane::Graphics);
    let present_small = builder.add_pass("present-small", QueueLane::Graphics);
    let update_sparse = builder.add_pass("update-sparse", QueueLane::AsyncCompute);
    let present_sparse = builder.add_pass("present-sparse", QueueLane::Graphics);

    builder.write_texture(write_large, large_color).unwrap();
    builder.write_buffer(write_large, small_buffer).unwrap();
    builder.read_texture(present_large, large_color).unwrap();
    builder.read_buffer(present_large, small_buffer).unwrap();
    builder.write_texture(write_small, small_color).unwrap();
    builder.write_buffer(write_small, large_buffer).unwrap();
    builder.read_texture(present_small, small_color).unwrap();
    builder.read_buffer(present_small, large_buffer).unwrap();
    builder
        .write_storage_texture(update_sparse, sparse_pages)
        .unwrap();
    builder.read_texture(present_sparse, sparse_pages).unwrap();
    builder.write_external(present_sparse, output).unwrap();
    builder
        .add_dependency(present_large, present_small)
        .unwrap();
    builder.add_dependency(present_large, write_small).unwrap();
    builder
        .add_dependency(present_small, update_sparse)
        .unwrap();
    builder
        .add_dependency(present_small, present_sparse)
        .unwrap();

    let graph = builder.compile().unwrap();
    let plan = graph.transient_allocation_plan();

    assert_eq!(plan.texture_slot_count, 2);
    assert_eq!(plan.buffer_slot_count, 2);
    assert_eq!(plan.sparse_texture_slot_count, 1);
    assert_eq!(plan.slot_for("large-color"), Some(0));
    assert_eq!(plan.slot_for("small-color"), Some(0));
    assert_eq!(plan.slot_for("small-buffer"), Some(0));
    assert_eq!(plan.slot_for("large-buffer"), Some(0));
    assert_eq!(plan.slot_for("sparse-pages"), None);
    assert_eq!(plan.size_bytes_for("large-color"), Some(1024));
    assert_eq!(plan.size_bytes_for("small-color"), Some(256));
    let large_color_allocation = plan
        .allocations
        .iter()
        .find(|allocation| allocation.resource_name == "large-color")
        .unwrap();
    let small_color_allocation = plan
        .allocations
        .iter()
        .find(|allocation| allocation.resource_name == "small-color")
        .unwrap();
    let small_buffer_allocation = plan
        .allocations
        .iter()
        .find(|allocation| allocation.resource_name == "small-buffer")
        .unwrap();
    let large_buffer_allocation = plan
        .allocations
        .iter()
        .find(|allocation| allocation.resource_name == "large-buffer")
        .unwrap();
    assert_ne!(
        large_color_allocation.bucket_key_hash,
        small_color_allocation.bucket_key_hash
    );
    assert_ne!(
        large_color_allocation.allocation_id,
        small_color_allocation.allocation_id
    );
    assert_ne!(
        small_buffer_allocation.bucket_key_hash,
        large_buffer_allocation.bucket_key_hash
    );
    assert_ne!(
        small_buffer_allocation.allocation_id,
        large_buffer_allocation.allocation_id
    );
    assert_ne!(
        large_color_allocation.allocation_id,
        small_buffer_allocation.allocation_id
    );
    assert_eq!(
        plan.slot_bytes_for_allocation(large_color_allocation.allocation_id),
        Some(1024)
    );
    assert_eq!(
        plan.slot_bytes_for_allocation(small_color_allocation.allocation_id),
        Some(256)
    );
    assert_eq!(
        plan.slot_bytes_for_allocation(small_buffer_allocation.allocation_id),
        Some(64)
    );
    assert_eq!(
        plan.slot_bytes_for_allocation(large_buffer_allocation.allocation_id),
        Some(128)
    );
    assert_eq!(plan.dense_texture_bytes_reserved, 1280);
    assert_eq!(plan.dense_buffer_bytes_reserved, 192);
    assert_eq!(plan.total_dense_bytes_reserved(), 1472);
    assert_eq!(plan.sparse_texture_virtual_bytes, 4096);
}
