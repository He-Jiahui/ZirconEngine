use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::asset::{AssetReference, ShaderAsset, ShaderRuntimeSourceKind};
use crate::core::framework::render::{
    ColorMaterialDescriptor, RenderMaterialDependencySet, RenderMaterialDiagnosticSource,
    RenderMaterialFallbackPolicy, RenderMaterialFallbackReason, RenderMaterialFallbackUsage,
    RenderMaterialReadinessDiagnostic, RenderMaterialReadinessReport,
    RenderMaterialValidationError, StandardMaterialDescriptor,
};
use crate::core::resource::ResourceId;

use super::{
    dependency_set, shader_property_values_for_shader, validate_alpha_mode,
    validate_shader_contract, AlphaMode, MaterialTextureSlotValue, ZMaterialDocument,
};

/// Asset-level material summary that does not require renderer preparation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialAssetOverview {
    pub name: Option<String>,
    pub shader: AssetReference,
    pub property_override_count: usize,
    pub texture_slot_count: usize,
    pub texture_reference_count: usize,
    pub fallback_texture_slot_count: usize,
    pub validation_error_count: usize,
    pub validation_diagnostic_count: usize,
    pub direct_reference_count: usize,
}

/// Stable list row for registered `.zmaterial` assets.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialAssetManagementRecord {
    pub material_id: ResourceId,
    pub overview: MaterialAssetOverview,
}

/// Cross-row totals for material assets before renderer readiness is considered.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialAssetManagementRecordSetSummary {
    pub material_count: usize,
    pub ready_count: usize,
    pub issue_material_count: usize,
    pub property_override_count: usize,
    pub texture_slot_count: usize,
    pub texture_reference_count: usize,
    pub fallback_texture_slot_count: usize,
    pub validation_error_count: usize,
    pub validation_diagnostic_count: usize,
    pub direct_reference_count: usize,
}

/// Sorted material asset rows plus aggregate authoring/dependency counts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialAssetManagementRecordSet {
    pub records: Vec<MaterialAssetManagementRecord>,
    pub summary: MaterialAssetManagementRecordSetSummary,
}

impl MaterialAssetManagementRecordSetSummary {
    pub fn from_records(records: &[MaterialAssetManagementRecord]) -> Self {
        let issue_material_count = records
            .iter()
            .filter(|record| {
                record.overview.validation_error_count + record.overview.validation_diagnostic_count
                    > 0
            })
            .count();
        Self {
            material_count: records.len(),
            ready_count: records.len() - issue_material_count,
            issue_material_count,
            property_override_count: records
                .iter()
                .map(|record| record.overview.property_override_count)
                .sum(),
            texture_slot_count: records
                .iter()
                .map(|record| record.overview.texture_slot_count)
                .sum(),
            texture_reference_count: records
                .iter()
                .map(|record| record.overview.texture_reference_count)
                .sum(),
            fallback_texture_slot_count: records
                .iter()
                .map(|record| record.overview.fallback_texture_slot_count)
                .sum(),
            validation_error_count: records
                .iter()
                .map(|record| record.overview.validation_error_count)
                .sum(),
            validation_diagnostic_count: records
                .iter()
                .map(|record| record.overview.validation_diagnostic_count)
                .sum(),
            direct_reference_count: records
                .iter()
                .map(|record| record.overview.direct_reference_count)
                .sum(),
        }
    }

    pub fn degraded_count(&self) -> usize {
        self.issue_material_count
    }

    pub fn issue_row_count(&self) -> usize {
        self.validation_error_count + self.validation_diagnostic_count
    }
}

impl MaterialAssetManagementRecordSet {
    pub fn from_records(mut records: Vec<MaterialAssetManagementRecord>) -> Self {
        records.sort_by_key(|record| record.material_id);
        let summary = MaterialAssetManagementRecordSetSummary::from_records(&records);
        Self { records, summary }
    }
}

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

    pub fn overview(&self) -> MaterialAssetOverview {
        MaterialAssetOverview {
            name: self.name.clone(),
            shader: self.shader.clone(),
            property_override_count: self.property_overrides().len(),
            texture_slot_count: self.texture_slots.len(),
            texture_reference_count: self.all_texture_slots().len(),
            fallback_texture_slot_count: self
                .texture_slots
                .values()
                .filter(|slot| slot.fallback.is_some())
                .count(),
            validation_error_count: self.validation_errors().len(),
            validation_diagnostic_count: self.validation_diagnostics.len(),
            direct_reference_count: self.direct_references().len(),
        }
    }

    pub fn management_record(&self, material_id: ResourceId) -> MaterialAssetManagementRecord {
        MaterialAssetManagementRecord {
            material_id,
            overview: self.overview(),
        }
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
                RenderMaterialValidationError::MissingRequiredProperty { name, .. } => Some(
                    format!("material property {name} is required by shader schema"),
                ),
                RenderMaterialValidationError::MissingRequiredTextureSlot { slot, .. } => Some(
                    format!("material texture slot {slot} requires a concrete texture reference"),
                ),
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

    pub fn shader_property_values_for_shader(
        &self,
        shader: &ShaderAsset,
    ) -> BTreeMap<String, crate::core::framework::render::RenderMaterialPropertyValue> {
        shader_property_values_for_shader(self, shader)
    }

    pub fn readiness_report(&self) -> RenderMaterialReadinessReport {
        self.readiness_report_with_resolution(|_| true, |_| true)
    }

    pub fn readiness_report_with_resolution(
        &self,
        shader_resolves: impl Fn(&AssetReference) -> bool,
        texture_resolves: impl Fn(&AssetReference) -> bool,
    ) -> RenderMaterialReadinessReport {
        self.readiness_report_from_texture_slots(
            self.dependency_set(),
            self.all_texture_slots()
                .into_iter()
                .map(|(slot, reference)| (slot, reference.clone()))
                .collect(),
            shader_resolves,
            texture_resolves,
        )
    }

    pub fn readiness_report_with_shader_contract(
        &self,
        shader: &ShaderAsset,
        shader_resolves: impl Fn(&AssetReference) -> bool,
        texture_resolves: impl Fn(&AssetReference) -> bool,
    ) -> RenderMaterialReadinessReport {
        let descriptor = self.standard_material_descriptor_for_shader(shader);
        let texture_slots = self.shader_aware_texture_slots_from_descriptor(&descriptor);
        let mut report = self.readiness_report_from_texture_slots(
            descriptor.dependencies,
            texture_slots,
            shader_resolves,
            texture_resolves,
        );
        for error in self.shader_contract_diagnostics(shader) {
            report.push_validation_error_once(error);
        }
        push_shader_readiness_validation_errors(&mut report, shader);
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

    pub fn standard_material_descriptor_for_shader(
        &self,
        shader: &ShaderAsset,
    ) -> StandardMaterialDescriptor {
        let mut descriptor = self.standard_material_descriptor();
        descriptor.base_color_texture = self
            .shader_texture_slot_reference(
                shader,
                &["base_color", "base_color_texture", "albedo", "diffuse"],
            )
            .or(descriptor.base_color_texture);
        descriptor.normal_texture = self
            .shader_texture_slot_reference(shader, &["normal", "normal_texture"])
            .or(descriptor.normal_texture);
        descriptor.metallic_roughness_texture = self
            .shader_texture_slot_reference(
                shader,
                &["metallic_roughness", "metallic_roughness_texture"],
            )
            .or(descriptor.metallic_roughness_texture);
        descriptor.occlusion_texture = self
            .shader_texture_slot_reference(shader, &["occlusion", "occlusion_texture"])
            .or(descriptor.occlusion_texture);
        descriptor.emissive_texture = self
            .shader_texture_slot_reference(shader, &["emissive", "emissive_texture"])
            .or(descriptor.emissive_texture);
        descriptor.dependencies = self.shader_aware_dependency_set_from_descriptor(&descriptor);
        descriptor
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

    fn shader_texture_slot_reference(
        &self,
        shader: &ShaderAsset,
        aliases: &[&str],
    ) -> Option<AssetReference> {
        aliases
            .iter()
            .filter(|alias| shader.texture_slots.iter().any(|slot| slot.name == **alias))
            .find_map(|alias| {
                self.texture_slots
                    .get(*alias)
                    .and_then(|value| value.reference.clone())
            })
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

impl MaterialAsset {
    fn readiness_report_from_texture_slots(
        &self,
        dependencies: RenderMaterialDependencySet,
        texture_slots: Vec<(String, AssetReference)>,
        shader_resolves: impl Fn(&AssetReference) -> bool,
        texture_resolves: impl Fn(&AssetReference) -> bool,
    ) -> RenderMaterialReadinessReport {
        let fallback_policy = RenderMaterialFallbackPolicy::DefaultMaterial;
        let mut validation_errors = self.validation_errors();
        let mut fallback_usages = Vec::new();

        if !shader_resolves(&dependencies.shader) {
            validation_errors.push(RenderMaterialValidationError::UnresolvedShaderReference {
                reference: dependencies.shader.clone(),
            });
            fallback_usages.push(RenderMaterialFallbackUsage {
                reason: RenderMaterialFallbackReason::Shader {
                    reference: dependencies.shader.clone(),
                },
                fallback_policy,
            });
        }

        for texture in &dependencies.textures {
            if !texture_resolves(texture) {
                let slot = texture_slots
                    .iter()
                    .find_map(|(slot, reference)| (reference == texture).then(|| slot.clone()))
                    .unwrap_or_else(|| "texture".to_string());
                validation_errors.push(RenderMaterialValidationError::UnresolvedTextureReference {
                    slot: slot.clone(),
                    reference: texture.clone(),
                });
                fallback_usages.push(RenderMaterialFallbackUsage {
                    reason: RenderMaterialFallbackReason::Texture {
                        slot,
                        reference: texture.clone(),
                    },
                    fallback_policy,
                });
            }
        }

        RenderMaterialReadinessReport {
            material_name: self.name.clone(),
            dependencies,
            fallback_policy,
            validation_errors,
            fallback_usages,
            property_value_summary: None,
            property_value_states: Vec::new(),
            uniform_summary: None,
            uniform_fields: Vec::new(),
            uniform_unsupported: Vec::new(),
            standard_texture_slot_summary: None,
            standard_texture_slot_states: Vec::new(),
            texture_slot_summary: None,
            non_standard_texture_slot_states: Vec::new(),
            diagnostics: material_readiness_diagnostics(self),
        }
    }

    fn shader_aware_dependency_set_from_descriptor(
        &self,
        descriptor: &StandardMaterialDescriptor,
    ) -> RenderMaterialDependencySet {
        let mut dependencies =
            RenderMaterialDependencySet::new(descriptor.dependencies.shader.clone());
        for (_slot, reference) in self.shader_aware_texture_slots_from_descriptor(descriptor) {
            dependencies.push_texture(reference);
        }
        dependencies
    }

    fn shader_aware_texture_slots_from_descriptor(
        &self,
        descriptor: &StandardMaterialDescriptor,
    ) -> Vec<(String, AssetReference)> {
        let mut slots = self.standard_texture_slots_from_descriptor(descriptor);
        for (slot, texture) in &self.texture_slots {
            if is_standard_texture_slot_alias(slot) {
                continue;
            }
            if let Some(reference) = texture.reference.clone() {
                slots.push((slot.clone(), reference));
            }
        }
        slots
    }

    fn standard_texture_slots_from_descriptor(
        &self,
        descriptor: &StandardMaterialDescriptor,
    ) -> Vec<(String, AssetReference)> {
        [
            ("base_color_texture", descriptor.base_color_texture.clone()),
            ("normal_texture", descriptor.normal_texture.clone()),
            (
                "metallic_roughness_texture",
                descriptor.metallic_roughness_texture.clone(),
            ),
            ("occlusion_texture", descriptor.occlusion_texture.clone()),
            ("emissive_texture", descriptor.emissive_texture.clone()),
        ]
        .into_iter()
        .filter_map(|(slot, reference)| reference.map(|reference| (slot.to_string(), reference)))
        .collect()
    }
}

fn is_standard_texture_slot_alias(slot: &str) -> bool {
    matches!(
        slot,
        "base_color"
            | "base_color_texture"
            | "albedo"
            | "diffuse"
            | "normal"
            | "normal_texture"
            | "metallic_roughness"
            | "metallic_roughness_texture"
            | "occlusion"
            | "occlusion_texture"
            | "emissive"
            | "emissive_texture"
    )
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

fn push_shader_readiness_validation_errors(
    report: &mut RenderMaterialReadinessReport,
    shader: &ShaderAsset,
) {
    let readiness = shader.readiness_report();
    if readiness.runtime_source.source_kind == ShaderRuntimeSourceKind::Unavailable {
        report
            .push_validation_error_once(RenderMaterialValidationError::MissingRuntimeShaderSource);
    }

    for entry in readiness.entry_points {
        if let Some(diagnostic) = entry.diagnostic {
            report.push_validation_error_once(
                RenderMaterialValidationError::ShaderReadinessDiagnostic {
                    source: RenderMaterialDiagnosticSource::ShaderReadiness,
                    path: format!("entry_points.{}", entry.name),
                    diagnostic,
                },
            );
        }
    }

    for definition in readiness.shader_defs {
        if let Some(diagnostic) = definition.diagnostic {
            let path_name = if definition.normalized_name.is_empty() {
                "<empty>".to_string()
            } else {
                definition.normalized_name
            };
            report.push_validation_error_once(
                RenderMaterialValidationError::ShaderReadinessDiagnostic {
                    source: RenderMaterialDiagnosticSource::ShaderReadiness,
                    path: format!("shader_defs.{path_name}"),
                    diagnostic,
                },
            );
        }
    }

    for diagnostic in readiness.validation_diagnostics {
        report.push_validation_error_once(RenderMaterialValidationError::MissingWgslCapture {
            source: RenderMaterialDiagnosticSource::WgslCapture,
            path: "shader.validation_diagnostics".to_string(),
            name: diagnostic,
        });
    }
}

fn material_readiness_diagnostics(
    material: &MaterialAsset,
) -> Vec<RenderMaterialReadinessDiagnostic> {
    material
        .validation_diagnostics
        .iter()
        .enumerate()
        .map(|(index, diagnostic)| RenderMaterialReadinessDiagnostic {
            source: RenderMaterialDiagnosticSource::MaterialAsset,
            path: format!("material.validation_diagnostics[{index}]"),
            diagnostic: diagnostic.clone(),
        })
        .collect()
}
