use super::*;

#[test]
fn runtime_15_runtime_diagnostics_tests_are_folder_backed() {
    let parent = read_runtime_src("tests/runtime_diagnostics/mod.rs");
    let capability_history_visibility =
        read_runtime_src("tests/runtime_diagnostics/capability_history_visibility.rs");
    let hzb_light_camera_capture =
        read_runtime_src("tests/runtime_diagnostics/hzb_light_camera_capture.rs");
    let graph_resources = read_runtime_src("tests/runtime_diagnostics/graph_resources.rs");
    let graph_execution = read_runtime_src("tests/runtime_diagnostics/graph_execution.rs");
    let post_process_material_mesh =
        read_runtime_src("tests/runtime_diagnostics/post_process_material_mesh.rs");
    let gpu_sprite_ui_advanced =
        read_runtime_src("tests/runtime_diagnostics/gpu_sprite_ui_advanced.rs");
    let motion_vector = read_runtime_src("tests/runtime_diagnostics/motion_vector.rs");
    let support = read_runtime_src("tests/runtime_diagnostics/support.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "runtime diagnostics parent test module mounts",
        &parent,
        &[
            "mod capability_history_visibility;",
            "mod gpu_sprite_ui_advanced;",
            "mod graph_execution;",
            "mod graph_resources;",
            "mod hzb_light_camera_capture;",
            "mod motion_vector;",
            "mod post_process_material_mesh;",
            "mod support;",
            "fn runtime_diagnostics_reports_missing_runtime_contracts_without_panicking",
            "fn runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins",
        ],
    );

    for moved_anchor in [
        "render.capability.queue_class_count",
        "render.hzb.occlusion.compacted_draw_count",
        "render.graph.materialization.required_resource_count",
        "render.graph.execution.alias.texture_logical_count",
        "render.post_process.effect_stack.enabled",
        "render.gpu_scene.primitive_count",
        "render.advanced_provider.virtual_geometry.ready",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "tests/runtime_diagnostics/mod.rs should delegate diagnostic series assertions instead of defining {moved_anchor}"
        );
    }

    assert_contains_all(
        "runtime diagnostics capability child owns core render series",
        &capability_history_visibility,
        &[
            "pub(super) fn assert_capability_history_visibility",
            "render.capability.queue_class_count",
            "render.visibility.static_index.main_view_static_candidate_count",
        ],
    );
    assert_contains_all(
        "runtime diagnostics HZB child owns camera/capture series",
        &hzb_light_camera_capture,
        &[
            "pub(super) fn assert_hzb_light_camera_capture",
            "render.hzb.occlusion.compacted_draw_count",
            "render.camera.target.graph_import.ready_for_direct_import",
            "render.capture.source.framework_offscreen",
        ],
    );
    assert_contains_all(
        "runtime diagnostics graph resources child owns resource series",
        &graph_resources,
        &[
            "pub(super) fn assert_graph_resources",
            "render.graph.execution.transient_pool.texture_created_count",
            "render.graph.materialization.required_resource_count",
        ],
    );
    assert_contains_all(
        "runtime diagnostics graph execution child owns execution series",
        &graph_execution,
        &[
            "pub(super) fn assert_graph_execution",
            "render.graph.execution.alias.texture_logical_count",
            "render.graph.execution.profile.cpu_elapsed_total_us",
            "render.graph.executed_async_compute_pass_count",
        ],
    );
    assert_contains_all(
        "runtime diagnostics post-process child owns material/mesh series",
        &post_process_material_mesh,
        &[
            "pub(super) fn assert_post_process_material_mesh",
            "render.post_process.effect_stack.enabled",
            "assert_light_family_series",
            "render.mesh.queue.indirect_args_count",
        ],
    );
    assert_contains_all(
        "runtime diagnostics advanced child owns GPU/provider series",
        &gpu_sprite_ui_advanced,
        &[
            "pub(super) fn assert_gpu_sprite_ui_advanced",
            "render.gpu_scene.primitive_count",
            "render.virtual_geometry.cluster_budget",
            "render.advanced_provider.virtual_geometry.ready",
            "render.solari.experimental_disabled_degradation_count",
        ],
    );
    assert_contains_all(
        "runtime diagnostics existing motion-vector child remains mounted",
        &motion_vector,
        &["runtime_diagnostics_reports_motion_vector_camera_and_mesh_draw_eligibility"],
    );
    assert_contains_all(
        "runtime diagnostics support remains shared helper owner",
        &support,
        &[
            "pub(super) fn fake_render_module",
            "pub(super) fn assert_render_count_series",
        ],
    );

    for (path, source) in [
        ("tests/runtime_diagnostics/mod.rs", parent.as_str()),
        (
            "tests/runtime_diagnostics/capability_history_visibility.rs",
            capability_history_visibility.as_str(),
        ),
        (
            "tests/runtime_diagnostics/hzb_light_camera_capture.rs",
            hzb_light_camera_capture.as_str(),
        ),
        (
            "tests/runtime_diagnostics/graph_resources.rs",
            graph_resources.as_str(),
        ),
        (
            "tests/runtime_diagnostics/graph_execution.rs",
            graph_execution.as_str(),
        ),
        (
            "tests/runtime_diagnostics/post_process_material_mesh.rs",
            post_process_material_mesh.as_str(),
        ),
        (
            "tests/runtime_diagnostics/gpu_sprite_ui_advanced.rs",
            gpu_sprite_ui_advanced.as_str(),
        ),
        (
            "tests/runtime_diagnostics/motion_vector.rs",
            motion_vector.as_str(),
        ),
        ("tests/runtime_diagnostics/support.rs", support.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
