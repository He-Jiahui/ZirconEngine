use super::*;

const STATUS: &str = "render_plan08_plugin_shader_permutation_registry_auto_export_focused_tests_passed_renderdoc_deferred";
const EXPORT_CONTRACT_STATUS: &str =
    "render_plan08_plugin_shader_permutation_registry_export_contract_python_passed_cargo_deferred";

#[test]
fn runtime_15_shader_prewarm_plugin_permutation_registry_auto_export_is_wired() {
    let package_manifest = read_runtime_src("plugin/package_manifest/plugin_package_manifest.rs");
    let shader_permutation_manifest =
        read_runtime_src("plugin/package_manifest/plugin_shader_permutation_manifest.rs");
    let package_constructors = read_runtime_src("plugin/package_manifest/constructors.rs");
    let virtual_geometry_plugin =
        read_repo("zircon_plugins/virtual_geometry/runtime/src/plugin.rs");
    let virtual_geometry_static_manifest = read_repo("zircon_plugins/virtual_geometry/plugin.toml");
    let build_tool = read_zircon_build_sources();
    let build_prewarm = read_repo("tools/zircon_build_shader_prewarm.py");
    let build_plugin_tests = read_repo("tools/tests/test_zircon_build_plugin_carriers.py");
    let build_prewarm_tests = read_repo("tools/tests/test_zircon_build_shader_prewarm.py");
    let build_prewarm_registry_contract_tests =
        read_repo("tools/tests/test_zircon_build_shader_permutation_registry_contract.py");
    let plan_08 = read_repo(
        "docs/plans/_archive/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let build_tool_doc = read_repo("docs/cli-and-tooling/zircon-build-tool.md");
    let plugin_readme = read_repo("zircon_plugins/README.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");

    assert_contains_all(
        "plugin package manifest owns shader permutation metadata",
        &package_manifest,
        &[
            "pub shader_permutation: PluginShaderPermutationManifest",
            "skip_serializing_if = \"PluginShaderPermutationManifest::is_empty\"",
        ],
    );
    assert_contains_all(
        "shader permutation manifest DTO separates ids by dimension",
        &shader_permutation_manifest,
        &[
            "pub struct PluginShaderPermutationManifest",
            "pub geometry_source_ids: Vec<PluginShaderPermutationIdManifest>",
            "pub shading_model_ids: Vec<PluginShaderPermutationIdManifest>",
            "pub struct PluginShaderPermutationIdManifest",
            "pub fn is_empty(&self) -> bool",
        ],
    );
    assert_contains_all(
        "manifest constructors expose explicit shader id appenders",
        &package_constructors,
        &[
            "with_shader_geometry_source_id",
            "with_shader_shading_model_id",
            "self.shader_permutation",
        ],
    );
    assert_contains_all(
        "virtual geometry declares its plugin range geometry source id",
        &virtual_geometry_plugin,
        &[
            "VIRTUAL_GEOMETRY_SHADER_GEOMETRY_SOURCE_TOKEN",
            "custom:virtual_geometry",
            "VIRTUAL_GEOMETRY_SHADER_GEOMETRY_SOURCE_ID",
            "GEOMETRY_SOURCE_PLUGIN_ID_START",
            "with_shader_geometry_source_id",
        ],
    );
    assert_contains_all(
        "static virtual geometry manifest carries generated shader permutation metadata",
        &virtual_geometry_static_manifest,
        &[
            "[shader_permutation]",
            "[[shader_permutation.geometry_source_ids]]",
            "token = \"custom:virtual_geometry\"",
            "id = 4",
        ],
    );
    assert_contains_all(
        "build tool discovers selected plugin shader permutation ids",
        &build_tool,
        &[
            "shader_geometry_source_ids: tuple[str, ...]",
            "shader_shading_model_ids: tuple[str, ...]",
            "collect_shader_permutation_id_specs",
            "shader_permutation",
        ],
    );
    assert_contains_all(
        "build prewarm helper merges config plugins into generated registry",
        &build_prewarm,
        &[
            "shader_geometry_source_id_specs",
            "shader_shading_model_id_specs",
            "_combined_plugin_id_specs",
            "for plugin in getattr(config, \"plugins\", ())",
            "validate_shader_permutation_registry_export_contract",
            "_validate_expected_shader_id_specs",
        ],
    );
    assert_contains_all(
        "build prewarm orchestration validates generated registry before run",
        &build_tool,
        &[
            "validate_shader_permutation_registry_export_contract",
            "permutation_registry_path = write_generated_shader_permutation_registry(config)",
            "if permutation_registry_path is not None",
        ],
    );
    assert_contains_all(
        "python tests cover manifest discovery and generated registry handoff",
        &(build_plugin_tests + &build_prewarm_tests + &build_prewarm_registry_contract_tests),
        &[
            "test_zircon_build_discovers_plugin_shader_permutation_records",
            "test_generated_shader_permutation_registry_document_merges_selected_plugin_ids",
            "test_build_command_uses_generated_shader_permutation_registry_for_selected_plugin_ids",
            "test_validate_generated_registry_requires_selected_plugin_ids",
            "test_prewarm_shaders_validates_generated_registry_before_run",
        ],
    );

    for (path, source) in [
        (
            "plugin/package_manifest/plugin_shader_permutation_manifest.rs",
            shader_permutation_manifest.as_str(),
        ),
        (
            "tools/zircon_build_shader_prewarm.py",
            build_prewarm.as_str(),
        ),
        (
            "tools/tests/test_zircon_build_shader_prewarm.py",
            build_prewarm_tests.as_str(),
        ),
        (
            "tools/tests/test_zircon_build_shader_permutation_registry_contract.py",
            build_prewarm_registry_contract_tests.as_str(),
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
        ("plugin README", plugin_readme.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Plugin shader permutation registry auto-export",
                STATUS,
                "Plugin shader permutation registry export contract",
                EXPORT_CONTRACT_STATUS,
                "test_zircon_build_discovers_plugin_shader_permutation_records",
                "test_validate_generated_registry_requires_selected_plugin_ids",
                "runtime_15_shader_prewarm_plugin_permutation_registry_auto_export_is_wired",
            ],
        );
    }
}
