use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_gpu_scene_tests_are_child_owner() {
    let root = read_runtime_src("graphics/scene/gpu_scene/gpu_scene.rs");
    let tests = read_runtime_src("graphics/scene/gpu_scene/gpu_scene/tests.rs");

    let plan_03 = read_repo("docs/plans/zircon_runtime/render/03/2026-07-09-gpu-scene-gpu-driven-output-records.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let gpu_scene_doc = read_repo("docs/zircon_runtime/graphics/scene/gpu_scene/mod.md");
    let mesh_pass_doc =
        read_repo("docs/zircon_runtime/graphics/scene/scene_renderer/mesh/mesh_pass.md");

    assert_contains_all(
        "GPUScene parent keeps data-plane buffers, upload owner, and child test mount",
        &root,
        &[
            "pub(crate) struct GpuScene",
            "pub(crate) fn register(",
            "pub(crate) fn write_primitive(",
            "pub(crate) fn write_instances(",
            "pub(crate) fn write_lights(",
            "pub(crate) fn flush_updates(",
            "fn create_storage_buffer(",
            "#[cfg(test)]",
            "mod tests;",
        ],
    );

    for moved_test_anchor in [
        "fn render_gpu_scene_static_scene_second_frame_uploads_zero_bytes(",
        "fn render_gpu_scene_single_moving_entity_uploads_only_its_entry(",
        "fn render_gpu_scene_light_buffer_grows_and_skips_unchanged_uploads(",
        "fn test_backend(",
        "fn test_gpu_scene(",
        "fn sync_test_entry(",
        "fn test_primitive_data(",
        "fn test_instance_data(",
        "fn test_light_data(",
    ] {
        assert!(
            !root.contains(moved_test_anchor),
            "GPUScene parent should delegate test anchor `{moved_test_anchor}` to tests.rs"
        );
    }

    assert_contains_all(
        "GPUScene tests child owns upload, diff-update, light-buffer, and headless fixtures",
        &tests,
        &[
            "use super::*;",
            "fn render_gpu_scene_static_scene_second_frame_uploads_zero_bytes(",
            "fn render_gpu_scene_single_moving_entity_uploads_only_its_entry(",
            "fn render_gpu_scene_light_buffer_grows_and_skips_unchanged_uploads(",
            "fn test_backend(",
            "fn test_gpu_scene(",
            "fn sync_test_entry(",
            "GPU_SCENE_INVALID_PAYLOAD_SLOT",
        ],
    );

    for (path, source) in [
        ("gpu_scene/gpu_scene.rs", root.as_str()),
        ("gpu_scene/gpu_scene/tests.rs", tests.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay under the R1.4 owner budget after the test split, got {line_count}"
        );
    }

    for (label, doc) in [
        ("Plan 03", plan_03.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("GPUScene docs", gpu_scene_doc.as_str()),
        ("mesh pass docs", mesh_pass_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            doc,
            &[
                "GPUScene tests owner split",
                "render_plan03_gpu_scene_tests_owner_split_static_passed_cargo_deferred_active_compile_lane",
                "graphics/scene/gpu_scene/gpu_scene.rs",
                "graphics/scene/gpu_scene/gpu_scene/tests.rs",
                "runtime_15_gpu_scene_tests_are_child_owner",
            ],
        );
    }
}
