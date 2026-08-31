use super::non_cullable;
use crate::render_graph::{
    QueueLane, RenderGraphBuilder, RenderGraphResourceAccessIntent, RenderGraphResourceAccessKind,
    RenderGraphTextureAspect, RenderGraphTextureSubresourceRange,
};
use crate::rhi::{TextureDesc, TextureFormat, TextureUsage};

#[test]
fn depth_and_stencil_plane_writes_do_not_create_a_waw_dependency() {
    let mut builder = RenderGraphBuilder::new("depth-stencil-plane-hazards");
    let texture = builder.create_texture(TextureDesc::new(
        "depth-stencil",
        32,
        32,
        TextureFormat::Depth24PlusStencil8,
        TextureUsage::RENDER_ATTACHMENT,
    ));
    let write_depth = builder.add_pass("write-depth", QueueLane::Graphics);
    let write_stencil = builder.add_pass("write-stencil", QueueLane::Graphics);

    for (pass, aspect) in [
        (write_depth, RenderGraphTextureAspect::Depth),
        (write_stencil, RenderGraphTextureAspect::Stencil),
    ] {
        builder
            .access_texture(
                pass,
                texture,
                RenderGraphResourceAccessKind::Write,
                RenderGraphTextureSubresourceRange::full().with_aspect(aspect),
                RenderGraphResourceAccessIntent::DepthStencilAttachment,
                None,
            )
            .expect("declare one depth-stencil plane write");
        builder
            .set_pass_flags(pass, non_cullable())
            .expect("keep the pass live");
    }

    let graph = builder
        .compile()
        .expect("depth and stencil planes must retain independent histories");

    assert!(graph.passes()[1].dependencies.is_empty());
}
