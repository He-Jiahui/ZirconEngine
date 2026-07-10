use super::*;

#[test]
fn runtime_15_shader_prewarm_geometry_source_enumeration_is_wired() {
    let args = read_runtime_src("bin/zircon_shader_prewarm/args.rs");
    let run = read_runtime_src("bin/zircon_shader_prewarm/run.rs");
    let manifest = read_runtime_src("bin/zircon_shader_prewarm/manifest.rs");
    let tests = read_runtime_src("bin/zircon_shader_prewarm/manifest/tests.rs");
    let build_tool = read_repo("tools/zircon_build.py");
    let build_prewarm = read_repo("tools/zircon_build_shader_prewarm.py");
    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let build_tool_doc = read_repo("docs/cli-and-tooling/zircon-build-tool.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let session_doc = read_repo(".codex/sessions/20260617-0926-render-hzb-progress.md");

    assert_contains_all(
        "shader prewarm CLI exposes explicit geometry source enumeration",
        &args,
        &[
            "pub geometry_sources: Vec<GeometrySourceId>",
            "\"--geometry-source\"",
            "parse_geometry_source",
            "builtin_geometry_source_descriptors()",
            "normalized_geometry_sources",
            "shader_prewarm_args_default_to_static_geometry_source",
            "shader_prewarm_args_expand_all_builtin_geometry_sources",
        ],
    );
    assert_contains_all(
        "shader prewarm run forwards geometry sources to asset-root manifests",
        &run,
        &[
            "asset_root_manifest_with_resource_registry_revisions",
            "&geometry_sources",
        ],
    );
    assert_contains_all(
        "shader prewarm manifest expands geometry source dimensions",
        &manifest,
        &[
            "pub fn asset_root_manifest_for_quality_tiers_and_geometry_sources",
            "fn manifest_geometry_sources",
            "geometry_sources: &[GeometrySourceId]",
            "geometry_source: *geometry_source",
        ],
    );
    assert_contains_all(
        "shader prewarm manifest tests cover explicit geometry source expansion",
        &tests,
        &[
            "shader_prewarm_asset_root_manifest_expands_requested_geometry_sources",
            "GEOMETRY_SOURCE_ID_STATIC_MESH",
            "GEOMETRY_SOURCE_ID_SKINNED_MESH",
        ],
    );
    assert_contains_all(
        "zircon build forwards geometry source prewarm dimensions",
        &build_tool,
        &[
            "--shader-geometry-source",
            "shader_geometry_sources: tuple[str, ...]",
            "parse_shader_geometry_sources",
        ],
    );
    assert_contains_all(
        "zircon build shader prewarm helper forwards geometry source arguments",
        &build_prewarm,
        &[
            "def build_shader_prewarm_command(config)",
            "command.extend([\"--geometry-source\", geometry_source])",
        ],
    );

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("shader doc", shader_doc.as_str()),
        ("build tool doc", build_tool_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("render session doc", session_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Shader prewarm geometry-source enumeration",
                "render_plan08_shader_prewarm_geometry_source_enumeration_static_passed_cargo_deferred_implementation_cadence",
                "bin/zircon_shader_prewarm/args.rs",
                "bin/zircon_shader_prewarm/manifest.rs",
                "tools/zircon_build.py",
                "shader_prewarm_asset_root_manifest_expands_requested_geometry_sources",
                "runtime_15_shader_prewarm_geometry_source_enumeration_is_wired",
            ],
        );
    }
}

#[test]
fn runtime_15_shader_prewarm_custom_geometry_source_id_is_wired() {
    let args = read_runtime_src("bin/zircon_shader_prewarm/args.rs");
    let run = read_runtime_src("bin/zircon_shader_prewarm/run.rs");
    let manifest = read_runtime_src("bin/zircon_shader_prewarm/manifest.rs");
    let tests = read_runtime_src("bin/zircon_shader_prewarm/manifest/tests.rs");
    let build_tool = read_repo("tools/zircon_build.py");
    let build_prewarm = read_repo("tools/zircon_build_shader_prewarm.py");
    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let build_tool_doc = read_repo("docs/cli-and-tooling/zircon-build-tool.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let session_doc = read_repo(".codex/sessions/20260617-0926-render-hzb-progress.md");

    assert_contains_all(
        "shader prewarm CLI exposes explicit custom geometry source ids",
        &args,
        &[
            "pub geometry_source_ids: BTreeMap<String, GeometrySourceId>",
            "\"--geometry-source-id\"",
            "parse_geometry_source_id",
            "normalized_custom_geometry_source_token",
            "normalized_geometry_source_ids",
            "GEOMETRY_SOURCE_PLUGIN_ID_START",
            "shader_prewarm_args_parse_custom_geometry_source_plugin_ids",
            "shader_prewarm_args_reject_builtin_geometry_source_id_range",
        ],
    );
    assert_contains_all(
        "shader prewarm run forwards custom geometry source ids through the geometry dimension list",
        &run,
        &[
            "asset_root_manifest_with_resource_registry_revisions",
            "&geometry_sources",
        ],
    );
    assert_contains_all(
        "shader prewarm manifest accepts plugin-range geometry source dimensions",
        &manifest,
        &[
            "geometry_sources: &[GeometrySourceId]",
            "manifest_geometry_sources(geometry_sources)",
            "geometry_source: *geometry_source",
        ],
    );
    assert_contains_all(
        "shader prewarm manifest tests cover custom geometry source ids",
        &tests,
        &[
            "shader_prewarm_asset_root_manifest_expands_custom_geometry_source_plugin_ids",
            "GEOMETRY_SOURCE_PLUGIN_ID_START",
            "request.key.geometry_source.is_plugin_range()",
        ],
    );
    assert_contains_all(
        "zircon build forwards custom geometry source id prewarm dimensions",
        &build_tool,
        &[
            "--shader-geometry-source-id",
            "shader_geometry_source_ids: tuple[str, ...]",
            "parse_shader_geometry_source_ids",
        ],
    );
    assert_contains_all(
        "zircon build shader prewarm helper forwards custom geometry source id arguments",
        &build_prewarm,
        &[
            "parse_shader_geometry_source_ids",
            "def build_shader_prewarm_command(config)",
            "command.extend([\"--geometry-source-id\", geometry_source_id])",
        ],
    );

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("shader doc", shader_doc.as_str()),
        ("build tool doc", build_tool_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("render session doc", session_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Asset-root custom geometry-source id prewarm",
                "render_plan08_asset_root_custom_geometry_source_id_prewarm_typecheck_passed_test_timeout_no_result",
                "bin/zircon_shader_prewarm/args.rs",
                "bin/zircon_shader_prewarm/manifest.rs",
                "tools/zircon_build.py",
                "tools/zircon_build_shader_prewarm.py",
                "shader_prewarm_asset_root_manifest_expands_custom_geometry_source_plugin_ids",
                "runtime_15_shader_prewarm_custom_geometry_source_id_is_wired",
            ],
        );
    }
}
