mod advanced_provider;
mod anti_alias;
mod capability;
mod graph;
mod history;
mod hybrid_gi;
mod particle;
mod post_process;
mod product;
mod profile;
mod shader_variant;
mod solari;
mod virtual_geometry;
mod volumetric_fog;

use crate::core::framework::render::RenderStats;

use super::DiagnosticStore;

pub(crate) fn record_render_stats_diagnostics(store: &mut DiagnosticStore, stats: &RenderStats) {
    capability::record(store, stats);
    history::record(store, stats);
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

fn record_count(
    store: &mut DiagnosticStore,
    path: &'static str,
    frame_index: u64,
    value: usize,
    subsystem_tags: &[&str],
) {
    store.record_static(
        path,
        frame_index,
        value as f64,
        Some("count"),
        subsystem_tags,
    );
}

fn record_bytes(
    store: &mut DiagnosticStore,
    path: &'static str,
    frame_index: u64,
    value: u64,
    subsystem_tags: &[&str],
) {
    store.record_static(
        path,
        frame_index,
        value as f64,
        Some("bytes"),
        subsystem_tags,
    );
}

fn record_microseconds(
    store: &mut DiagnosticStore,
    path: &'static str,
    frame_index: u64,
    value: u64,
    subsystem_tags: &[&str],
) {
    store.record_static(
        path,
        frame_index,
        value as f64,
        Some("microseconds"),
        subsystem_tags,
    );
}

fn record_bool(
    store: &mut DiagnosticStore,
    path: &'static str,
    frame_index: u64,
    value: bool,
    subsystem_tags: &[&str],
) {
    store.record_static(
        path,
        frame_index,
        u8::from(value) as f64,
        Some("bool"),
        subsystem_tags,
    );
}

#[cfg(test)]
mod performance_tests {
    #[test]
    fn render_stats_helpers_use_static_metadata_recording() {
        let source = include_str!("render_stats_store.rs");
        let end = source
            .find("mod performance_tests {")
            .expect("performance test module");
        let implementation = &source[..end];

        assert_eq!(implementation.matches("store.record_static(").count(), 4);
        assert_eq!(implementation.matches("store.record(").count(), 0);
    }

    #[test]
    fn render_stats_product_leaves_use_static_metadata_recording() {
        for (name, source) in [
            (
                "effect_stack",
                include_str!("render_stats_store/product/effect_stack.rs"),
            ),
            (
                "light_grid",
                include_str!("render_stats_store/product/light_grid.rs"),
            ),
        ] {
            assert!(
                !source.contains("store.record("),
                "{name} bypassed static render diagnostic metadata"
            );
        }
    }
}
