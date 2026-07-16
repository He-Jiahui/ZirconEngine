use super::*;

const STATUS: &str = "render_plan08_plugin_geometry_source_descriptor_registration_typecheck_python_cargo_check_passed_renderdoc_deferred";

#[test]
fn runtime_15_shader_prewarm_plugin_geometry_source_descriptor_registration_is_wired() {
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
    let virtual_geometry_plugin =
        read_repo("zircon_plugins/virtual_geometry/runtime/src/plugin.rs");
    let virtual_geometry_static_manifest = read_repo("zircon_plugins/virtual_geometry/plugin.toml");
    let virtual_geometry_tests = read_repo("zircon_plugins/virtual_geometry/runtime/src/tests.rs");
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
    let build_shader_descriptors = read_repo("tools/zircon_build_plugin_shader_descriptors.py");
    let build_plugin_tests = read_repo("tools/tests/test_zircon_build_plugin_carriers.py");
    let plan_08 = read_repo(
        "docs/plans/_archive/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let package_manifest_doc = read_repo("docs/zircon_runtime/plugin/package_manifest.md");
    let build_tool_doc = read_repo("docs/cli-and-tooling/zircon-build-tool.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");

    assert_contains_all(
        "package manifest owns custom geometry source descriptors",
        &(package_manifest + &package_constructors),
        &[
            "pub geometry_sources: Vec<GeometrySourceDescriptor>",
            "with_geometry_source_descriptor",
            "with_geometry_source_descriptors",
        ],
    );
    assert_contains_all(
        "runtime extension registry owns geometry source descriptor registration",
        &(extension_registry
            + &extension_register_metadata
            + &extension_access_metadata
            + &extension_error
            + &extension_ownership),
        &[
            "geometry_sources: TypedExtensionPoint<String, GeometrySourceDescriptor>",
            "register_geometry_source",
            "register_geometry_source_for_owner",
            "InvalidGeometrySource",
            "DuplicateGeometrySource",
            "pub geometry_sources: Vec<ExtensionSlot>",
            "geometry_source_entries",
        ],
    );
    assert_contains_all(
        "manifest contributions and catalog merge geometry source descriptor owners",
        &(manifest_metadata + &catalog_extension),
        &[
            "register_package_manifest_geometry_sources",
            "package_manifest.geometry_sources",
            "extensions.register_geometry_source",
            "extensions.geometry_source_entries",
            "registry.register_geometry_source_for_owner",
        ],
    );
    assert_contains_all(
        "virtual geometry declares descriptor owner and keeps legacy id row",
        &(virtual_geometry_plugin + &virtual_geometry_static_manifest + &virtual_geometry_tests),
        &[
            "virtual_geometry_source_descriptor",
            "VIRTUAL_GEOMETRY_SHADER_GEOMETRY_SOURCE_WGSL_INCLUDE",
            "with_geometry_source_descriptor",
            "[[geometry_sources]]",
            "[[shader_permutation.geometry_source_ids]]",
            "report.extensions.geometry_sources()",
        ],
    );
    assert_contains_all(
        "tests cover manifest schema, runtime registry, static manifest, and build discovery",
        &(package_manifest_tests
            + &extension_registry_tests
            + &static_manifest_fields
            + &static_manifest_nested
            + &build_plugin_tests),
        &[
            "plugin_package_manifest_declares_custom_geometry_source_descriptors",
            "weather_gpu_geometry_source",
            "KNOWN_GEOMETRY_SOURCE_FIELDS",
            "geometry_sources",
            "test_zircon_build_discovers_plugin_geometry_source_descriptors_as_shader_ids",
        ],
    );
    assert_contains_all(
        "build tool derives selected plugin prewarm geometry ids through the shader-descriptor owner",
        &(build_tool + &build_shader_descriptors),
        &[
            "collect_geometry_source_descriptor_id_specs",
            "_collect_descriptor_rows(manifest_path, data, \"geometry_sources\")",
            "shader_geometry_source_ids = tuple",
        ],
    );

    for (path, source) in [
        (
            "plugin/extension_registry/register/metadata.rs",
            extension_register_metadata.as_str(),
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
        ("package manifest doc", package_manifest_doc.as_str()),
        ("build tool doc", build_tool_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Plugin geometry-source descriptor registration",
                STATUS,
                "test_zircon_build_discovers_plugin_geometry_source_descriptors_as_shader_ids",
                "runtime_15_shader_prewarm_plugin_geometry_source_descriptor_registration_is_wired",
            ],
        );
    }
}
