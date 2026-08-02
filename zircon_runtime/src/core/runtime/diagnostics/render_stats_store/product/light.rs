use crate::core::framework::render::RenderStats;

use super::{record_count, DiagnosticStore};

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_light_family(
        store,
        frame_index,
        "directional",
        stats.last_directional_light_count,
        stats.last_directional_light_ready_count,
        stats.last_directional_light_degraded_count,
        (
            "render.light.directional.count",
            "render.light.directional.ready_count",
            "render.light.directional.degraded_count",
        ),
    );
    record_light_family(
        store,
        frame_index,
        "point",
        stats.last_point_light_count,
        stats.last_point_light_ready_count,
        stats.last_point_light_degraded_count,
        (
            "render.light.point.count",
            "render.light.point.ready_count",
            "render.light.point.degraded_count",
        ),
    );
    record_light_family(
        store,
        frame_index,
        "spot",
        stats.last_spot_light_count,
        stats.last_spot_light_ready_count,
        stats.last_spot_light_degraded_count,
        (
            "render.light.spot.count",
            "render.light.spot.ready_count",
            "render.light.spot.degraded_count",
        ),
    );
    record_light_family(
        store,
        frame_index,
        "ambient",
        stats.last_ambient_light_count,
        stats.last_ambient_light_ready_count,
        stats.last_ambient_light_degraded_count,
        (
            "render.light.ambient.count",
            "render.light.ambient.ready_count",
            "render.light.ambient.degraded_count",
        ),
    );
    record_light_family(
        store,
        frame_index,
        "rect",
        stats.last_rect_light_count,
        stats.last_rect_light_ready_count,
        stats.last_rect_light_degraded_count,
        (
            "render.light.rect.count",
            "render.light.rect.ready_count",
            "render.light.rect.degraded_count",
        ),
    );
}

fn record_light_family(
    store: &mut DiagnosticStore,
    frame_index: u64,
    family_tag: &'static str,
    count: usize,
    ready_count: usize,
    degraded_count: usize,
    paths: (&'static str, &'static str, &'static str),
) {
    record_count(
        store,
        paths.0,
        frame_index,
        count,
        &["render", "light", family_tag],
    );
    record_count(
        store,
        paths.1,
        frame_index,
        ready_count,
        &["render", "light", family_tag, "ready"],
    );
    record_count(
        store,
        paths.2,
        frame_index,
        degraded_count,
        &["render", "light", family_tag, "degraded"],
    );
}
