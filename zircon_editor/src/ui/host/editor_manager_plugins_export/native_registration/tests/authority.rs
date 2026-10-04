#[test]
fn project_discovery_cannot_mint_native_execution_authority() {
    let source = include_str!("../authority.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("production source");
    assert!(!production.contains("capture_trusted_local_package("));
    assert!(!production.contains("NativePluginArtifactAuthority::from_expectations("));
    assert!(production.contains("Ok(NativePluginArtifactAuthority::deny_all())"));
    assert!(!production.contains("load_discovered_native_plugins("));
}
