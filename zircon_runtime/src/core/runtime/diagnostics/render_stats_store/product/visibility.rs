use crate::core::framework::render::RenderStats;

use super::{DiagnosticStore, record_bool, record_count};
pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.visibility.view_count",
        frame_index,
        stats.last_visibility_view_count,
        &["render", "visibility"],
    );
    record_count(
        store,
        "render.visibility.input_count",
        frame_index,
        stats.last_visibility_input_count,
        &["render", "visibility"],
    );
    record_count(
        store,
        "render.visibility.layer_filtered_count",
        frame_index,
        stats.last_visibility_layer_filtered_count,
        &["render", "visibility", "layer"],
    );
    record_count(
        store,
        "render.visibility.frustum_culled_count",
        frame_index,
        stats.last_visibility_frustum_culled_count,
        &["render", "visibility", "frustum"],
    );
    record_count(
        store,
        "render.visibility.occlusion_culled_count",
        frame_index,
        stats.last_visibility_occlusion_culled_count,
        &["render", "visibility", "occlusion"],
    );
    record_count(
        store,
        "render.visibility.visible_count",
        frame_index,
        stats.last_visibility_visible_count,
        &["render", "visibility", "visible"],
    );
    record_count(
        store,
        "render.visibility.static_index.full_rebuild_count",
        frame_index,
        stats.last_visibility_static_index_full_rebuild_count,
        &["render", "visibility", "static_index", "rebuild"],
    );
    record_count(
        store,
        "render.visibility.static_index.incremental_update_count",
        frame_index,
        stats.last_visibility_static_index_incremental_update_count,
        &["render", "visibility", "static_index", "incremental"],
    );
    record_count(
        store,
        "render.visibility.static_index.inserted_count",
        frame_index,
        stats.last_visibility_static_index_inserted_count,
        &["render", "visibility", "static_index", "change"],
    );
    record_count(
        store,
        "render.visibility.static_index.updated_count",
        frame_index,
        stats.last_visibility_static_index_updated_count,
        &["render", "visibility", "static_index", "change"],
    );
    record_count(
        store,
        "render.visibility.static_index.removed_count",
        frame_index,
        stats.last_visibility_static_index_removed_count,
        &["render", "visibility", "static_index", "change"],
    );
    record_count(
        store,
        "render.visibility.static_index.indexed_entity_count",
        frame_index,
        stats.last_visibility_static_index_indexed_entity_count,
        &["render", "visibility", "static_index", "entity"],
    );
    record_count(
        store,
        "render.visibility.static_index.occupied_cell_count",
        frame_index,
        stats.last_visibility_static_index_occupied_cell_count,
        &["render", "visibility", "static_index", "cell"],
    );
    record_bool(
        store,
        "render.visibility.static_index.main_view_prefilter_used",
        frame_index,
        stats.last_visibility_static_index_main_view_prefilter_used,
        &[
            "render",
            "visibility",
            "static_index",
            "main_view",
            "prefilter",
        ],
    );
    record_count(
        store,
        "render.visibility.static_index.main_view_static_input_count",
        frame_index,
        stats.last_visibility_static_index_main_view_static_input_count,
        &["render", "visibility", "static_index", "main_view", "input"],
    );
    record_count(
        store,
        "render.visibility.static_index.main_view_static_candidate_count",
        frame_index,
        stats.last_visibility_static_index_main_view_static_candidate_count,
        &[
            "render",
            "visibility",
            "static_index",
            "main_view",
            "candidate",
        ],
    );
}
