use crate::core::framework::render::RenderStats;

use super::{
    advanced_provider, ambient_occlusion, anti_alias, capability, graph, history, hybrid_gi,
    particle, post_process, product, profile, scene_submission_completion, shader_variant, solari,
    virtual_geometry, volumetric_fog, DiagnosticStore,
};

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
