//! 默认 PBR 引用指向内建复合 shader 资产；材质缺省值和导入流程借它维持同一稳定 URI，而不是自行拼接子资产路径。

use std::sync::OnceLock;

use crate::asset::{AssetReference, AssetUri};

/// Project asset root for the built-in compound Standard-PBR shader package.
pub const DEFAULT_PBR_SHADER_URI: &str = "res://shaders/default_pbr";

/// Returns the canonical reference used when an imported material has no explicit shader.
///
/// The cached reference avoids reparsing the stable compound asset root for every imported
/// material while preserving the regular value-based `AssetReference` API at call sites.
pub fn default_pbr_shader_reference() -> AssetReference {
    static REFERENCE: OnceLock<AssetReference> = OnceLock::new();

    REFERENCE
        .get_or_init(|| {
            AssetReference::from_locator(
                AssetUri::parse(DEFAULT_PBR_SHADER_URI)
                    .expect("default PBR shader locator must be valid"),
            )
        })
        .clone()
}

#[cfg(test)]
#[path = "tests/default_pbr.rs"]
mod tests;
