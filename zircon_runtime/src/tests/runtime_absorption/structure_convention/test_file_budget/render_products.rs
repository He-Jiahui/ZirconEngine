use super::*;

#[test]
fn runtime_15_render_camera_target_products_are_folder_backed() {
    let parent = read_runtime_src("graphics/tests/render_product_camera_targets.rs");
    let custom_target_root =
        read_runtime_src("graphics/tests/render_product_camera_targets/custom_target.rs");
    let custom_target_composite =
        read_runtime_src("graphics/tests/render_product_camera_targets/custom_target/composite.rs");
    let custom_target_material_sampling = read_runtime_src(
        "graphics/tests/render_product_camera_targets/custom_target/material_sampling.rs",
    );
    let custom_target_ordering =
        read_runtime_src("graphics/tests/render_product_camera_targets/custom_target/ordering.rs");
    let custom_target_viewport =
        read_runtime_src("graphics/tests/render_product_camera_targets/custom_target/viewport.rs");
    let primary_surface =
        read_runtime_src("graphics/tests/render_product_camera_targets/primary_surface.rs");
    let texture_target =
        read_runtime_src("graphics/tests/render_product_camera_targets/texture_target.rs");
    let fixture = read_runtime_src("graphics/tests/render_product_camera_targets/fixture.rs");
    let m4_parent = read_runtime_src("graphics/tests/m4_behavior_layers.rs");
    let m4_particles = read_runtime_src("graphics/tests/m4_behavior_layers/particles.rs");
    let m4_queue_override = read_runtime_src("graphics/tests/m4_behavior_layers/queue_override.rs");
    let m4_transparent3d = read_runtime_src("graphics/tests/m4_behavior_layers/transparent3d.rs");
    let plan_09 = read_repo("docs/plans/zircon_runtime/render/09/2026-07-09-camera-render-ordering-output-records.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let render_product_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert_contains_all(
        "camera-target product parent mounts folder-backed owners",
        &parent,
        &[
            "mod assertions;",
            "mod camera;",
            "mod custom_target;",
            "mod fixture;",
            "mod mesh;",
            "mod primary_surface;",
            "mod texture_target;",
        ],
    );
    assert_contains_all(
        "m4 behavior parent mounts folder-backed product owners",
        &m4_parent,
        &[
            "mod particles;",
            "mod queue_override;",
            "mod transparent3d;",
        ],
    );
    assert!(
        !m4_parent.contains("fn render_product_queue_override_reorders_draws"),
        "m4_behavior_layers.rs should mount the queue override product owner instead of defining render_product_queue_override_reorders_draws"
    );
    for moved_guard in [
        "fn render_product_dual_camera_rt_then_main",
        "fn custom_target_stacks_feed_later_primary_surface_materials_independently",
        "fn custom_target_viewport_regions_feed_later_primary_surface_sample",
        "fn custom_target_overlay_inherits_base_viewport_region_before_primary_sample",
        "fn custom_target_two_viewport_stacks_preserve_independent_composites_before_primary_sample",
        "fn custom_target_chain_feeds_later_texture_and_primary_surface_samples",
        "fn custom_target_late_producer_feeds_previous_frame_not_future_sample",
        "fn render_product_camera_render_order_swap_changes_composite",
        "fn render_product_overlay_stack_composites_over_base",
        "fn render_product_split_screen_viewports",
        "fn texture_target_overlay_camera_draws_layered_mesh_over_base_clear",
        "fn texture_target_overlay_camera_converts_linear_final_product_after_composite",
        "fn texture_target_stack_preserves_composite_when_primary_surface_renders_later",
    ] {
        assert!(
            !parent.contains(moved_guard),
            "render_product_camera_targets.rs should mount child product owners instead of defining {moved_guard}"
        );
    }

    assert_contains_all(
        "custom-target root mounts folder-backed product owners",
        &custom_target_root,
        &[
            "use super::assertions",
            "use super::camera",
            "use super::fixture::RenderFixture",
            "use super::mesh",
            "mod composite;",
            "mod material_sampling;",
            "mod ordering;",
            "mod viewport;",
        ],
    );
    for moved_guard in [
        "fn render_product_dual_camera_rt_then_main",
        "fn custom_target_stacks_feed_later_primary_surface_materials_independently",
        "fn custom_target_viewport_regions_feed_later_primary_surface_sample",
        "fn custom_target_overlay_inherits_base_viewport_region_before_primary_sample",
        "fn custom_target_two_viewport_stacks_preserve_independent_composites_before_primary_sample",
        "fn custom_target_chain_feeds_later_texture_and_primary_surface_samples",
        "fn custom_target_late_producer_feeds_previous_frame_not_future_sample",
    ] {
        assert!(
            !custom_target_root.contains(moved_guard),
            "custom_target.rs should mount custom-target product owners instead of defining {moved_guard}"
        );
    }
    assert_contains_all(
        "custom-target composite child owns wider viewport-stack product guard",
        &custom_target_composite,
        &[
            "use super::*;",
            "fn custom_target_two_viewport_stacks_preserve_independent_composites_before_primary_sample",
        ],
    );
    assert_contains_all(
        "custom-target material-sampling child owns RT sampling guards",
        &custom_target_material_sampling,
        &[
            "use super::*;",
            "fn render_product_dual_camera_rt_then_main",
            "fn custom_target_stacks_feed_later_primary_surface_materials_independently",
            "fn custom_target_chain_feeds_later_texture_and_primary_surface_samples",
        ],
    );
    assert_contains_all(
        "custom-target viewport child owns viewport product guards",
        &custom_target_viewport,
        &[
            "use super::*;",
            "fn custom_target_viewport_regions_feed_later_primary_surface_sample",
            "fn custom_target_overlay_inherits_base_viewport_region_before_primary_sample",
        ],
    );
    assert_contains_all(
        "custom-target ordering child owns previous-frame ordering guard",
        &custom_target_ordering,
        &[
            "use super::*;",
            "fn custom_target_late_producer_feeds_previous_frame_not_future_sample",
        ],
    );
    assert_contains_all(
        "primary surface child owns PrimarySurface product guards",
        &primary_surface,
        &[
            "fn render_product_camera_render_order_swap_changes_composite",
            "fn render_product_overlay_stack_composites_over_base",
            "fn render_product_split_screen_viewports",
        ],
    );
    assert_contains_all(
        "texture target child owns texture-target product guards",
        &texture_target,
        &[
            "fn texture_target_overlay_camera_draws_layered_mesh_over_base_clear",
            "fn texture_target_overlay_camera_converts_linear_final_product_after_composite",
            "fn texture_target_stack_preserves_composite_when_primary_surface_renders_later",
        ],
    );
    assert_contains_all(
        "camera-target fixture centralizes WGPU setup and linear RT fixture",
        &fixture,
        &[
            "fn configured_framework",
            "fn insert_linear_render_target_texture",
        ],
    );
    assert_contains_all(
        "m4 queue override child owns Plan 09 CO-M3 product guard",
        &m4_queue_override,
        &[
            "use super::{average_channel_in_region, centered_quad_transform, RenderFixture};",
            "fn render_product_queue_override_reorders_draws",
            "GeometryPhaseInput::new(10, 0, RenderMaterialAlphaMode::Opaque, 0.0)",
            ".with_render_queue(2_900)",
        ],
    );

    for (path, source) in [
        (
            "graphics/tests/render_product_camera_targets.rs",
            parent.as_str(),
        ),
        (
            "graphics/tests/render_product_camera_targets/custom_target.rs",
            custom_target_root.as_str(),
        ),
        (
            "graphics/tests/render_product_camera_targets/custom_target/composite.rs",
            custom_target_composite.as_str(),
        ),
        (
            "graphics/tests/render_product_camera_targets/custom_target/material_sampling.rs",
            custom_target_material_sampling.as_str(),
        ),
        (
            "graphics/tests/render_product_camera_targets/custom_target/ordering.rs",
            custom_target_ordering.as_str(),
        ),
        (
            "graphics/tests/render_product_camera_targets/custom_target/viewport.rs",
            custom_target_viewport.as_str(),
        ),
        (
            "graphics/tests/render_product_camera_targets/primary_surface.rs",
            primary_surface.as_str(),
        ),
        (
            "graphics/tests/render_product_camera_targets/texture_target.rs",
            texture_target.as_str(),
        ),
        (
            "graphics/tests/render_product_camera_targets/fixture.rs",
            fixture.as_str(),
        ),
        ("graphics/tests/m4_behavior_layers.rs", m4_parent.as_str()),
        (
            "graphics/tests/m4_behavior_layers/particles.rs",
            m4_particles.as_str(),
        ),
        (
            "graphics/tests/m4_behavior_layers/queue_override.rs",
            m4_queue_override.as_str(),
        ),
        (
            "graphics/tests/m4_behavior_layers/transparent3d.rs",
            m4_transparent3d.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 09", plan_09.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("render product submit doc", render_product_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "render_plan09_camera_target_custom_owner_split_static_passed",
                "render_plan09_custom_target_composite_source_guard_static_passed",
                "render_plan09_queue_override_product_source_guard_static_passed",
                "render_product_queue_override_reorders_draws",
                "m4_behavior_layers/queue_override.rs",
                "render_product_camera_targets/custom_target.rs",
                "render_product_camera_targets/custom_target/composite.rs",
                "render_product_camera_targets/custom_target/material_sampling.rs",
                "render_product_camera_targets/custom_target/ordering.rs",
                "render_product_camera_targets/custom_target/viewport.rs",
                "render_product_camera_targets/texture_target.rs",
                "render_product_camera_targets/primary_surface.rs",
            ],
        );
    }
}
