use crate::core::CoreRuntime;
use crate::runtime_diagnostics::collect_runtime_diagnostics;

use super::support::{
    assert_render_bool_series, assert_render_count_series, fake_render_module,
    DIAGNOSTICS_TEST_MODULE,
};

#[test]
fn runtime_diagnostics_reports_motion_vector_camera_and_mesh_draw_eligibility() {
    let runtime = CoreRuntime::new();
    runtime.register_module(fake_render_module()).unwrap();
    runtime.activate_module(DIAGNOSTICS_TEST_MODULE).unwrap();

    let snapshot = collect_runtime_diagnostics(&runtime.handle());

    assert_render_bool_series(
        &snapshot.store,
        "render.post_process.motion_vector.camera.ready",
        true,
        &["post_process", "motion_vector", "camera", "ready"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.previous_velocity_transform_draw_count",
        5.0,
        &["mesh", "queue", "velocity", "previous"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.skinned_gpu_cpu_morphed_previous_shape_velocity_missing_count",
        1.0,
        &[
            "mesh",
            "queue",
            "skinned",
            "gpu_source",
            "cpu_morphed",
            "previous_shape_missing",
            "velocity",
        ],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.missing_velocity_transform_draw_count",
        2.0,
        &["mesh", "queue", "velocity", "missing"],
    );
}
