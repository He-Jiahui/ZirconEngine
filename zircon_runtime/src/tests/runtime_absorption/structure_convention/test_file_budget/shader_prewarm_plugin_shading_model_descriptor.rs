use super::*;

const STATUS: &str =
    "render_plan08_plugin_shading_model_descriptor_registration_typecheck_python_passed_libtest_blocked_by_ui_input_error";

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
    let build_plugin_tests = read_repo("tools/tests/test_zircon_build_plugin_carriers.py");
    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08-material-shader-permutation.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let package_manifest_doc = read_repo("docs/zircon_runtime/plugin/package_manifest.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let session_doc = read_repo(".codex/sessions/20260617-0926-render-hzb-progress.md");

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
        ],
    );
    assert_contains_all(
        "build tool derives selected plugin prewarm ids from descriptors",
        &build_tool,
        &[
            "collect_shading_model_descriptor_id_specs",
            "data.get(\"shading_models\", [])",
            "shader_shading_model_ids = tuple",
        ],
    );

    for (path, source) in [
        (
            "graphics/material/shading_models/registry.rs",
            shading_model_registry.as_str(),
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
