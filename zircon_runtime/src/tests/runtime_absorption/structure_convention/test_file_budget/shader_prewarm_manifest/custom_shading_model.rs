use super::*;

#[test]
fn runtime_15_shader_prewarm_custom_shading_model_id_is_wired() {
    let args = read_runtime_src("bin/zircon_shader_prewarm/args.rs");
    let run = read_runtime_src("bin/zircon_shader_prewarm/run.rs");
    let manifest = read_runtime_src("bin/zircon_shader_prewarm/manifest.rs");
    let material_sources =
        read_runtime_src("bin/zircon_shader_prewarm/manifest/material_sources.rs");
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
        "shader prewarm CLI exposes explicit custom shading model ids",
        &args,
        &[
            "pub shading_model_ids: BTreeMap<String, ShadingModelId>",
            "\"--shading-model-id\"",
            "parse_shading_model_id",
            "normalized_custom_shading_model_token",
            "normalized_shading_model_ids",
            "SHADING_MODEL_PLUGIN_ID_START",
            "shader_prewarm_args_parse_custom_shading_model_plugin_ids",
            "shader_prewarm_args_reject_builtin_shading_model_id_range",
        ],
    );
    assert_contains_all(
        "shader prewarm run forwards custom shading model ids to asset-root manifests",
        &run,
        &[
            "asset_root_manifest_with_resource_registry_revisions",
            "&shading_model_ids",
        ],
    );
    assert_contains_all(
        "shader prewarm manifest resolves custom material lighting models through explicit ids",
        &(manifest.clone() + &material_sources),
        &[
            "asset_root_manifest_for_quality_tiers_geometry_sources_and_shading_model_ids",
            "shading_model_ids: &BTreeMap<String, ShadingModelId>",
            "material_shading_model_id(&material, shading_model_ids)",
            "lighting_model.as_token().trim().to_ascii_lowercase()",
        ],
    );
    assert_contains_all(
        "shader prewarm manifest tests cover custom shading model ids",
        &tests,
        &[
            "shader_prewarm_asset_root_manifest_maps_custom_shading_model_plugin_ids",
            "custom:subsurface",
            "SHADING_MODEL_PLUGIN_ID_START",
            "request.key.shading_model",
        ],
    );
    assert_contains_all(
        "zircon build forwards custom shading model id prewarm dimensions",
        &build_tool,
        &[
            "--shader-shading-model-id",
            "shader_shading_model_ids: tuple[str, ...]",
            "parse_shader_shading_model_ids",
        ],
    );
    assert_contains_all(
        "zircon build shader prewarm helper forwards custom shading model id arguments",
        &build_prewarm,
        &[
            "parse_shader_shading_model_ids",
            "def build_shader_prewarm_command(config)",
            "command.extend([\"--shading-model-id\", shading_model_id])",
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
                "Asset-root custom shading-model id prewarm",
                "render_plan08_asset_root_custom_shading_model_id_prewarm_static_passed_cargo_deferred_implementation_cadence",
                "bin/zircon_shader_prewarm/args.rs",
                "bin/zircon_shader_prewarm/manifest.rs",
                "tools/zircon_build.py",
                "shader_prewarm_asset_root_manifest_maps_custom_shading_model_plugin_ids",
                "runtime_15_shader_prewarm_custom_shading_model_id_is_wired",
            ],
        );
    }
}
