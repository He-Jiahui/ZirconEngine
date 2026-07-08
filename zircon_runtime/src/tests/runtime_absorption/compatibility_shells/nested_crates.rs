#[test]
fn runtime_absorption_does_not_keep_nested_compatibility_shells() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));

    assert!(
        !runtime_root.join("crates").exists(),
        "zircon_runtime should not keep nested compatibility crates after absorption"
    );
}
