use super::*;

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
    let output = builder.import_external_resource("viewport-output");

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
    let plan = graph.transient_allocation_plan();

    assert_eq!(plan.texture_slot_count, 2);
    assert_eq!(plan.buffer_slot_count, 0);
    assert_eq!(plan.slot_for("history"), Some(0));
    assert_eq!(plan.slot_for("lighting"), Some(1));
    assert_eq!(plan.slot_for("resolved"), Some(0));
    assert_eq!(plan.slot_for("viewport-output"), None);
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
    let output = builder.import_external_resource("viewport-output");

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
        small_buffer_allocation.bucket_key_hash,
        large_buffer_allocation.bucket_key_hash
    );
    assert_eq!(
        plan.slot_bytes_for_bucket(
            RenderGraphResourceKind::TransientTexture,
            0,
            large_color_allocation.bucket_key_hash,
        ),
        Some(1024)
    );
    assert_eq!(
        plan.slot_bytes_for_bucket(
            RenderGraphResourceKind::TransientTexture,
            0,
            small_color_allocation.bucket_key_hash,
        ),
        Some(256)
    );
    assert_eq!(
        plan.slot_bytes_for_bucket(
            RenderGraphResourceKind::TransientBuffer,
            0,
            small_buffer_allocation.bucket_key_hash,
        ),
        Some(64)
    );
    assert_eq!(
        plan.slot_bytes_for_bucket(
            RenderGraphResourceKind::TransientBuffer,
            0,
            large_buffer_allocation.bucket_key_hash,
        ),
        Some(128)
    );
    assert_eq!(
        plan.slot_bytes(RenderGraphResourceKind::TransientTexture, 0),
        Some(1280)
    );
    assert_eq!(
        plan.slot_bytes(RenderGraphResourceKind::TransientBuffer, 0),
        Some(192)
    );
    assert_eq!(plan.dense_texture_bytes_reserved, 1280);
    assert_eq!(plan.dense_buffer_bytes_reserved, 192);
    assert_eq!(plan.total_dense_bytes_reserved(), 1472);
    assert_eq!(plan.sparse_texture_virtual_bytes, 4096);
}
