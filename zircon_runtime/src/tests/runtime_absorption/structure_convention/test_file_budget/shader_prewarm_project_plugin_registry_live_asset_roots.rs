use super::*;

const STATUS: &str =
    "render_plan08_project_plugin_registry_live_asset_roots_static_passed_cargo_deferred";

#[test]
fn runtime_15_shader_prewarm_project_plugin_registry_live_asset_roots_are_wired() {
    let run = read_repo("zircon_runtime/src/bin/zircon_shader_prewarm/run.rs");
    let registry =
        read_repo("zircon_runtime/src/bin/zircon_shader_prewarm/manifest/resource_registry.rs");
    let project_record_export =
        read_repo("zircon_runtime/src/asset/project/shader_resource_records.rs");
    let manifest_registry_tests = read_repo(
        "zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests/resource_registry.rs",
    );
    let registry_tests = read_repo(
        "zircon_runtime/src/bin/zircon_shader_prewarm/manifest/resource_registry/tests.rs",
    );
    let plan_08 = read_repo(
        "docs/plans/_archive/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");

    assert_contains_all(
        "run path exports registry records before scanning project/plugin roots",
        &run,
        &[
            "export_shader_resource_registry_for_asset_roots",
            "shader_resource_records_from_asset_roots",
            "ShaderPrewarmResourceRegistryOverlay::from_records",
            "asset_root_manifest_with_resource_registry_revisions",
        ],
    );
    assert_contains_all(
        "registry owner delegates live asset-root export to asset/project",
        &registry,
        &[
            "project_shader_resource_records_from_asset_roots",
            "ShaderResourceRecordExportError",
            "impl From<ShaderResourceRecordExportError> for ShaderPrewarmResourceRegistryError",
        ],
    );
    assert_contains_all(
        "asset/project owner merges multiple asset roots into ready shader records",
        &project_record_export,
        &[
            "shader_resource_records_from_asset_roots",
            "deduplicate_shader_resource_records",
            "ResourceKind::Shader",
            "ResourceState::Ready",
        ],
    );
    assert_contains_all(
        "manifest fixture proves exported project/plugin revisions reach prewarm requests",
        &manifest_registry_tests,
        &[
            "shader_prewarm_project_and_plugin_asset_roots_use_exported_registry_revisions",
            "shader_resource_records_from_asset_roots",
            "project_root.clone()",
            "plugin_root.clone()",
            "merge_manifests",
            "source_label == label",
            "material_revision == record.revision",
        ],
    );
    assert_contains_all(
        "registry fixture proves project/plugin roots export distinct shader records",
        &registry_tests,
        &[
            "shader_resource_records_from_project_and_plugin_asset_roots_export_distinct_shader_sources",
            "package://virtual_geometry/shaders/plugin",
            "res://project/shaders/project",
            "records.len(), 2",
        ],
    );

    for (path, source) in [
        (
            "zircon_runtime/src/bin/zircon_shader_prewarm/manifest/tests/resource_registry.rs",
            manifest_registry_tests.as_str(),
        ),
        (
            "zircon_runtime/src/asset/project/shader_resource_records.rs",
            project_record_export.as_str(),
        ),
        (
            "zircon_runtime/src/bin/zircon_shader_prewarm/manifest/resource_registry/tests.rs",
            registry_tests.as_str(),
        ),
        (
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_live_asset_roots.rs",
            include_str!("shader_prewarm_project_plugin_registry_live_asset_roots.rs"),
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
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Project/plugin registry live asset-root export",
                STATUS,
                "runtime_15_shader_prewarm_project_plugin_registry_live_asset_roots_are_wired",
                "shader_prewarm_project_and_plugin_asset_roots_use_exported_registry_revisions",
                "shader_resource_records_from_project_and_plugin_asset_roots_export_distinct_shader_sources",
            ],
        );
    }
}
