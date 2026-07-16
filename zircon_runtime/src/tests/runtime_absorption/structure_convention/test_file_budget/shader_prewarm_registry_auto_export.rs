use super::*;

const STATUS: &str =
    "render_plan08_shader_resource_registry_auto_export_focused_tests_passed_renderdoc_deferred";

#[test]
fn runtime_15_shader_prewarm_registry_auto_export_is_wired() {
    let args = read_runtime_src("bin/zircon_shader_prewarm/args.rs");
    let run = read_runtime_src("bin/zircon_shader_prewarm/run.rs");
    let manifest = read_runtime_src("bin/zircon_shader_prewarm/manifest.rs");
    let paths = read_runtime_src("bin/zircon_shader_prewarm/manifest/paths.rs");
    let registry = read_runtime_src("bin/zircon_shader_prewarm/manifest/resource_registry.rs");
    let project_record_export = read_runtime_src("asset/project/shader_resource_records.rs");
    let registry_tests =
        read_runtime_src("bin/zircon_shader_prewarm/manifest/resource_registry/tests.rs");
    let tests = read_runtime_src("bin/zircon_shader_prewarm/manifest/tests.rs");
    let manifest_registry_tests =
        read_runtime_src("bin/zircon_shader_prewarm/manifest/tests/resource_registry.rs");
    let build_tool = read_zircon_build_sources();
    let build_prewarm = read_repo("tools/zircon_build_shader_prewarm.py");
    let plan_08 = read_repo(
        "docs/plans/_archive/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let build_tool_doc = read_repo("docs/cli-and-tooling/zircon-build-tool.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");

    assert_contains_all(
        "shader prewarm CLI exports staged shader resource records",
        &args,
        &[
            "pub export_resource_registry: Option<PathBuf>",
            "\"--export-resource-registry\"",
            "shader_prewarm_args_parse_export_resource_registry_path",
        ],
    );
    assert_contains_all(
        "shader prewarm run exports records before asset-root manifest scan",
        &run,
        &[
            "export_shader_resource_registry_for_asset_roots",
            "shader_resource_records_from_asset_roots",
            "serde_json::json!({ \"resources\": records })",
            "ShaderPrewarmResourceRegistryOverlay::from_records",
        ],
    );
    assert_contains_all(
        "resource registry owner delegates staged shader-only records to asset/project",
        &registry,
        &[
            "project_shader_resource_records_from_asset_root",
            "project_shader_resource_records_from_asset_roots",
            "ShaderResourceRecordExportError",
            "impl From<ShaderResourceRecordExportError> for ShaderPrewarmResourceRegistryError",
        ],
    );
    assert_contains_all(
        "asset/project owner generates staged shader-only records",
        &project_record_export,
        &[
            "shader_resource_records_from_asset_root",
            "shader_resource_records_from_asset_roots",
            "deduplicate_shader_resource_records",
            "AssetMetaDocument::load",
            "asset_scan_revision_from_source_digest",
            "ResourceKind::Shader",
            "ResourceState::Ready",
            "ResourceId::from_asset_uuid",
        ],
    );
    assert_contains_all(
        "resource registry child tests cover multi-root export dedupe",
        &registry_tests,
        &[
            "shader_resource_records_from_asset_roots_deduplicates_duplicate_shader_records",
            "engine_assets",
            "plugin_assets",
            "assert_eq!(records.len(), 1)",
        ],
    );
    assert_contains_all(
        "manifest scan lets sidecar metadata own single-file shader sources",
        &manifest,
        &[
            "shader_source_from_zmeta",
            "has_sidecar_zmeta(&path)",
            "continue;",
        ],
    );
    assert_contains_all(
        "manifest path owner detects sidecar zmeta files",
        &paths,
        &[
            "has_sidecar_zmeta",
            "format!(\"{file_name}.zmeta\")",
            ".exists()",
        ],
    );
    assert_contains_all(
        "focused tests cover staged registry export handoff",
        &manifest_registry_tests,
        &[
            "shader_prewarm_asset_root_exports_shader_resource_records",
            "shader_resource_records_from_asset_root",
            "ResourceState::Ready",
            "assert_eq!(manifest.variants.len(), 6)",
            "request.key.material_revision == record.revision",
        ],
    );
    assert_contains_all(
        "zircon build owns automatic staged registry path",
        &build_tool,
        &[
            "shader_prewarm_resource_registry_path",
            "shader_resource_records.json",
            "exports a staged shader registry automatically",
        ],
    );
    assert_contains_all(
        "zircon build shader prewarm forwards auto export unless explicit registry wins",
        &build_prewarm,
        &[
            "shader resource registry export",
            "\"--export-resource-registry\"",
            "if config.shader_resource_registry",
            "\"--resource-registry\"",
        ],
    );

    for (path, source) in [
        ("bin/zircon_shader_prewarm/manifest.rs", manifest.as_str()),
        (
            "bin/zircon_shader_prewarm/manifest/paths.rs",
            paths.as_str(),
        ),
        (
            "bin/zircon_shader_prewarm/manifest/resource_registry.rs",
            registry.as_str(),
        ),
        (
            "asset/project/shader_resource_records.rs",
            project_record_export.as_str(),
        ),
        (
            "bin/zircon_shader_prewarm/manifest/resource_registry/tests.rs",
            registry_tests.as_str(),
        ),
        (
            "bin/zircon_shader_prewarm/manifest/tests.rs",
            tests.as_str(),
        ),
        (
            "bin/zircon_shader_prewarm/manifest/tests/resource_registry.rs",
            manifest_registry_tests.as_str(),
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
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Staged shader resource registry auto-export",
                STATUS,
                "shader_resource_records_from_asset_root",
                "shader_prewarm_asset_root_exports_shader_resource_records",
                "runtime_15_shader_prewarm_registry_auto_export_is_wired",
            ],
        );
    }
}
