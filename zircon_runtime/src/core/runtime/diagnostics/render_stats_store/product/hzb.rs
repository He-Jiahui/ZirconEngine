use crate::core::framework::render::RenderStats;

use super::{record_bool, record_count, DiagnosticStore};
pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.hzb.mip_count",
        frame_index,
        stats.last_hzb_mip_count,
        &["render", "hzb", "mip"],
    );
    record_count(
        store,
        "render.hzb.graph_executed_pass_count",
        frame_index,
        stats.last_hzb_graph_executed_pass_count,
        &["render", "hzb", "graph"],
    );
    record_bool(
        store,
        "render.hzb.occlusion.reported",
        frame_index,
        stats.last_hzb_occlusion_reported,
        &["render", "hzb", "occlusion"],
    );
    record_count(
        store,
        "render.hzb.occlusion.candidate_arg_count",
        frame_index,
        stats.last_hzb_occlusion_candidate_arg_count,
        &["render", "hzb", "occlusion", "candidate"],
    );
    record_count(
        store,
        "render.hzb.occlusion.candidate_instance_count",
        frame_index,
        stats.last_hzb_occlusion_candidate_instance_count,
        &["render", "hzb", "occlusion", "candidate"],
    );
    record_count(
        store,
        "render.hzb.occlusion.dispatch_group_count",
        frame_index,
        stats.last_hzb_occlusion_dispatch_group_count,
        &["render", "hzb", "occlusion", "dispatch"],
    );
    record_count(
        store,
        "render.hzb.occlusion.dispatched_phase_count",
        frame_index,
        stats.last_hzb_occlusion_dispatched_phase_count,
        &["render", "hzb", "occlusion", "dispatch"],
    );
    record_bool(
        store,
        "render.hzb.occlusion.history_available",
        frame_index,
        stats.last_hzb_occlusion_history_available,
        &["render", "hzb", "occlusion", "history"],
    );
    record_bool(
        store,
        "render.hzb.occlusion.readback_available",
        frame_index,
        stats.last_hzb_occlusion_readback_available,
        &["render", "hzb", "occlusion", "readback"],
    );
    record_count(
        store,
        "render.hzb.occlusion.tested_arg_count",
        frame_index,
        stats.last_hzb_occlusion_tested_arg_count,
        &["render", "hzb", "occlusion", "tested"],
    );
    record_count(
        store,
        "render.hzb.occlusion.tested_instance_count",
        frame_index,
        stats.last_hzb_occlusion_tested_instance_count,
        &["render", "hzb", "occlusion", "tested"],
    );
    record_count(
        store,
        "render.hzb.occlusion.culled_arg_count",
        frame_index,
        stats.last_hzb_occlusion_culled_arg_count,
        &["render", "hzb", "occlusion", "culled"],
    );
    record_count(
        store,
        "render.hzb.occlusion.culled_instance_count",
        frame_index,
        stats.last_hzb_occlusion_culled_instance_count,
        &["render", "hzb", "occlusion", "culled"],
    );
    record_bool(
        store,
        "render.hzb.occlusion.indirect_args_readback_available",
        frame_index,
        stats.last_hzb_occlusion_indirect_args_readback_available,
        &["render", "hzb", "occlusion", "readback", "indirect_args"],
    );
    record_count(
        store,
        "render.hzb.occlusion.readback_arg_count",
        frame_index,
        stats.last_hzb_occlusion_readback_arg_count,
        &["render", "hzb", "occlusion", "readback", "indirect_args"],
    );
    record_count(
        store,
        "render.hzb.occlusion.compacted_draw_count",
        frame_index,
        stats.last_hzb_occlusion_compacted_draw_count,
        &["render", "hzb", "occlusion", "readback", "indirect_args"],
    );
    record_count(
        store,
        "render.hzb.occlusion.zero_instance_arg_count",
        frame_index,
        stats.last_hzb_occlusion_zero_instance_arg_count,
        &["render", "hzb", "occlusion", "readback", "indirect_args"],
    );
    record_count(
        store,
        "render.hzb.occlusion.remaining_instance_count",
        frame_index,
        stats.last_hzb_occlusion_remaining_instance_count,
        &["render", "hzb", "occlusion", "readback", "indirect_args"],
    );
}
