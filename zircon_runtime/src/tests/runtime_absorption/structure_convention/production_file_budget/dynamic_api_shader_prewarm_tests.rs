use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_dynamic_api_shader_prewarm_tests_are_child_owner() {
    let parent = read_runtime_src("dynamic_api/shader_prewarm.rs");
    let tests = read_runtime_src("dynamic_api/shader_prewarm/tests.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let build_tool_doc = read_repo("docs/cli-and-tooling/zircon-build-tool.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs",
    );
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m4_surface_cleanup.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m4_surface_cleanup.rs",
    );

    assert_contains_all(
        "dynamic API shader prewarm parent delegates tests and keeps runtime API",
        &parent,
        &[
            "#[path = \"shader_prewarm/tests.rs\"]",
            "mod tests;",
            "pub fn prewarm_shader_variants(",
            "pub fn prewarm_shader_variants_with_wgpu_module_validation(",
            "pub fn prewarm_shader_variants_with_wgpu_pipeline_validation(",
            "pub fn builtin_fallback_shader_prewarm_manifest()",
            "pub fn default_shader_variant_cache_root_for_project(",
        ],
    );
    for moved_test in [
        "fn builtin_fallback_shader_prewarm_manifest_uses_mesh_template_source()",
        "fn builtin_standard_material_shader_prewarm_manifest_projects_material_features()",
        "fn builtin_standard_material_shader_prewarm_manifest_projects_geometry_source()",
        "fn builtin_standard_material_prewarm_writes_restart_hits_and_wgpu_modules()",
        "fn builtin_standard_material_cache_validation_manifest()",
    ] {
        assert!(
            !parent.contains(moved_test),
            "dynamic_api/shader_prewarm.rs should delegate {moved_test} to dynamic_api/shader_prewarm/tests.rs"
        );
    }
    assert_contains_all(
        "dynamic API shader prewarm tests child owns builtin manifest and WGPU cache assertions",
        &tests,
        &[
            "fn builtin_fallback_shader_prewarm_manifest_uses_mesh_template_source()",
            "fn builtin_standard_material_shader_prewarm_manifest_projects_material_features()",
            "fn builtin_standard_material_shader_prewarm_manifest_projects_geometry_source()",
            "fn builtin_standard_material_prewarm_writes_restart_hits_and_wgpu_modules()",
            "fn builtin_standard_material_cache_validation_manifest()",
            "ShaderVariantCacheDiskKey::from_variant_key",
            "zircon-test-staged-builtin-standard-material-prewarm-shader",
        ],
    );

    for (path, source) in [
        ("dynamic_api/shader_prewarm.rs", parent.as_str()),
        ("dynamic_api/shader_prewarm/tests.rs", tests.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("build tool doc", build_tool_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M4 dynamic API shader prewarm tests owner split",
                "runtime_15_dynamic_api_shader_prewarm_tests_owner_split_static_passed_cargo_deferred",
                "dynamic_api/shader_prewarm.rs",
                "dynamic_api/shader_prewarm/tests.rs",
                "runtime_15_dynamic_api_shader_prewarm_tests_are_child_owner",
            ],
        );
    }
    assert_contains_all(
        "status-output status/date maps record dynamic API shader prewarm tests owner split",
        &format!("{status_map}\n{date_map}"),
        &[
            "Runtime 15 M4 dynamic API shader prewarm tests owner split",
            "runtime_15_dynamic_api_shader_prewarm_tests_owner_split_static_passed_cargo_deferred",
            "2026-07-01",
        ],
    );
}
