use crate::core::framework::render::RenderStats;

use super::super::{record_bool, record_count, DiagnosticStore};

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.post_process.graph.node_count",
        frame_index,
        stats.last_post_process_graph_node_count,
        &["render", "post_process", "graph"],
    );
    record_count(
        store,
        "render.post_process.graph.skipped_node_count",
        frame_index,
        stats.last_post_process_graph_skipped_node_count,
        &["render", "post_process", "graph"],
    );
    record_count(
        store,
        "render.post_process.graph.executed_node_count",
        frame_index,
        stats.last_post_process_graph_executed_nodes.len(),
        &["render", "post_process", "graph"],
    );
    record_bool(
        store,
        "render.post_process.graph.output_transfer_present",
        frame_index,
        stats.last_post_process_output_transfer_node.is_some(),
        &["render", "post_process", "graph", "output_transfer"],
    );
}
