use super::two_mip_storage_texture;
use crate::render_graph::{
    QueueLane, RenderGraphBuilder, RenderGraphResourceAccessIntent, RenderGraphResourceAccessKind,
    RenderGraphShaderStages, RenderGraphTextureSubresourceRange,
};

#[test]
fn persistent_texture_view_alias_root_keeps_a_later_parent_write_to_the_same_scope() {
    let mut builder = RenderGraphBuilder::new("persistent-texture-view-alias-final-parent-writer");
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
        .expect("the view is the culling root");
    let write_alias = builder.add_pass("write-alias-mip-one", QueueLane::AsyncCompute);
    let write_parent = builder.add_pass("write-parent-mip-one", QueueLane::AsyncCompute);
    let write_intent =
        RenderGraphResourceAccessIntent::storage_texture_write(RenderGraphShaderStages::COMPUTE);

    builder
        .access_texture(
            write_alias,
            alias,
            RenderGraphResourceAccessKind::Write,
            RenderGraphTextureSubresourceRange::full(),
            write_intent,
            None,
        )
        .expect("write the rooted alias scope");
    builder
        .access_texture(
            write_parent,
            parent,
            RenderGraphResourceAccessKind::Write,
            RenderGraphTextureSubresourceRange::single_mip(1),
            write_intent,
            None,
        )
        .expect("overwrite the same parent mip");

    let graph = builder
        .compile()
        .expect("the root must resolve the shared physical history");
    let alias_writer = graph
        .passes()
        .iter()
        .find(|pass| pass.id == write_alias)
        .expect("alias writer pass");
    let parent_writer = graph
        .passes()
        .iter()
        .find(|pass| pass.id == write_parent)
        .expect("parent writer pass");

    assert!(alias_writer.culled);
    assert!(!parent_writer.culled);
}
