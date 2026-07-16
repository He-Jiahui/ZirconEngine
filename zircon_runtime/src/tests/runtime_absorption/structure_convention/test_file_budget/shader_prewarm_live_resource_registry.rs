use super::*;

const STATUS: &str = "render_plan08_live_resource_manager_shader_registry_export_focused_tests_passed_renderdoc_deferred";

#[test]
fn runtime_15_shader_prewarm_live_resource_manager_registry_export_is_wired() {
    let manager_mod = read_runtime_src("core/resource/manager/mod.rs");
    let registry_export = read_runtime_src("core/resource/manager/registry_export.rs");
    let prewarm_registry =
        read_runtime_src("bin/zircon_shader_prewarm/manifest/resource_registry.rs");
    let tests = read_runtime_src("bin/zircon_shader_prewarm/manifest/tests.rs");
    let manifest_registry_tests =
        read_runtime_src("bin/zircon_shader_prewarm/manifest/tests/resource_registry.rs");
    let plan_08 = read_repo(
        "docs/plans/_archive/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let resource_doc = read_repo("docs/zircon_runtime/core/resource.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");

    assert_contains_all(
        "ResourceManager owns live registry export module",
        &manager_mod,
        &["mod registry_export;"],
    );
    assert_contains_all(
        "ResourceManager exports deterministic ready records by kind",
        &registry_export,
        &[
            "ready_records_for_kind",
            "ResourceState::Ready",
            "record.revision != 0",
            "records.sort_by",
            "resource_manager_exports_ready_records_for_kind_with_live_revisions",
        ],
    );
    assert_contains_all(
        "shader prewarm consumes live ResourceManager shader records",
        &prewarm_registry,
        &[
            "shader_resource_records_from_manager",
            "ResourceManager",
            "ready_records_for_kind(ResourceKind::Shader)",
        ],
    );
    assert_contains_all(
        "focused test proves live ResourceManager revisions feed material revisions",
        &manifest_registry_tests,
        &[
            "shader_prewarm_resource_registry_overlay_uses_live_resource_manager_shader_revisions",
            "ResourceManager::new",
            "shader_resource_records_from_manager(&manager)",
            "request.key.material_revision == live_revision",
        ],
    );

    for (path, source) in [
        (
            "core/resource/manager/registry_export.rs",
            registry_export.as_str(),
        ),
        (
            "bin/zircon_shader_prewarm/manifest/resource_registry.rs",
            prewarm_registry.as_str(),
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
        ("resource doc", resource_doc.as_str()),
        ("shader doc", shader_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Live ResourceManager shader registry export",
                STATUS,
                "ready_records_for_kind",
                "shader_prewarm_resource_registry_overlay_uses_live_resource_manager_shader_revisions",
                "runtime_15_shader_prewarm_live_resource_manager_registry_export_is_wired",
            ],
        );
    }
}
