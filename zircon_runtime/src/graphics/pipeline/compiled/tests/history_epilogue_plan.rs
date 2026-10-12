use crate::render_graph::{QueueLane, RenderGraphBuilder};
use crate::rhi::{TextureDesc, TextureFormat, TextureUsage};

use super::*;

#[test]
fn plan_keeps_the_final_live_writer_access_for_history_outputs() {
    let mut builder = RenderGraphBuilder::new("history-epilogue-final-writer");
    let output = builder.create_texture(TextureDesc::new(
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY,
        64,
        32,
        TextureFormat::Rgba16Float,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::COPY_SRC,
    ));
    builder.mark_persistent(output).unwrap();
    let first = builder.add_pass("history-first-writer", QueueLane::Graphics);
    let final_writer = builder.add_pass("history-final-writer", QueueLane::Graphics);
    builder.write_texture(first, output).unwrap();
    builder.write_texture(final_writer, output).unwrap();
    builder.add_dependency(first, final_writer).unwrap();
    let graph = builder.compile().unwrap();
    let final_access = graph.access_id_at(final_writer, 0).unwrap();

    let plan = CompiledHistoryEpiloguePlan::from_graph(&graph).unwrap();

    assert_eq!(
        plan.screen_space_reflection()
            .map(|source| source.access_id()),
        Some(final_access)
    );
}

#[test]
fn plan_rejects_a_history_output_without_copy_source_usage() {
    let mut builder = RenderGraphBuilder::new("history-epilogue-copy-source-contract");
    let output = builder.create_texture(TextureDesc::new(
        PostProcessGraphResourceNames::GLOBAL_ILLUMINATION,
        32,
        32,
        TextureFormat::Rgba16Float,
        TextureUsage::RENDER_ATTACHMENT,
    ));
    builder.mark_persistent(output).unwrap();
    let writer = builder.add_pass("history-writer", QueueLane::Graphics);
    builder.write_texture(writer, output).unwrap();
    let graph = builder.compile().unwrap();

    let error = CompiledHistoryEpiloguePlan::from_graph(&graph)
        .expect_err("history output must be copyable before frame execution");

    assert!(error.contains("must declare COPY_SRC usage"));
}
