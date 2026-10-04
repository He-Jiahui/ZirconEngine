use crate::asset::{ImportedAsset, ShaderSurfaceSourceContract};

use super::builtin_resources;

#[test]
fn builtin_pbr_shader_publishes_only_the_surface_material_contract() {
    let shader = builtin_resources()
        .into_iter()
        .find_map(|(locator, asset)| (locator == "builtin://shader/pbr.wgsl").then_some(asset))
        .and_then(|asset| match asset {
            ImportedAsset::Shader(shader) => Some(shader),
            _ => None,
        })
        .expect("builtin PBR shader");

    assert!(shader.entry_points.is_empty());
    assert_eq!(
        shader.surface_source_contract(),
        Ok(Some(ShaderSurfaceSourceContract::MaterialFunction))
    );
}
