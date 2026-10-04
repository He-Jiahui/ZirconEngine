use super::*;

#[test]
fn runtime_mipgen_graph_node_follows_the_last_writer() {
    let mut graph = RenderGraphBuilder::new("runtime-mipgen");
    let texture = graph.import_present_external_resource("runtime-texture");
    let writer = graph.add_pass("texture-producer", QueueLane::Graphics);
    graph
        .write_external(writer, texture)
        .expect("producer writes the imported texture");
    let insertion =
        insert_runtime_mipgen_after_last_writer(&mut graph, texture, "runtime-texture", writer)
            .expect("mip node is inserted after producer");
    let compiled = graph.compile().expect("runtime mip graph compiles");
    let mip_pass = compiled
        .passes()
        .iter()
        .find(|pass| pass.id == insertion.pass())
        .expect("compiled graph retains mip pass");

    assert_eq!(insertion.source_writer(), writer);
    assert!(mip_pass.dependencies.contains(&writer));
    assert_eq!(mip_pass.queue, QueueLane::AsyncCompute);
    assert_eq!(
        mip_pass.executor_id.as_deref(),
        Some(RUNTIME_MIP_GEN_EXECUTOR_ID)
    );
    assert_eq!(mip_pass.resources.len(), 1);
}
