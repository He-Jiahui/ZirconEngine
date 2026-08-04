use super::*;

#[test]
fn runtime_15_scene_world_basics_tests_are_folder_backed() {
    let parent = read_runtime_src("scene/tests/world_basics.rs");
    let world_state = read_runtime_src("scene/tests/world_basics/world_state.rs");
    let render_extract = read_runtime_src("scene/tests/world_basics/render_extract.rs");
    let sprites = read_runtime_src("scene/tests/world_basics/sprites.rs");

    assert_contains_all(
        "scene world basics parent keeps shared imports and mounts children",
        &parent,
        &["mod render_extract;", "mod sprites;", "mod world_state;"],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "scene/tests/world_basics.rs should only keep shared imports and mount child owners"
    );
    for moved_test in [
        "fn world_bootstraps_with_renderable_defaults",
        "fn spawned_entities_have_unique_ids",
        "fn spawn_node_assigns_one_based_kind_ordinals",
        "fn hierarchy_updates_world_transform",
        "fn updated_transform_is_reflected_in_render_extract",
        "fn mesh_renderer_sort_fields_feed_geometry_phase_queue",
        "fn project_roundtrip_preserves_imported_meshes",
        "fn node_record_roundtrip_restores_same_entity",
        "fn recursive_remove_returns_parent_and_children_records",
        "fn set_parent_checked_rejects_hierarchy_cycles",
        "fn render_extract_separates_directional_point_and_spot_lights",
        "fn render_product_pbr_world_frame_extract_exposes_authored_ambient_and_rect_light_slots",
        "fn render_product_sprite_world_frame_extract_exposes_runtime_sprite_components",
        "fn render_product_sprite_world_frame_extract_filters_by_camera_layers",
        "fn render_product_sprite_mesh2d_component_does_not_count_as_particle_sprite",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved scene world-basics test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "world-state child owns bootstrap, hierarchy, project, and node-record coverage",
        &world_state,
        &[
            "fn world_bootstraps_with_renderable_defaults",
            "fn spawned_entities_have_unique_ids",
            "fn spawn_node_assigns_one_based_kind_ordinals",
            "fn hierarchy_updates_world_transform",
            "fn project_roundtrip_preserves_imported_meshes",
            "fn node_record_roundtrip_restores_same_entity",
            "fn recursive_remove_returns_parent_and_children_records",
            "fn set_parent_checked_rejects_hierarchy_cycles",
        ],
    );
    assert_contains_all(
        "render-extract child owns transform, phase, and light extraction coverage",
        &render_extract,
        &[
            "fn updated_transform_is_reflected_in_render_extract",
            "fn mesh_renderer_sort_fields_feed_geometry_phase_queue",
            "fn render_extract_separates_directional_point_and_spot_lights",
            "fn render_product_pbr_world_frame_extract_exposes_authored_ambient_and_rect_light_slots",
        ],
    );
    assert_contains_all(
        "sprites child owns 2D sprite extraction coverage",
        &sprites,
        &[
            "fn render_product_sprite_world_frame_extract_exposes_runtime_sprite_components",
            "fn render_product_sprite_world_frame_extract_filters_by_camera_layers",
            "fn render_product_sprite_mesh2d_component_does_not_count_as_particle_sprite",
            "fn texture_handle",
        ],
    );

    let child_test_total = [
        world_state.as_str(),
        render_extract.as_str(),
        sprites.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 15,
        "scene world-basics children should preserve all 15 parent tests"
    );

    for (path, source) in [
        ("scene/tests/world_basics.rs", parent.as_str()),
        (
            "scene/tests/world_basics/world_state.rs",
            world_state.as_str(),
        ),
        (
            "scene/tests/world_basics/render_extract.rs",
            render_extract.as_str(),
        ),
        ("scene/tests/world_basics/sprites.rs", sprites.as_str()),
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
    let render_extract_doc = read_repo("docs/zircon_runtime/scene/render_extract.md");
    let inspection_doc = read_repo("docs/zircon_runtime/scene/inspection.md");
}
