use super::*;

#[test]
fn runtime_15_scene_render_extract_tests_are_folder_backed() {
    let parent = read_runtime_src("scene/tests/render_extract.rs");
    let camera_order = read_runtime_src("scene/tests/render_extract/camera_order.rs");
    let direct_sections = read_runtime_src("scene/tests/render_extract/direct_sections.rs");
    let level_source_guards = read_runtime_src("scene/tests/render_extract/level_source_guards.rs");
    let lighting_postprocess =
        read_runtime_src("scene/tests/render_extract/lighting_postprocess.rs");
    let particles = read_runtime_src("scene/tests/render_extract/particles.rs");

    assert_contains_all(
        "scene render extract parent mounts folder-backed children",
        &parent,
        &[
            "mod camera_order;",
            "mod direct_sections;",
            "mod level_source_guards;",
            "mod lighting_postprocess;",
            "mod particles;",
            "fn spawn_camera_on_layer",
            "fn assert_runtime_submit_tree_excludes_snapshot_adapters",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "scene/tests/render_extract.rs should only keep shared helpers and mount child owners"
    );
    for moved_test in [
        "world_render_frame_extract_populates_direct_renderer_sections",
        "render_frame_extract_collects_dynamic_particle_sprites_by_camera_layers",
        "render_frame_extract_filters_lights_by_camera_layers",
        "world_render_camera_order_report_projects_active_scene_cameras",
        "render_frame_extract_snapshot_adapters_are_not_scene_production_paths",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved scene render-extract test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "direct sections child owns canonical frame extraction tests",
        &direct_sections,
        &[
            "fn world_render_frame_extract_populates_direct_renderer_sections",
            "fn render_frame_extract_selects_mesh_lod_by_camera_distance",
            "fn inactive_camera_render_frame_extract_keeps_view_but_removes_scene_payload",
            "fn hierarchy_inactive_camera_render_frame_extract_keeps_view_but_removes_scene_payload",
            "fn render_frame_extract_filters_meshes_sprites_and_visibility_by_camera_layers",
        ],
    );
    assert_contains_all(
        "particles child owns scene particle extraction tests",
        &particles,
        &[
            "fn render_frame_extract_collects_dynamic_particle_sprites_by_camera_layers",
            "fn render_frame_extract_collects_dynamic_particle_gpu_frames_by_camera_layers",
            "fn render_frame_extract_collects_world_hud_health_bars_as_scene_particles",
        ],
    );
    assert_contains_all(
        "lighting/postprocess child owns lights, request layers, and volume tests",
        &lighting_postprocess,
        &[
            "fn render_frame_extract_filters_lights_by_camera_layers",
            "fn explicit_camera_request_layers_override_scene_camera_layers_for_direct_frame_extract",
            "fn render_frame_extract_carries_scene_post_process_volumes_for_camera_layers",
            "fn inactive_post_process_volume_hierarchy_is_excluded_from_frame_extract",
        ],
    );
    assert_contains_all(
        "camera order child owns scheduling and custom-target tests",
        &camera_order,
        &[
            "fn world_render_camera_order_report_projects_active_scene_cameras",
            "fn render_frame_extract_carries_scene_camera_order_report_for_scene_camera",
            "fn explicit_camera_render_frame_extract_has_no_scene_camera_order_report",
            "fn render_frame_extract_keeps_custom_target_layer_geometry_for_visibility_views",
        ],
    );
    assert_contains_all(
        "level/source guard child owns level-system and source-guard tests",
        &level_source_guards,
        &[
            "fn level_system_render_extract_uses_world_direct_path_and_merges_animation_poses",
            "fn render_frame_extract_snapshot_adapters_are_not_scene_production_paths",
            "fn render_view_extract_keeps_selected_scene_camera_descriptor_when_inactive",
        ],
    );

    let child_test_total = [
        camera_order.as_str(),
        direct_sections.as_str(),
        level_source_guards.as_str(),
        lighting_postprocess.as_str(),
        particles.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 19,
        "scene render-extract children should preserve all 19 parent tests"
    );

    for (path, source) in [
        ("scene/tests/render_extract.rs", parent.as_str()),
        (
            "scene/tests/render_extract/camera_order.rs",
            camera_order.as_str(),
        ),
        (
            "scene/tests/render_extract/direct_sections.rs",
            direct_sections.as_str(),
        ),
        (
            "scene/tests/render_extract/level_source_guards.rs",
            level_source_guards.as_str(),
        ),
        (
            "scene/tests/render_extract/lighting_postprocess.rs",
            lighting_postprocess.as_str(),
        ),
        (
            "scene/tests/render_extract/particles.rs",
            particles.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let scene_doc = read_repo("docs/zircon_runtime/scene/render_extract.md");
}
