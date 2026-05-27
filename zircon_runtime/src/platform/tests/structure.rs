#[test]
fn platform_root_stays_structural_after_module_split() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("platform")
            .join("mod.rs"),
    )
    .expect("platform mod source");

    for forbidden in [
        "pub struct PlatformConfig",
        "pub struct PlatformModule",
        "pub struct PlatformDriver",
        "pub struct PlatformManager",
        "pub fn module_descriptor(",
        "impl EngineModule for PlatformModule",
    ] {
        assert!(
            !source.contains(forbidden),
            "expected platform/mod.rs to stay structural after split, found `{forbidden}`"
        );
    }
}

#[test]
fn platform_tests_stay_folder_backed() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/platform");

    for relative in [
        "tests/mod.rs",
        "tests/app_feature_manifest.rs",
        "tests/backend_tokens.rs",
        "tests/cross_target.rs",
        "tests/desktop_defaults.rs",
        "tests/diagnostic_status_consistency.rs",
        "tests/diagnostic_keys.rs",
        "tests/diagnostic_metadata.rs",
        "tests/diagnostics.rs",
        "tests/event_loop_policy.rs",
        "tests/feature_gate_propagation.rs",
        "tests/feature_manifest.rs",
        "tests/feature_selection.rs",
        "tests/gamepad.rs",
        "tests/gestures.rs",
        "tests/headless.rs",
        "tests/headless_synthetic_input.rs",
        "tests/linux.rs",
        "tests/matrix_cross_product.rs",
        "tests/status_semantics.rs",
        "tests/structure.rs",
        "tests/target_topology.rs",
        "tests/target_modes.rs",
    ] {
        assert!(
            root.join(relative).exists(),
            "expected platform test module {relative} under {:?}",
            root
        );
    }
    assert!(
        !root.join("tests.rs").exists(),
        "platform tests should stay folder-backed instead of returning to an umbrella tests.rs file"
    );
}
