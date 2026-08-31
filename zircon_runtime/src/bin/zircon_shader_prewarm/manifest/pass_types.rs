use zircon_runtime::asset::ZShaderDocumentV2;
use zircon_runtime::core::framework::render::{ShaderAssetKind, ShaderPassType};

const ASSET_SCAN_FULL_MATERIAL_PASSES: [ShaderPassType; 6] = [
    ShaderPassType::Forward,
    ShaderPassType::GBuffer,
    ShaderPassType::DepthPrepass,
    ShaderPassType::Shadow,
    ShaderPassType::Velocity,
    ShaderPassType::TaaReactiveMask,
];

pub(super) fn asset_scan_full_material_passes() -> Vec<ShaderPassType> {
    ASSET_SCAN_FULL_MATERIAL_PASSES.to_vec()
}

pub(super) fn asset_scan_pass_types_for_zshader(
    document: &ZShaderDocumentV2,
) -> Vec<ShaderPassType> {
    if document.kind() != ShaderAssetKind::Surface {
        return Vec::new();
    }

    let disabled_passes = document
        .disabled_passes()
        .iter()
        .filter_map(|pass| pass_type_from_token(pass))
        .collect::<Vec<_>>();
    let mut passes = asset_scan_full_material_passes();
    passes.retain(|pass| !disabled_passes.contains(pass));
    passes
}

fn pass_type_from_token(token: &str) -> Option<ShaderPassType> {
    let token = token.trim();
    ASSET_SCAN_FULL_MATERIAL_PASSES
        .iter()
        .copied()
        .find(|pass_type| token.eq_ignore_ascii_case(pass_type.token()))
}

#[cfg(test)]
#[path = "pass_types/borrowed_token_tests.rs"]
mod borrowed_token_tests;
