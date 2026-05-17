use crate::core::framework::render::PostProcessPassGraph;

pub(crate) fn build_post_process_pass_graph(graph: &PostProcessPassGraph) -> PostProcessPassGraph {
    graph.clone()
}
