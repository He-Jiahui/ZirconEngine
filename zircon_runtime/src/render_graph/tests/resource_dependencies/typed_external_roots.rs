use crate::render_graph::{
    QueueLane, RenderGraphBufferRange, RenderGraphBuilder, RenderGraphExternalResourceBinding,
    RenderGraphResourceAccessIntent, RenderGraphResourceAccessKind, RenderGraphResourceAccessRange,
    RenderGraphShaderStages,
};
use crate::rhi::{BufferDesc, BufferUsage};

#[test]
fn descriptor_backed_present_external_buffer_cull_root_keeps_each_final_window_writer() {
    let mut builder = RenderGraphBuilder::new("typed-present-external-buffer-cull-root");
    let buffer = builder.import_present_external_buffer_with_binding(
        "present-history-worklist",
        BufferDesc::new("present-history-worklist", 128, BufferUsage::STORAGE),
        RenderGraphExternalResourceBinding::required_buffer(),
    );
    let write_first = builder.add_pass("write-present-first-window", QueueLane::AsyncCompute);
    let write_second = builder.add_pass("write-present-second-window", QueueLane::AsyncCompute);
    let write_intent = RenderGraphResourceAccessIntent::storage_buffer_read_write(
        RenderGraphShaderStages::COMPUTE,
    );

    for (pass, range) in [
        (write_first, RenderGraphBufferRange::new(0, Some(32))),
        (write_second, RenderGraphBufferRange::new(64, Some(32))),
    ] {
        builder
            .access_external(
                pass,
                buffer,
                RenderGraphResourceAccessKind::Write,
                RenderGraphResourceAccessRange::Buffer(range),
                write_intent,
                None,
            )
            .expect("write one present external buffer window");
    }

    let graph = builder
        .compile()
        .expect("the typed buffer root must cover each written window");

    assert!(graph.passes().iter().all(|pass| !pass.culled));
}
