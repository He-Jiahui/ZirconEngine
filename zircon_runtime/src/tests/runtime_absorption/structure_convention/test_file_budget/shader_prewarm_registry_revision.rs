use super::*;

const STATUS: &str =
    "render_plan08_asset_root_resource_registry_revision_overlay_static_passed_cargo_timeout_no_result";

#[test]
fn runtime_15_shader_prewarm_resource_registry_revision_overlay_is_wired() {
    let args = read_runtime_src("bin/zircon_shader_prewarm/args.rs");
    let run = read_runtime_src("bin/zircon_shader_prewarm/run.rs");
    let manifest = read_runtime_src("bin/zircon_shader_prewarm/manifest.rs");
    let registry = read_runtime_src("bin/zircon_shader_prewarm/manifest/resource_registry.rs");
    let tests = read_runtime_src("bin/zircon_shader_prewarm/manifest/tests.rs");
    let build_tool = read_repo("tools/zircon_build.py");
    let build_prewarm = read_repo("tools/zircon_build_shader_prewarm.py");
    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08-material-shader-permutation.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let build_tool_doc = read_repo("docs/cli-and-tooling/zircon-build-tool.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let session_doc = read_repo(".codex/sessions/20260617-0926-render-hzb-progress.md");

    assert_contains_all(
        "shader prewarm CLI accepts exported resource registry revisions",
        &args,
        &[
            "pub resource_registry: Option<PathBuf>",
            "\"--resource-registry\"",
            "shader_prewarm_args_parse_resource_registry_path",
        ],
    );
    assert_contains_all(
        "shader prewarm run forwards resource registry overlay",
        &run,
        &[
            "ShaderPrewarmResourceRegistryOverlay::read",
            "asset_root_manifest_with_resource_registry_revisions",
            "resource_registry.as_ref()",
        ],
    );
    assert_contains_all(
        "asset-root manifest consumes registry revisions without owning registry export",
        &manifest,
        &[
            "pub(crate) mod resource_registry;",
            "asset_root_manifest_with_resource_registry_revisions",
            "registry.revision_for(resource_id, &stable_label)",
            "asset_scan_revision_from_source_hash",
        ],
    );
    assert_contains_all(
        "resource registry overlay owner decodes shader ResourceRecord revisions",
        &registry,
        &[
            "ShaderPrewarmResourceRegistryOverlay",
            "ResourceRecord",
            "ResourceKind::Shader",
            "record.revision == 0",
            "revision_for",
            "serde_json::from_value::<Vec<ResourceRecord>>",
        ],
    );
    assert_contains_all(
        "resource registry overlay focused tests",
        &tests,
        &[
            "shader_prewarm_asset_root_manifest_uses_resource_registry_revision_overlay",
            "ResourceRecord::new",
            "ResourceLocator::parse(\"res://shaders/example\")",
            "record.revision = 77",
            "material_revision == 77",
        ],
    );
    assert_contains_all(
        "zircon build forwards resource registry prewarm path",
        &build_tool,
        &[
            "--shader-resource-registry",
            "shader_resource_registry: Path | None",
            "resolve_optional_path(args.shader_resource_registry)",
        ],
    );
    assert_contains_all(
        "zircon build shader prewarm helper forwards resource registry path",
        &build_prewarm,
        &[
            "shader resource registry",
            "command.extend([\"--resource-registry\", str(config.shader_resource_registry)])",
        ],
    );

    for (path, source) in [
        ("bin/zircon_shader_prewarm/manifest.rs", manifest.as_str()),
        (
            "bin/zircon_shader_prewarm/manifest/resource_registry.rs",
            registry.as_str(),
        ),
        (
            "bin/zircon_shader_prewarm/manifest/tests.rs",
            tests.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 owner budget; got {line_count} lines"
        );
    }

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
                "Asset-root resource registry revision overlay",
                STATUS,
                "bin/zircon_shader_prewarm/manifest/resource_registry.rs",
                "shader_prewarm_asset_root_manifest_uses_resource_registry_revision_overlay",
                "runtime_15_shader_prewarm_resource_registry_revision_overlay_is_wired",
            ],
        );
    }
}
