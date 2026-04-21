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
