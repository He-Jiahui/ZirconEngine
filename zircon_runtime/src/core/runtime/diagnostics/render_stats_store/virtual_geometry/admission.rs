use crate::core::framework::render::{RenderStats, RenderVirtualGeometryPayloadSource};

use super::super::{record_bool, record_count, DiagnosticStore};

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    record_input_and_visibility(store, stats);
    record_payload_source(store, stats);
}

fn record_input_and_visibility(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.virtual_geometry.cluster_budget",
        frame_index,
        stats.last_virtual_geometry_cluster_budget,
        &["render", "virtual_geometry", "budget"],
    );
    record_count(
        store,
        "render.virtual_geometry.page_budget",
        frame_index,
        stats.last_virtual_geometry_page_budget,
        &["render", "virtual_geometry", "budget"],
    );
    record_count(
        store,
        "render.virtual_geometry.input_cluster_count",
        frame_index,
        stats.last_virtual_geometry_input_cluster_count,
        &["render", "virtual_geometry", "input"],
    );
    record_count(
        store,
        "render.virtual_geometry.input_page_count",
        frame_index,
        stats.last_virtual_geometry_input_page_count,
        &["render", "virtual_geometry", "input"],
    );
    record_count(
        store,
        "render.virtual_geometry.visible_cluster_count",
        frame_index,
        stats.last_virtual_geometry_visible_cluster_count,
        &["render", "virtual_geometry", "visibility"],
    );
    record_count(
        store,
        "render.virtual_geometry.visible_entity_count",
        frame_index,
        stats.last_virtual_geometry_visible_entity_count,
        &["render", "virtual_geometry", "visibility"],
    );
    record_count(
        store,
        "render.virtual_geometry.instance_count",
        frame_index,
        stats.last_virtual_geometry_instance_count,
        &["render", "virtual_geometry", "instance"],
    );
}

fn record_payload_source(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    let source = stats.last_virtual_geometry_payload_source;
    record_bool(
        store,
        "render.virtual_geometry.payload.source.none",
        frame_index,
        source == RenderVirtualGeometryPayloadSource::None,
        &["render", "virtual_geometry", "payload", "source"],
    );
    record_bool(
        store,
        "render.virtual_geometry.payload.source.authored",
        frame_index,
        source == RenderVirtualGeometryPayloadSource::Authored,
        &["render", "virtual_geometry", "payload", "source"],
    );
    record_bool(
        store,
        "render.virtual_geometry.payload.source.automatic_fallback",
        frame_index,
        source == RenderVirtualGeometryPayloadSource::AutomaticFallback,
        &[
            "render",
            "virtual_geometry",
            "payload",
            "source",
            "fallback",
        ],
    );
}
