use serde::{Deserialize, Serialize};

use crate::asset::AssetReference;
use crate::core::framework::render::{
    ColorMaterialDescriptor, RenderMaterialDependencySet, RenderMaterialFallbackPolicy,
    RenderMaterialFallbackReason, RenderMaterialFallbackUsage, RenderMaterialReadinessReport,
    RenderMaterialValidationError, StandardMaterialDescriptor,
};

use super::{dependency_set, validate_alpha_mode, AlphaMode};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MaterialAsset {
    pub name: Option<String>,
    pub shader: AssetReference,
    pub base_color: [f32; 4],
    pub base_color_texture: Option<AssetReference>,
    pub normal_texture: Option<AssetReference>,
    pub metallic: f32,
    pub roughness: f32,
    pub metallic_roughness_texture: Option<AssetReference>,
    pub occlusion_texture: Option<AssetReference>,
    pub emissive: [f32; 3],
    pub emissive_texture: Option<AssetReference>,
    pub alpha_mode: AlphaMode,
    pub double_sided: bool,
}

impl MaterialAsset {
    pub fn from_toml_str(document: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(document)
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    pub fn dependency_set(&self) -> RenderMaterialDependencySet {
        dependency_set::material_dependency_set(self)
    }

    pub fn direct_references(&self) -> Vec<AssetReference> {
        dependency_set::direct_references(self)
    }

    pub fn validation_errors(&self) -> Vec<RenderMaterialValidationError> {
        validate_alpha_mode(&self.alpha_mode)
    }

    pub fn readiness_report(&self) -> RenderMaterialReadinessReport {
        self.readiness_report_with_resolution(|_| true, |_| true)
    }

    pub fn readiness_report_with_resolution(
        &self,
        shader_resolves: impl Fn(&AssetReference) -> bool,
        texture_resolves: impl Fn(&AssetReference) -> bool,
    ) -> RenderMaterialReadinessReport {
        let fallback_policy = RenderMaterialFallbackPolicy::DefaultMaterial;
        let mut validation_errors = self.validation_errors();
        let mut fallback_usages = Vec::new();

        if !shader_resolves(&self.shader) {
            validation_errors.push(RenderMaterialValidationError::UnresolvedShaderReference {
                reference: self.shader.clone(),
            });
            fallback_usages.push(RenderMaterialFallbackUsage {
                reason: RenderMaterialFallbackReason::Shader {
                    reference: self.shader.clone(),
                },
                fallback_policy,
            });
        }

        for (slot, texture) in self.texture_slots() {
            if !texture_resolves(texture) {
                validation_errors.push(RenderMaterialValidationError::UnresolvedTextureReference {
                    slot: slot.to_string(),
                    reference: texture.clone(),
                });
                fallback_usages.push(RenderMaterialFallbackUsage {
                    reason: RenderMaterialFallbackReason::Texture {
                        slot: slot.to_string(),
                        reference: texture.clone(),
                    },
                    fallback_policy,
                });
            }
        }

        RenderMaterialReadinessReport {
            material_name: self.name.clone(),
            dependencies: self.dependency_set(),
            fallback_policy,
            validation_errors,
            fallback_usages,
        }
    }

    pub fn standard_material_descriptor(&self) -> StandardMaterialDescriptor {
        StandardMaterialDescriptor {
            name: self.name.clone(),
            dependencies: self.dependency_set(),
            base_color: self.base_color,
            base_color_texture: self.base_color_texture.clone(),
            normal_texture: self.normal_texture.clone(),
            metallic: self.metallic,
            roughness: self.roughness,
            metallic_roughness_texture: self.metallic_roughness_texture.clone(),
            occlusion_texture: self.occlusion_texture.clone(),
            emissive: self.emissive,
            emissive_texture: self.emissive_texture.clone(),
            alpha_mode: (&self.alpha_mode).into(),
            unlit: false,
            double_sided: self.double_sided,
            fallback_policy: RenderMaterialFallbackPolicy::DefaultMaterial,
        }
    }

    pub fn color_material_descriptor(&self) -> ColorMaterialDescriptor {
        ColorMaterialDescriptor {
            name: self.name.clone(),
            dependencies: self.dependency_set(),
            color: self.base_color,
            texture: self.base_color_texture.clone(),
            alpha_mode: (&self.alpha_mode).into(),
            unlit: true,
            double_sided: self.double_sided,
            fallback_policy: RenderMaterialFallbackPolicy::DefaultMaterial,
        }
    }

    fn texture_slots(&self) -> Vec<(&'static str, &AssetReference)> {
        [
            ("base_color_texture", self.base_color_texture.as_ref()),
            ("normal_texture", self.normal_texture.as_ref()),
            (
                "metallic_roughness_texture",
                self.metallic_roughness_texture.as_ref(),
            ),
            ("occlusion_texture", self.occlusion_texture.as_ref()),
            ("emissive_texture", self.emissive_texture.as_ref()),
        ]
        .into_iter()
        .filter_map(|(slot, texture)| texture.map(|texture| (slot, texture)))
        .collect::<Vec<_>>()
    }
}
