use crate::core::framework::render::RenderStats;

use super::{
    advanced_provider, ambient_occlusion, anti_alias, capability, graph, history, hybrid_gi,
    particle, post_process, product, profile, scene_submission_completion, shader_variant, solari,
    virtual_geometry, volumetric_fog, DiagnosticStore,
};

// TODO: [CR-RENDER-STATS-0001] 确认同一提交帧重复查询时是否应再次写入所有序列；当前采集器每次查询均调用此处，DiagnosticStore 不按帧去重，会挤占历史窗口并改变平滑值。
/// 由运行时诊断采集器在取得 RenderStats 后调用，将同一提交帧的各子域指标写入共享存储。
/// 子域应保持固定路径、单位与 submitted_frames 帧索引，使完整快照和当前值快照可对齐。
pub(crate) fn record_render_stats_diagnostics(store: &mut DiagnosticStore, stats: &RenderStats) {
    capability::record(store, stats);
    history::record(store, stats);
    ambient_occlusion::record(store, stats);
    scene_submission_completion::record(store, stats);
    graph::record(store, stats);
    profile::record(store, stats);
    product::record(store, stats);
    shader_variant::record(store, stats);
    post_process::record(store, stats);
    anti_alias::record(store, stats);
    particle::record(store, stats);
    virtual_geometry::record(store, stats);
    hybrid_gi::record(store, stats);
    volumetric_fog::record(store, stats);
    advanced_provider::record(store, stats);
    solari::record(store, stats);
}
