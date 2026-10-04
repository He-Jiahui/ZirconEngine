/// Exact compiler identity encoded into shader source and disk-cache keys.
pub const SHADER_VARIANT_CACHE_NAGA_VERSION: &str = "naga-29.0.3";

/// Exact backend identity encoded into shader source and disk-cache keys.
pub const SHADER_VARIANT_CACHE_WGPU_VERSION: &str = "wgpu-29.0.3";

#[cfg(test)]
#[path = "tests/toolchain.rs"]
mod tests;
