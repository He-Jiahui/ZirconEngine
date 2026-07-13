use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_sprite_build_vertices_tests_are_child_owner_split() {
    let parent = read_runtime_src("graphics/scene/scene_renderer/sprite/build_sprite_vertices.rs");
    let tests =
        read_runtime_src("graphics/scene/scene_renderer/sprite/build_sprite_vertices/tests.rs");

    let plan_14 =
        read_repo("docs/plans/zircon_runtime/render/14/2026-07-09-2d-stack-output-records.md");
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let render_product_submit = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");
    let sprite_docs = read_repo("docs/zircon_runtime/core/framework/render/sprite.md");

    assert_contains_all(
        "sprite vertex build parent keeps production owner and test mount",
        &parent,
        &[
            "pub(crate) fn build_sprite_vertices(",
            "fn sprite_image_vertices(",
            "fn sprite_image_slices(",
            "fn sliced_image_slices(",
            "fn tile_slice(",
            "#[cfg(test)]\nmod tests;",
        ],
    );

    for moved_test in [
        "fn build_sprite_vertices_routes_transparent3d_to_transparent3d_phase(",
        "fn build_sprite_vertices_filters_sprites_by_selected_camera_layers(",
        "fn sprite_image_vertices_tile_custom_size_into_repeated_quads(",
        "fn sprite_image_vertices_slice_custom_size_into_nine_regions(",
        "fn sprite_image_slices_cap_excessive_tile_subdivision(",
        "fn sprite_image_slices_fill_center_crops_source_rect(",
        "fn sprite_image_vertices_scale_fill_remains_single_quad(",
    ] {
        assert!(
            !parent.contains(moved_test),
            "sprite build-vertices parent should not own moved test `{moved_test}`"
        );
        assert!(
            tests.contains(moved_test),
            "sprite build-vertices test owner should contain moved test `{moved_test}`"
        );
    }

    assert_contains_all(
        "sprite build-vertices test owner keeps private helper coverage",
        &tests,
        &[
            "use super::*;",
            "include_str!(\"../build_sprite_vertices.rs\")",
            "RenderLayerSet::layer(40)",
            "RenderSpriteImageMode::tiled(true, true, 1.0)",
            "RenderSpriteImageMode::Sliced(RenderSpriteSlicer",
            "fn test_sprite(",
            "fn empty_sprite_extract(",
        ],
    );

    for (path, source) in [
        (
            "scene_renderer/sprite/build_sprite_vertices.rs",
            parent.as_str(),
        ),
        (
            "scene_renderer/sprite/build_sprite_vertices/tests.rs",
            tests.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay under the R1.4 owner budget after the sprite build-vertices test split, got {line_count}"
        );
    }

    for (label, doc) in [
        ("Plan 14", &plan_14),
        ("render index", &render_index),
        ("review findings", &review_findings),
        ("structure convention", &structure_convention),
        ("render product submit docs", &render_product_submit),
        ("sprite docs", &sprite_docs),
    ] {
        assert_contains_all(
            label,
            doc,
            &[
                "Sprite build vertices test owner split",
                "render_plan14_sprite_build_vertices_test_owner_split_static_passed_cargo_deferred_active_compile_lane",
                "graphics/scene/scene_renderer/sprite/build_sprite_vertices.rs",
                "graphics/scene/scene_renderer/sprite/build_sprite_vertices/tests.rs",
                "runtime_15_sprite_build_vertices_tests_are_child_owner_split",
            ],
        );
    }
}
