use crate::core::CoreRuntime;

use super::support::{
    assert_render_bool_series, assert_render_count_series, fake_render_module,
    DIAGNOSTICS_TEST_MODULE,
};

#[test]
fn runtime_diagnostics_reports_motion_vector_object_history_and_mesh_draw_eligibility() {
    let runtime = CoreRuntime::new();
    runtime.register_module(fake_render_module()).unwrap();
    runtime.activate_module(DIAGNOSTICS_TEST_MODULE).unwrap();

    let snapshot = crate::core::diagnostics::collect_runtime_diagnostics(&runtime.handle());

    assert_render_bool_series(
        &snapshot.store,
        "render.post_process.motion_vector.camera.ready",
        true,
        &["post_process", "motion_vector", "camera", "ready"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.post_process.motion_vector.object.previous_history_count",
        3.0,
        &["post_process", "motion_vector", "object", "history"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.post_process.motion_vector.object.current_history_count",
        4.0,
        &["post_process", "motion_vector", "object", "history"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.post_process.motion_vector.object.matched_history_count",
        2.0,
        &[
            "post_process",
            "motion_vector",
            "object",
            "history",
            "matched",
        ],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.post_process.motion_vector.object.missing_history_count",
        2.0,
        &[
            "post_process",
            "motion_vector",
            "object",
            "history",
            "missing",
        ],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.previous_motion_vector_transform_draw_count",
        5.0,
        &["mesh", "queue", "motion_vector", "previous"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.missing_motion_vector_transform_draw_count",
        2.0,
        &["mesh", "queue", "motion_vector", "missing"],
    );
}
