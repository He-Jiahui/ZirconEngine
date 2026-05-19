use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::asset::{AssetReference, ShaderAsset};
use crate::core::framework::render::{
    ColorMaterialDescriptor, RenderMaterialDependencySet, RenderMaterialDiagnosticSource,
    RenderMaterialFallbackPolicy, RenderMaterialFallbackReason, RenderMaterialFallbackUsage,
    RenderMaterialReadinessReport, RenderMaterialValidationError, StandardMaterialDescriptor,
};

use super::{
    dependency_set, validate_alpha_mode, validate_shader_contract, AlphaMode,
    MaterialTextureSlotValue, ZMaterialDocument,
};

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
    #[serde(default)]
    pub property_values: BTreeMap<String, toml::Value>,
    #[serde(default)]
    pub texture_slots: BTreeMap<String, MaterialTextureSlotValue>,
    #[serde(default)]
    pub validation_diagnostics: Vec<String>,
}

impl MaterialAsset {
    pub fn from_toml_str(document: &str) -> Result<Self, toml::de::Error> {
        ZMaterialDocument::from_toml_str(document).map(Self::from_zmaterial_document)
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        self.to_zmaterial_document().to_toml_string()
    }

    pub fn from_zmaterial_document(document: ZMaterialDocument) -> Self {
        let base_color =
            override_vec4(&document.overrides, "base_color").unwrap_or([1.0, 1.0, 1.0, 1.0]);
        let metallic = override_f32(&document.overrides, "metallic").unwrap_or(0.0);
        let roughness = override_f32(&document.overrides, "roughness").unwrap_or(1.0);
        let emissive = override_vec3(&document.overrides, "emissive").unwrap_or([0.0, 0.0, 0.0]);
        let alpha_mode = document
            .overrides
            .get("alpha_mode")
            .and_then(|value| value.clone().try_into().ok())
            .unwrap_or(AlphaMode::Opaque);
        let double_sided = override_bool(&document.overrides, "double_sided").unwrap_or(false);
        let base_color_texture = texture_slot_reference(&document.textures, "base_color")
            .or_else(|| texture_slot_reference(&document.textures, "base_color_texture"));
        let normal_texture = texture_slot_reference(&document.textures, "normal")
            .or_else(|| texture_slot_reference(&document.textures, "normal_texture"));
        let metallic_roughness_texture =
            texture_slot_reference(&document.textures, "metallic_roughness").or_else(|| {
                texture_slot_reference(&document.textures, "metallic_roughness_texture")
            });
        let occlusion_texture = texture_slot_reference(&document.textures, "occlusion")
            .or_else(|| texture_slot_reference(&document.textures, "occlusion_texture"));
        let emissive_texture = texture_slot_reference(&document.textures, "emissive")
            .or_else(|| texture_slot_reference(&document.textures, "emissive_texture"));

        Self {
            name: document.name,
            shader: document.shader,
            base_color,
            base_color_texture,
            normal_texture,
            metallic,
            roughness,
            metallic_roughness_texture,
            occlusion_texture,
            emissive,
            emissive_texture,
            alpha_mode,
            double_sided,
            property_values: document.overrides,
            texture_slots: document.textures,
            validation_diagnostics: document.validation_diagnostics,
        }
    }

    pub fn to_zmaterial_document(&self) -> ZMaterialDocument {
        ZMaterialDocument {
            version: 1,
            name: self.name.clone(),
            shader: self.shader.clone(),
            overrides: self.property_overrides_with_legacy_defaults(),
            textures: self.texture_slots_with_legacy_defaults(),
            editor: toml::Table::new(),
            validation_diagnostics: self.validation_diagnostics.clone(),
        }
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

    pub fn shader_property_diagnostics(&self, shader: &ShaderAsset) -> Vec<String> {
        self.shader_contract_diagnostics(shader)
            .into_iter()
            .filter_map(|error| match error {
                RenderMaterialValidationError::UnknownPropertyOverride { name, .. } => Some(
                    format!("material property {name} is not declared by shader schema"),
                ),
                RenderMaterialValidationError::PropertyOverrideTypeMismatch {
                    name,
                    expected,
                    ..
                } => Some(format!(
                    "material property {name} expects {expected} but received override value"
                )),
                _ => None,
            })
            .collect()
    }

    pub fn shader_contract_diagnostics(
        &self,
        shader: &ShaderAsset,
    ) -> Vec<RenderMaterialValidationError> {
        validate_shader_contract(self, shader)
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

        for (slot, texture) in self.all_texture_slots() {
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

    pub fn readiness_report_with_shader_contract(
        &self,
        shader: &ShaderAsset,
        shader_resolves: impl Fn(&AssetReference) -> bool,
        texture_resolves: impl Fn(&AssetReference) -> bool,
    ) -> RenderMaterialReadinessReport {
        let mut report = self.readiness_report_with_resolution(shader_resolves, texture_resolves);
        for error in self.shader_contract_diagnostics(shader) {
            report.push_validation_error_once(error);
        }
        for diagnostic in &shader.validation_diagnostics {
            report.push_validation_error_once(RenderMaterialValidationError::MissingWgslCapture {
                source: RenderMaterialDiagnosticSource::WgslCapture,
                path: "shader.validation_diagnostics".to_string(),
                name: diagnostic.clone(),
            });
        }
        report
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

    pub fn property_overrides(&self) -> &BTreeMap<String, toml::Value> {
        &self.property_values
    }

    pub fn all_texture_slots(&self) -> Vec<(String, &AssetReference)> {
        let mut slots = self
            .legacy_texture_slots()
            .into_iter()
            .map(|(slot, texture)| (slot.to_string(), texture))
            .collect::<Vec<_>>();
        for (slot, texture) in &self.texture_slots {
            if let Some(reference) = texture.reference.as_ref() {
                if !slots.iter().any(|(existing, _)| existing == slot) {
                    slots.push((slot.clone(), reference));
                }
            }
        }
        slots
    }

    fn legacy_texture_slots(&self) -> Vec<(&'static str, &AssetReference)> {
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

    fn texture_slots_with_legacy_defaults(&self) -> BTreeMap<String, MaterialTextureSlotValue> {
        let mut slots = self.texture_slots.clone();
        // Canonical PBR slots own serialized references; shader fallback metadata can stay.
        sync_texture_slot(&mut slots, "base_color", self.base_color_texture.as_ref());
        sync_texture_slot(&mut slots, "normal", self.normal_texture.as_ref());
        sync_texture_slot(
            &mut slots,
            "metallic_roughness",
            self.metallic_roughness_texture.as_ref(),
        );
        sync_texture_slot(&mut slots, "occlusion", self.occlusion_texture.as_ref());
        sync_texture_slot(&mut slots, "emissive", self.emissive_texture.as_ref());
        slots
    }

    fn property_overrides_with_legacy_defaults(&self) -> BTreeMap<String, toml::Value> {
        let mut overrides = self.property_values.clone();
        // Runtime PBR fields must overwrite hydrated maps so source rewrites are real edits.
        sync_vec4_override(
            &mut overrides,
            "base_color",
            self.base_color,
            [1.0, 1.0, 1.0, 1.0],
        );
        sync_f32_override(&mut overrides, "metallic", self.metallic, 0.0);
        sync_f32_override(&mut overrides, "roughness", self.roughness, 1.0);
        sync_vec3_override(&mut overrides, "emissive", self.emissive, [0.0, 0.0, 0.0]);
        if self.alpha_mode != AlphaMode::Opaque {
            overrides.insert(
                "alpha_mode".to_string(),
                toml::Value::try_from(self.alpha_mode.clone()).unwrap(),
            );
        } else {
            overrides.remove("alpha_mode");
        }
        if self.double_sided {
            overrides.insert("double_sided".to_string(), toml::Value::Boolean(true));
        } else {
            overrides.remove("double_sided");
        }
        overrides
    }
}

fn texture_slot_reference(
    slots: &BTreeMap<String, MaterialTextureSlotValue>,
    slot: &str,
) -> Option<AssetReference> {
    slots.get(slot).and_then(|value| value.reference.clone())
}

fn override_f32(values: &BTreeMap<String, toml::Value>, key: &str) -> Option<f32> {
    values
        .get(key)
        .and_then(|value| {
            value
                .as_float()
                .or_else(|| value.as_integer().map(|value| value as f64))
        })
        .map(|value| value as f32)
}

fn override_bool(values: &BTreeMap<String, toml::Value>, key: &str) -> Option<bool> {
    values.get(key).and_then(toml::Value::as_bool)
}

fn override_vec4(values: &BTreeMap<String, toml::Value>, key: &str) -> Option<[f32; 4]> {
    let items = values.get(key)?.as_array()?;
    Some([
        toml_number_as_f32(items.first()?)?,
        toml_number_as_f32(items.get(1)?)?,
        toml_number_as_f32(items.get(2)?)?,
        toml_number_as_f32(items.get(3)?)?,
    ])
}

fn override_vec3(values: &BTreeMap<String, toml::Value>, key: &str) -> Option<[f32; 3]> {
    let items = values.get(key)?.as_array()?;
    Some([
        toml_number_as_f32(items.first()?)?,
        toml_number_as_f32(items.get(1)?)?,
        toml_number_as_f32(items.get(2)?)?,
    ])
}

fn toml_number_as_f32(value: &toml::Value) -> Option<f32> {
    value
        .as_float()
        .or_else(|| value.as_integer().map(|value| value as f64))
        .map(|value| value as f32)
}

fn sync_texture_slot(
    slots: &mut BTreeMap<String, MaterialTextureSlotValue>,
    slot: &str,
    texture: Option<&AssetReference>,
) {
    match texture {
        Some(texture) => {
            let fallback = slots.get(slot).and_then(|value| value.fallback.clone());
            let mut value = MaterialTextureSlotValue::new(texture.clone());
            value.fallback = fallback;
            slots.insert(slot.to_string(), value);
        }
        None => {
            let should_remove = if let Some(value) = slots.get_mut(slot) {
                value.reference = None;
                value.fallback.is_none()
            } else {
                false
            };
            if should_remove {
                slots.remove(slot);
            }
        }
    }
}

fn sync_f32_override(
    values: &mut BTreeMap<String, toml::Value>,
    key: &str,
    value: f32,
    default: f32,
) {
    if (value - default).abs() > f32::EPSILON {
        values.insert(key.to_string(), toml::Value::Float(value as f64));
    } else {
        values.remove(key);
    }
}

fn sync_vec4_override(
    values: &mut BTreeMap<String, toml::Value>,
    key: &str,
    value: [f32; 4],
    default: [f32; 4],
) {
    if value != default {
        values.insert(key.to_string(), toml_array(value));
    } else {
        values.remove(key);
    }
}

fn sync_vec3_override(
    values: &mut BTreeMap<String, toml::Value>,
    key: &str,
    value: [f32; 3],
    default: [f32; 3],
) {
    if value != default {
        values.insert(key.to_string(), toml_array(value));
    } else {
        values.remove(key);
    }
}

fn toml_array<const N: usize>(value: [f32; N]) -> toml::Value {
    toml::Value::Array(
        value
            .into_iter()
            .map(|value| toml::Value::Float(value as f64))
            .collect(),
    )
}
