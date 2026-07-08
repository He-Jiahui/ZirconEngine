use std::fs;
use std::path::Path;

use super::inventory::LIFECYCLE_FALLBACK_TESTS;

#[test]
fn runtime_06_vm_lifecycle_fallback_failure_tests_are_folder_backed() {
    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let vm_tests_source = include_str!("../../../script/vm/tests.rs");
    assert!(
        vm_tests_source.contains("mod lifecycle_failures;"),
        "Runtime 06 M1.2 fallback lifecycle test owner should be mounted by script/vm/tests.rs"
    );

    let lifecycle_tests_path = runtime_root.join("src/script/vm/tests/lifecycle_failures.rs");
    assert!(
        lifecycle_tests_path.exists(),
        "Runtime 06 M1.2 fallback lifecycle tests should live in a folder-backed script/vm test owner"
    );

    let lifecycle_tests_source =
        fs::read_to_string(&lifecycle_tests_path).unwrap_or_else(|error| {
            panic!("failed to read {}: {error}", lifecycle_tests_path.display())
        });
    for test_name in LIFECYCLE_FALLBACK_TESTS {
        assert!(
            lifecycle_tests_source.contains(test_name),
            "Runtime 06 M1.2 fallback lifecycle test `{test_name}` is missing"
        );
    }
    assert!(
        lifecycle_tests_source.contains("lifecycle:fallback"),
        "Runtime 06 M1.2 fallback lifecycle tests should not require the real ZrVM backend"
    );
}
