/// Exact compiler identity encoded into shader source and disk-cache keys.
pub const SHADER_VARIANT_CACHE_NAGA_VERSION: &str = "naga-29.0.3";

/// Exact backend identity encoded into shader source and disk-cache keys.
pub const SHADER_VARIANT_CACHE_WGPU_VERSION: &str = "wgpu-29.0.3";

#[cfg(test)]
mod tests {
    use super::{SHADER_VARIANT_CACHE_NAGA_VERSION, SHADER_VARIANT_CACHE_WGPU_VERSION};

    const WORKSPACE_MANIFEST: &str =
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../Cargo.toml"));
    const WORKSPACE_LOCK: &str =
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../Cargo.lock"));

    #[test]
    fn shader_cache_toolchain_identity_matches_exact_workspace_resolution() {
        let manifest = WORKSPACE_MANIFEST
            .parse::<toml::Value>()
            .expect("workspace Cargo.toml must parse");
        let lock = WORKSPACE_LOCK
            .parse::<toml::Value>()
            .expect("workspace Cargo.lock must parse");

        assert_toolchain_dependency(&manifest, &lock, "naga", SHADER_VARIANT_CACHE_NAGA_VERSION);
        assert_toolchain_dependency(&manifest, &lock, "wgpu", SHADER_VARIANT_CACHE_WGPU_VERSION);
    }

    fn assert_toolchain_dependency(
        manifest: &toml::Value,
        lock: &toml::Value,
        dependency_name: &str,
        cache_identity: &str,
    ) {
        let resolved_version = lock
            .get("package")
            .and_then(toml::Value::as_array)
            .and_then(|packages| {
                packages.iter().find_map(|package| {
                    (package.get("name").and_then(toml::Value::as_str) == Some(dependency_name))
                        .then(|| package.get("version").and_then(toml::Value::as_str))
                        .flatten()
                })
            })
            .expect("shader toolchain dependency must exist in Cargo.lock");
        let declared_version = manifest
            .get("workspace")
            .and_then(|workspace| workspace.get("dependencies"))
            .and_then(|dependencies| dependencies.get(dependency_name))
            .and_then(|dependency| {
                dependency
                    .as_str()
                    .or_else(|| dependency.get("version").and_then(toml::Value::as_str))
            })
            .expect("shader toolchain dependency must exist in workspace dependencies");

        assert_eq!(declared_version, format!("={resolved_version}"));
        assert_eq!(
            cache_identity,
            format!("{dependency_name}-{resolved_version}")
        );
    }
}
