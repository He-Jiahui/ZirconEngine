use super::*;

const STATUS: &str =
    "render_plan08_plugin_shading_model_descriptor_registration_typecheck_python_passed_libtest_blocked_by_ui_input_error";
const DESCRIPTOR_EXPORT_STATUS: &str =
    "render_plan08_plugin_shading_model_descriptor_registry_export_static_passed_cargo_deferred";

#[test]
fn runtime_15_shader_prewarm_plugin_shading_model_descriptor_registration_is_wired() {
    let package_manifest = read_runtime_src("plugin/package_manifest/plugin_package_manifest.rs");
    let package_constructors = read_runtime_src("plugin/package_manifest/constructors.rs");
    let extension_registry =
        read_runtime_src("plugin/extension_registry/runtime_extension_registry.rs");
    let extension_register_metadata =
        read_runtime_src("plugin/extension_registry/register/metadata.rs");
    let extension_access_metadata =
        read_runtime_src("plugin/extension_registry/access/metadata.rs");
    let extension_error = read_runtime_src("plugin/extension_registry_error.rs");
    let extension_ownership = read_runtime_src("plugin/extension_registry/ownership.rs");
    let manifest_metadata = read_runtime_src(
        "plugin/runtime_plugin/registration_report/package_contributions/manifest_metadata.rs",
    );
    let catalog_extension =
        read_runtime_src("plugin/runtime_plugin/runtime_plugin_catalog/contributions/extension.rs");
    let shading_model_contract =
        read_runtime_src("core/framework/render/material/shading_model.rs");
    let shading_model_registry = read_runtime_src("graphics/material/shading_models/registry.rs");
    let package_manifest_tests =
        read_runtime_src("tests/plugin_extensions/package_manifest_declarations.rs");
    let extension_registry_tests =
        read_runtime_src("tests/plugin_extensions/extension_registry_metadata.rs");
    let static_manifest_fields = read_runtime_src(
        "tests/plugin_extensions/static_manifest_contracts/manifest_schema/field_sets.rs",
    );
    let static_manifest_nested = read_runtime_src(
        "tests/plugin_extensions/static_manifest_contracts/manifest_schema/nested.rs",
    );
    let build_tool = read_repo("tools/zircon_build.py");
    let build_plugin_packages = read_repo("tools/zircon_build_plugin_packages.py");
    let build_shader_descriptors = read_repo("tools/zircon_build_plugin_shader_descriptors.py");
    let prewarm_helper = read_repo("tools/zircon_build_shader_prewarm.py");
    let build_plugin_tests = read_repo("tools/tests/test_zircon_build_plugin_carriers.py");
    let prewarm_tests = read_repo("tools/tests/test_zircon_build_shader_prewarm.py");
    let prewarm_registry =
        read_runtime_src("bin/zircon_shader_prewarm/manifest/permutation_registry.rs");
    let prewarm_run = read_runtime_src("bin/zircon_shader_prewarm/run.rs");
    let prewarm_errors = read_runtime_src("bin/zircon_shader_prewarm/error.rs");
    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08-material-shader-permutation.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let package_manifest_doc = read_repo("docs/zircon_runtime/plugin/package_manifest.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let session_doc = read_repo(".codex/sessions/20260617-0926-render-hzb-progress.md");
    let active_session_doc =
        read_repo(".codex/sessions/20260628-0141-render-plan08-continuation.md");

    assert_contains_all(
        "package manifest owns custom shading model descriptors",
        &(package_manifest + &package_constructors),
        &[
            "pub shading_models: Vec<ShadingModelDescriptor>",
            "with_shading_model_descriptor",
            "with_shading_model_descriptors",
        ],
    );
    assert_contains_all(
        "runtime extension registry owns descriptor registration",
        &(extension_registry
            + &extension_register_metadata
            + &extension_access_metadata
            + &extension_error
            + &extension_ownership),
        &[
            "shading_models: TypedExtensionPoint<String, ShadingModelDescriptor>",
            "register_shading_model",
            "register_shading_model_for_owner",
            "InvalidShadingModel",
            "DuplicateShadingModel",
            "pub shading_models: Vec<ExtensionSlot>",
            "shading_model_entries",
        ],
    );
    assert_contains_all(
        "manifest contributions and catalog merge descriptor owners",
        &(manifest_metadata + &catalog_extension),
        &[
            "register_package_manifest_shading_models",
            "package_manifest.shading_models",
            "extensions.register_shading_model",
            "extensions.shading_model_entries",
            "registry.register_shading_model_for_owner",
        ],
    );
    assert_contains_all(
        "graphics registry keeps plugin descriptor id range distinct from built-ins",
        &(shading_model_contract + &shading_model_registry),
        &[
            "PluginIdReserved",
            "SHADING_MODEL_PLUGIN_ID_START",
            "register_plugin_descriptor",
            "shading_model_registry_rejects_plugin_descriptor_in_builtin_id_range",
        ],
    );
    assert_contains_all(
        "tests cover manifest schema, runtime registry, and build discovery",
        &(package_manifest_tests
            + &extension_registry_tests
            + &static_manifest_fields
            + &static_manifest_nested
            + &build_plugin_tests),
        &[
            "plugin_package_manifest_declares_custom_shading_model_descriptors",
            "weather_toon_shading_model",
            "KNOWN_SHADING_MODEL_FIELDS",
            "shading_models",
            "test_zircon_build_discovers_plugin_shading_model_descriptors_as_shader_ids",
            "shader_shading_model_descriptors",
        ],
    );
    assert_contains_all(
        "build tool derives selected plugin prewarm ids and registry descriptors through package and descriptor owners",
        &(build_tool + &build_plugin_packages + &build_shader_descriptors),
        &[
            "shader_shading_model_descriptors: tuple[dict[str, object], ...]",
            "collect_shading_model_descriptors",
            "shader_shading_model_descriptors=shader_shading_model_descriptors",
            "shading_model_descriptor_id_specs",
            "collect_shading_model_descriptor_id_specs",
            "_collect_descriptor_rows(manifest_path, data, \"shading_models\")",
            "shader_shading_model_ids = tuple",
        ],
    );
    assert_contains_all(
        "generated shader permutation registry exports selected plugin shading model descriptors",
        &(prewarm_helper + &prewarm_tests),
        &[
            "\"shading_model_descriptors\"",
            "shader_shading_model_descriptors(config)",
            "def shader_shading_model_descriptors",
            "shader_shading_model_descriptors=(descriptor,)",
            "test_generated_shader_permutation_registry_document_exports_selected_plugin_shading_model_descriptors",
        ],
    );
    let prewarm_overlay_sources = format!("{prewarm_registry}{prewarm_run}{prewarm_errors}");
    assert_contains_all(
        "zircon shader prewarm overlay accepts selected plugin shading model descriptors",
        &prewarm_overlay_sources,
        &[
            "ShadingModelDescriptor",
            "shading_model_descriptors: BTreeMap<ShadingModelId, ShadingModelDescriptor>",
            "shading_model_descriptor_from_registry",
            "merge_shading_model_descriptor",
            "IncompatibleShadingModelDescriptor",
            "shader_prewarm_permutation_registry_merges_custom_shading_model_descriptors",
            "let mut shading_model_descriptors = BTreeMap::new();",
        ],
    );

    for (path, source) in [
        (
            "graphics/material/shading_models/registry.rs",
            shading_model_registry.as_str(),
        ),
        (
            "bin/zircon_shader_prewarm/manifest/permutation_registry.rs",
            prewarm_registry.as_str(),
        ),
        (
            "tools/zircon_build_plugin_packages.py",
            build_plugin_packages.as_str(),
        ),
        (
            "tools/zircon_build_plugin_shader_descriptors.py",
            build_shader_descriptors.as_str(),
        ),
        (
            "tools/tests/test_zircon_build_plugin_carriers.py",
            build_plugin_tests.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 900,
            "{path} should stay below the Runtime 15 owner budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("shader doc", shader_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("active render session doc", active_session_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Plugin shading-model descriptor registry export",
                DESCRIPTOR_EXPORT_STATUS,
                "shading_model_descriptors",
                "test_generated_shader_permutation_registry_document_exports_selected_plugin_shading_model_descriptors",
                "shader_prewarm_permutation_registry_merges_custom_shading_model_descriptors",
                "runtime_15_shader_prewarm_plugin_shading_model_descriptor_registration_is_wired",
            ],
        );
    }

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("shader doc", shader_doc.as_str()),
        ("package manifest doc", package_manifest_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("render session doc", session_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Plugin shading-model descriptor registration",
                STATUS,
                "test_zircon_build_discovers_plugin_shading_model_descriptors_as_shader_ids",
                "runtime_15_shader_prewarm_plugin_shading_model_descriptor_registration_is_wired",
            ],
        );
    }
}
