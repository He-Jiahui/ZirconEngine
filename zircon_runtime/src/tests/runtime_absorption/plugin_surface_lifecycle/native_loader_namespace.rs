use std::collections::BTreeSet;
use std::path::Path;

use super::inventory::NATIVE_LOADER_TEST_PATTERNS;
use super::support::{files_containing, native_root_import_leak_files};

#[test]
fn runtime_06_plugin_root_does_not_forward_native_loader_for_tests() {
    let plugin_root = include_str!("../../../plugin/mod.rs");

    assert!(
        !plugin_root.contains("native_plugin_loader::NativePluginLoader"),
        "plugin root must not retain a test-only NativePluginLoader forwarding export; use plugin::native instead"
    );
}

#[test]
fn runtime_06_native_loader_tests_use_isolated_plugin_native_namespace() {
    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let plugin_extension_tests = runtime_root
        .join("src")
        .join("tests")
        .join("plugin_extensions");

    let native_loader_test_files =
        files_containing(&plugin_extension_tests, NATIVE_LOADER_TEST_PATTERNS);
    let expected_native_loader_test_files = BTreeSet::from([
        "zircon_runtime/src/tests/plugin_extensions/export_build_plan/catalog_projection.rs"
            .to_string(),
        "zircon_runtime/src/tests/plugin_extensions/export_build_plan_native_dynamic.rs"
            .to_string(),
        "zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs".to_string(),
        "zircon_runtime/src/tests/plugin_extensions/native_plugin_loader/real_fixture.rs"
            .to_string(),
    ]);
    assert_eq!(
        native_loader_test_files, expected_native_loader_test_files,
        "native loader test files should stay isolated under plugin_extensions and mirror Runtime 06 M2.2"
    );

    let namespace_import_files = files_containing(
        &plugin_extension_tests,
        &[
            "crate::plugin::native::",
            "zircon_runtime::plugin::native::",
        ],
    );
    let expected_namespace_import_files = BTreeSet::from([
        "zircon_runtime/src/tests/plugin_extensions/export_build_plan_native_dynamic.rs"
            .to_string(),
        "zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs".to_string(),
        "zircon_runtime/src/tests/plugin_extensions/native_plugin_loader/real_fixture.rs"
            .to_string(),
    ]);
    assert_eq!(
        namespace_import_files, expected_namespace_import_files,
        "tests that import native loader symbols should use the isolated plugin::native namespace"
    );

    let native_root_import_leaks = native_root_import_leak_files(&plugin_extension_tests);
    assert!(
        native_root_import_leaks.is_empty(),
        "native loader tests must not import NativePlugin or ZIRCON_NATIVE_PLUGIN symbols from the plugin root: {native_root_import_leaks:?}"
    );
}
