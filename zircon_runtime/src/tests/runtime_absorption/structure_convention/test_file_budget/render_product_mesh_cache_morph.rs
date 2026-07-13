use super::*;

#[test]
fn runtime_15_render_product_mesh_cache_morph_tests_are_child_owners() {
    let parent = read_runtime_src("graphics/tests/render_product_mesh_cache/morph.rs");
    let direct_velocity =
        read_runtime_src("graphics/tests/render_product_mesh_cache/morph/direct_velocity.rs");
    let skinned_velocity =
        read_runtime_src("graphics/tests/render_product_mesh_cache/morph/skinned_velocity.rs");
    let velocity_png =
        read_runtime_src("graphics/tests/render_product_mesh_cache/morph/velocity_png.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/asset_budget_tests.rs",
    );
    let status_map = [
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/asset_budget_maps.rs",
        ),
    ]
    .join("\n");
    let date_map = [
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/asset_budget_maps.rs",
        ),
    ]
    .join("\n");

    assert_contains_all(
        "morph product parent mounts velocity child owners and keeps shared fixtures",
        &parent,
        &[
            "mod direct_velocity;",
            "mod skinned_velocity;",
            "mod velocity_png;",
            "fn render_product_direct_mesh_active_morph_weights_use_gpu_morphed_source",
            "fn render_product_direct_mesh_gpu_morph_matches_cpu_baked_reference_pixels",
            "fn render_product_skinned_mesh_gpu_morph_matches_cpu_baked_reference_pixels",
            "fn morph_velocity_quality_profile() -> RenderQualityProfile",
            "fn assert_scene_velocity_readback_nonzero",
        ],
    );
    for moved_owner in [
        "fn render_product_direct_mesh_morph_weight_change_writes_scene_velocity_pixels()",
        "fn direct_morph_velocity_extract(",
        "fn direct_morph_velocity_mesh_snapshot(",
        "const DIRECT_MORPH_VELOCITY_NODE_ID",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "graphics/tests/render_product_mesh_cache/morph.rs should delegate {moved_owner} to morph/direct_velocity.rs"
        );
    }
    assert_contains_all(
        "direct velocity child owns direct morph 0-to-1 velocity product path",
        &direct_velocity,
        &[
            "use super::*;",
            "fn render_product_direct_mesh_morph_weight_change_writes_scene_velocity_pixels()",
            "fn export_direct_morph_weight_velocity_product_png()",
            "fn capture_direct_morph_velocity_product() -> DirectMorphVelocityCapture",
            "fn direct_morph_velocity_extract(",
            "fn direct_morph_velocity_mesh_snapshot(",
            "const DIRECT_MORPH_VELOCITY_NODE_ID: u64 = 2601;",
            "DIRECT_MORPH_VELOCITY_PNG_STATUS",
            "runtime_render_plan08_direct_morph_weight_velocity_20260703.png",
            "direct morph weight 0 -> 1 should write object velocity from previous morph weights",
        ],
    );
    assert_contains_all(
        "skinned velocity child remains mounted beside direct velocity child",
        &skinned_velocity,
        &[
            "use super::*;",
            "fn render_product_skinned_mesh_morph_weight_change_writes_scene_velocity_pixels()",
            "fn export_skinned_morph_weight_velocity_product_png()",
            "fn capture_skinned_morph_velocity_product() -> SkinnedMorphVelocityCapture",
            "fn skinned_morph_velocity_extract(",
            "fn skinned_morph_velocity_mesh_snapshot(",
            "const SKINNED_MORPH_VELOCITY_NODE_ID: u64 = 2701;",
            "SKINNED_MORPH_VELOCITY_PNG_STATUS",
            "runtime_render_plan08_skinned_morph_weight_velocity_20260703.png",
        ],
    );
    assert_contains_all(
        "velocity PNG child owns scene-velocity RG16Float artifact encoding",
        &velocity_png,
        &[
            "fn save_scene_velocity_png(",
            "fn visualize_scene_velocity_rg16_float_bits(",
            "fn rg16_float_payload_bits(",
            "last_scene_velocity_readback_rg16_float_bytes_for_tests",
            "ImageFormat::Png",
        ],
    );

    for (path, source) in [
        (
            "graphics/tests/render_product_mesh_cache/morph.rs",
            parent.as_str(),
        ),
        (
            "graphics/tests/render_product_mesh_cache/morph/direct_velocity.rs",
            direct_velocity.as_str(),
        ),
        (
            "graphics/tests/render_product_mesh_cache/morph/skinned_velocity.rs",
            skinned_velocity.as_str(),
        ),
        (
            "graphics/tests/render_product_mesh_cache/morph/velocity_png.rs",
            velocity_png.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 render product mesh-cache morph tests child-owner split",
                "runtime_15_render_product_mesh_cache_morph_tests_child_owner_split_static_passed_cargo_deferred",
                "graphics/tests/render_product_mesh_cache/morph.rs",
                "graphics/tests/render_product_mesh_cache/morph/direct_velocity.rs",
                "runtime_15_render_product_mesh_cache_morph_tests_are_child_owners",
            ],
        );
    }
    assert_contains_all(
        "status-output status/date maps record render product mesh-cache morph tests split",
        &format!("{status_map}\n{date_map}"),
        &[
            "Runtime 15 M3 render product mesh-cache morph tests child-owner split",
            "runtime_15_render_product_mesh_cache_morph_tests_child_owner_split_static_passed_cargo_deferred",
            "2026-07-01",
        ],
    );
}
