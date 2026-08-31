use std::path::Path;

use zircon_runtime::asset::assets::{AlphaMode, MaterialAsset, ZMaterialDocument};
use zircon_runtime::asset::{AssetReference, ReferenceResolutionError};
use zircon_runtime_interface::resource::ResourceScheme;

use crate::material_fixture::ViewerMaterialFixture;

/// Validates the generated material through the same project-document path the
/// runtime uses before a persistent viewer project tree may be reused.
pub(crate) fn viewer_material_matches_fixture(
    material_path: &Path,
    material_fixture: ViewerMaterialFixture,
) -> bool {
    let Ok(source) = std::fs::read_to_string(material_path) else {
        return false;
    };
    let Ok(document) = ZMaterialDocument::from_project_toml_str(&source, |reference| {
        reference
            .builtin_locator()
            .cloned()
            .map(AssetReference::from_locator)
            .ok_or_else(|| ReferenceResolutionError::Registry {
                message: "viewer fixture validation requires a builtin shader reference"
                    .to_string(),
            })
    }) else {
        return false;
    };
    let material = MaterialAsset::from_zmaterial_document(document);
    let descriptor = material.standard_material_descriptor();
    material.name.as_deref() == Some(material_fixture.material_name())
        && material.shader.locator.scheme() == ResourceScheme::Builtin
        && material.shader.locator.path() == "shader/pbr.wgsl"
        && material.parent.is_none()
        && material.base_color == material_fixture.base_color()
        && material.metallic.to_bits() == material_fixture.metallic().to_bits()
        && material.roughness.to_bits() == material_fixture.roughness().to_bits()
        && material.emissive == [0.0, 0.0, 0.0]
        && material.alpha_mode == AlphaMode::Opaque
        && !material.double_sided
        && material.base_color_texture.is_none()
        && material.normal_texture.is_none()
        && material.metallic_roughness_texture.is_none()
        && material.occlusion_texture.is_none()
        && material.emissive_texture.is_none()
        && material.texture_slots.is_empty()
        && material.options.is_empty()
        && material.queue.is_none()
        && material.validation_diagnostics.is_empty()
        && material
            .property_values
            .get("lighting_model")
            .and_then(toml::Value::as_str)
            == Some("pbr")
        && material
            .property_values
            .get("receive_shadows")
            .and_then(toml::Value::as_bool)
            == Some(false)
        && matches_fixture_ior(&material, material_fixture)
        && descriptor.receive_shadows == false
        && matches_fixture_pbr_features(&material, material_fixture)
}

fn matches_fixture_ior(material: &MaterialAsset, material_fixture: ViewerMaterialFixture) -> bool {
    match material_fixture.dielectric_ior() {
        Some(expected) => {
            material
                .property_values
                .get("ior")
                .and_then(toml::Value::as_float)
                == Some(expected)
        }
        None => !material.property_values.contains_key("ior"),
    }
}

fn matches_fixture_pbr_features(
    material: &MaterialAsset,
    material_fixture: ViewerMaterialFixture,
) -> bool {
    let features = material.advanced_pbr_features();
    match material_fixture {
        ViewerMaterialFixture::MetalMirror => {
            !features.uses_dielectric_f0_override() && !features.requires_forward_path()
        }
        ViewerMaterialFixture::DielectricIor => {
            features.ior.to_bits() == 2.0_f32.to_bits()
                && (features.dielectric_f0() - (1.0 / 9.0)).abs() <= f32::EPSILON
                && features.uses_dielectric_f0_override()
                && features.requires_forward_path()
        }
    }
}
