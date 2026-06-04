mod advanced_provider;
mod anti_alias;
mod capability;
mod graph;
mod history;
mod hybrid_gi;
mod particle;
mod product;
mod solari;
mod virtual_geometry;

use crate::core::framework::render::RenderStats;

use super::DiagnosticStore;

pub(super) fn record_render_stats_diagnostics(store: &mut DiagnosticStore, stats: &RenderStats) {
    capability::record(store, stats);
    history::record(store, stats);
    graph::record(store, stats);
    product::record(store, stats);
    anti_alias::record(store, stats);
    particle::record(store, stats);
    virtual_geometry::record(store, stats);
    hybrid_gi::record(store, stats);
    advanced_provider::record(store, stats);
    solari::record(store, stats);
}

fn record_count(
    store: &mut DiagnosticStore,
    path: &'static str,
    frame_index: u64,
    value: usize,
    subsystem_tags: &[&str],
) {
    store.record(
        path,
        frame_index,
        value as f64,
        Some("count"),
        subsystem_tags.iter().copied(),
    );
}

fn record_bytes(
    store: &mut DiagnosticStore,
    path: &'static str,
    frame_index: u64,
    value: u64,
    subsystem_tags: &[&str],
) {
    store.record(
        path,
        frame_index,
        value as f64,
        Some("bytes"),
        subsystem_tags.iter().copied(),
    );
}

fn record_bool(
    store: &mut DiagnosticStore,
    path: &'static str,
    frame_index: u64,
    value: bool,
    subsystem_tags: &[&str],
) {
    store.record(
        path,
        frame_index,
        u8::from(value) as f64,
        Some("bool"),
        subsystem_tags.iter().copied(),
    );
}
