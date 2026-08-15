use crate::render_graph::{
    ExternalResource, QueueLane, RenderGraphBuilder, RenderGraphError,
    RenderGraphExternalResourceBinding, RenderGraphResource, RenderGraphResourceAccessKind,
    RenderGraphResourceUsageFlags,
};

fn import_texture_view(
    builder: &mut RenderGraphBuilder,
    name: &str,
    alias_group: &str,
) -> ExternalResource {
    builder.import_external_resource_with_usage_binding_and_alias_group(
        name,
        RenderGraphResourceUsageFlags::persistent(),
        RenderGraphExternalResourceBinding::required_texture(),
        alias_group,
    )
}

#[test]
fn shared_external_alias_infers_raw_dependency_across_distinct_views() {
    let mut builder = RenderGraphBuilder::new("external-alias-raw");
    let storage_view = import_texture_view(&mut builder, "cube.storage.mip0", "cube");
    let sampled_view = import_texture_view(&mut builder, "cube.sampled", "cube");
    let producer = builder.add_pass("produce-cube", QueueLane::AsyncCompute);
    let consumer = builder.add_pass("consume-cube", QueueLane::AsyncCompute);

    builder
        .write_storage_external(producer, storage_view)
        .unwrap();
    builder.read_external(consumer, sampled_view).unwrap();

    let graph = builder.compile().unwrap();
    let compiled_consumer = graph
        .passes()
        .iter()
        .find(|pass| pass.id == consumer)
        .unwrap();

    assert_eq!(compiled_consumer.dependencies, vec![producer]);
    assert_eq!(
        graph
            .resource_version_for_access(
                consumer,
                RenderGraphResource::External(sampled_view),
                RenderGraphResourceAccessKind::Read,
            )
            .unwrap()
            .ordinal(),
        1
    );
}

#[test]
fn shared_external_alias_infers_waw_dependency_across_distinct_views() {
    let mut builder = RenderGraphBuilder::new("external-alias-waw");
    let first_view = import_texture_view(&mut builder, "cube.storage.mip0", "cube");
    let second_view = import_texture_view(&mut builder, "cube.storage.mip1", "cube");
    let first_writer = builder.add_pass("write-mip0", QueueLane::AsyncCompute);
    let second_writer = builder.add_pass("write-mip1", QueueLane::AsyncCompute);

    builder
        .write_storage_external(first_writer, first_view)
        .unwrap();
    builder
        .write_storage_external(second_writer, second_view)
        .unwrap();

    let graph = builder.compile().unwrap();
    let compiled_second_writer = graph
        .passes()
        .iter()
        .find(|pass| pass.id == second_writer)
        .unwrap();

    assert_eq!(compiled_second_writer.dependencies, vec![first_writer]);
}

#[test]
fn shared_external_alias_rejects_mixed_resource_types() {
    let mut builder = RenderGraphBuilder::new("external-alias-type");
    builder.import_external_resource_with_usage_binding_and_alias_group(
        "cube.texture",
        RenderGraphResourceUsageFlags::persistent(),
        RenderGraphExternalResourceBinding::required_texture(),
        "shared-allocation",
    );
    builder.import_external_resource_with_usage_binding_and_alias_group(
        "cube.buffer",
        RenderGraphResourceUsageFlags::persistent(),
        RenderGraphExternalResourceBinding::required_buffer(),
        "shared-allocation",
    );

    assert!(matches!(
        builder.compile(),
        Err(RenderGraphError::ExternalAliasResourceTypeMismatch { .. })
    ));
}
