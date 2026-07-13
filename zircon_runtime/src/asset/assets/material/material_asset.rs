use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::asset::{AssetReference, ShaderAsset};
use crate::core::framework::render::{
    ColorMaterialDescriptor, RenderMaterialAlphaMode, RenderMaterialDependencySet,
    RenderMaterialFallbackPolicy, RenderMaterialFallbackReason, RenderMaterialFallbackUsage,
    RenderMaterialLightingModel, RenderMaterialReadinessReport, RenderMaterialTextureTransform,
    RenderMaterialValidationError, RenderQueueValue, ShaderQueueDescriptor, ShaderQueueSegment,
    StandardMaterialDescriptor,
};
use crate::core::resource::ResourceId;

mod management;
mod readiness;
mod subsurface;
mod value_sync;

pub use self::management::{
    MaterialAssetManagementRecord, MaterialAssetManagementRecordSet,
    MaterialAssetManagementRecordSetSummary, MaterialAssetOverview,
};
use self::readiness::{material_readiness_diagnostics, push_shader_readiness_validation_errors};
use self::value_sync::{
    override_bool, override_f32, override_vec3, override_vec4, sync_f32_override,
    sync_texture_slot, sync_vec3_override, sync_vec4_override, texture_slot_reference,
};
use super::{
    dependency_set, is_standard_texture_slot_alias, material_control,
    shader_property_values_for_shader, validate_alpha_mode, validate_render_queue_alpha_mode,
    validate_shader_contract, AlphaMode, MaterialTextureSlotValue, ZMaterialDocument,
    ZMaterialQueueOverride,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MaterialAsset {
    pub name: Option<String>,
    pub shader: AssetReference,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<AssetReference>,
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
    pub options: BTreeMap<String, toml::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queue: Option<ZMaterialQueueOverride>,
    #[serde(default)]
    pub validation_diagnostics: Vec<String>,
}

impl MaterialAsset {
    #[cfg(test)]
    pub fn from_toml_str(document: &str) -> Result<Self, toml::de::Error> {
        ZMaterialDocument::from_toml_str(document).map(Self::from_zmaterial_document)
    }

    #[cfg(test)]
    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        self.to_zmaterial_document().to_toml_string()
    }

    pub fn to_project_toml_string(
        &self,
        resolver: impl FnMut(
            &AssetReference,
        ) -> Result<
            zircon_runtime_interface::project::PersistedAssetReference,
            crate::asset::ReferenceResolutionError,
        >,
    ) -> Result<String, crate::asset::assets::ProjectDocumentError> {
        self.to_zmaterial_document()
            .to_project_toml_string(resolver)
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
            parent: document.parent,
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
            options: document.options,
            queue: document.queue,
            validation_diagnostics: document.validation_diagnostics,
        }
    }

    pub fn to_zmaterial_document(&self) -> ZMaterialDocument {
        ZMaterialDocument {
            version: 2,
            name: self.name.clone(),
            shader: self.shader.clone(),
            parent: self.parent.clone(),
            options: self.options.clone(),
            overrides: self.property_overrides_with_schema_v1_defaults(),
            textures: self.texture_slots_with_schema_v1_defaults(),
            queue: self.queue,
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
        let mut errors = validate_alpha_mode(&self.alpha_mode);
        errors.extend(validate_render_queue_alpha_mode(
            &self.alpha_mode,
            self.render_queue_from_property(),
        ));
        errors.extend(super::validation::validate_material_queue_override(
            self.queue,
        ));
        errors.extend(material_control::validation_errors(&self.property_values));
        errors
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
        let unresolved_shader_imports = shader
            .dependencies
            .iter()
            .filter(|dependency| !shader_resolves(&dependency.reference))
            .map(|dependency| dependency.reference.clone())
            .collect::<Vec<_>>();
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
        for reference in unresolved_shader_imports {
            report.push_validation_error_once(
                RenderMaterialValidationError::UnresolvedShaderReference {
                    reference: reference.clone(),
                },
            );
            report.push_fallback_usage_once(RenderMaterialFallbackUsage {
                reason: RenderMaterialFallbackReason::Shader { reference },
                fallback_policy: report.fallback_policy,
            });
        }
        push_shader_readiness_validation_errors(&mut report, shader);
        report
    }

    pub fn standard_material_descriptor(&self) -> StandardMaterialDescriptor {
        let lighting_model = self.lighting_model();
        let unlit = lighting_model.is_unlit();
        StandardMaterialDescriptor {
            name: self.name.clone(),
            dependencies: self.dependency_set(),
            base_color: self.base_color,
            base_color_texture: self.base_color_texture.clone(),
            base_color_texture_transform: self
                .texture_slot_transform(&["base_color", "base_color_texture"]),
            base_color_texture_uv_channel: self
                .texture_slot_uv_channel(&["base_color", "base_color_texture"]),
            normal_texture: self.normal_texture.clone(),
            normal_texture_transform: self.texture_slot_transform(&["normal", "normal_texture"]),
            normal_texture_uv_channel: self.texture_slot_uv_channel(&["normal", "normal_texture"]),
            metallic: self.metallic,
            roughness: self.roughness,
            metallic_roughness_texture: self.metallic_roughness_texture.clone(),
            metallic_roughness_texture_transform: self
                .texture_slot_transform(&["metallic_roughness", "metallic_roughness_texture"]),
            metallic_roughness_texture_uv_channel: self
                .texture_slot_uv_channel(&["metallic_roughness", "metallic_roughness_texture"]),
            occlusion_texture: self.occlusion_texture.clone(),
            occlusion_texture_transform: self
                .texture_slot_transform(&["occlusion", "occlusion_texture"]),
            occlusion_texture_uv_channel: self
                .texture_slot_uv_channel(&["occlusion", "occlusion_texture"]),
            emissive: self.emissive,
            emissive_texture: self.emissive_texture.clone(),
            emissive_texture_transform: self
                .texture_slot_transform(&["emissive", "emissive_texture"]),
            emissive_texture_uv_channel: self
                .texture_slot_uv_channel(&["emissive", "emissive_texture"]),
            alpha_mode: (&self.alpha_mode).into(),
            lighting_model,
            unlit,
            double_sided: self.double_sided,
            cast_shadows: self.cast_shadows(),
            receive_shadows: self.receive_shadows(),
            render_queue: self.render_queue(),
            render_queue_value: self.render_queue_value(),
            material_queue: self.material_queue(),
            depth_bias: self.depth_bias(),
            taa_reactive_mask_strength: self.taa_reactive_mask_strength(),
            subsurface_profile_index: self.subsurface_profile_index(),
            fallback_policy: RenderMaterialFallbackPolicy::DefaultMaterial,
        }
    }

    pub fn standard_material_descriptor_for_shader(
        &self,
        shader: &ShaderAsset,
    ) -> StandardMaterialDescriptor {
        let mut descriptor = self.standard_material_descriptor();
        if let Some(lighting_model) = shader
            .shading_model
            .as_deref()
            .and_then(|token| token.parse::<RenderMaterialLightingModel>().ok())
        {
            descriptor.unlit = lighting_model.is_unlit();
            descriptor.lighting_model = lighting_model;
        }
        if let Some(queue) = shader.queue {
            descriptor.render_queue_value = Some(shader_queue_value(queue));
            descriptor.material_queue =
                i32::from(self.queue.map(|queue| queue.offset).unwrap_or(0));
        }
        if let Some(slot) = self.shader_texture_slot(
            shader,
            &["base_color", "base_color_texture", "albedo", "diffuse"],
        ) {
            if let Some(reference) = slot.reference.clone() {
                descriptor.base_color_texture = Some(reference);
            }
            descriptor.base_color_texture_transform = slot.texture_transform();
            descriptor.base_color_texture_uv_channel = slot.texture_uv_channel();
        }
        if let Some(slot) = self.shader_texture_slot(shader, &["normal", "normal_texture"]) {
            if let Some(reference) = slot.reference.clone() {
                descriptor.normal_texture = Some(reference);
            }
            descriptor.normal_texture_transform = slot.texture_transform();
            descriptor.normal_texture_uv_channel = slot.texture_uv_channel();
        }
        if let Some(slot) = self.shader_texture_slot(
            shader,
            &["metallic_roughness", "metallic_roughness_texture"],
        ) {
            if let Some(reference) = slot.reference.clone() {
                descriptor.metallic_roughness_texture = Some(reference);
            }
            descriptor.metallic_roughness_texture_transform = slot.texture_transform();
            descriptor.metallic_roughness_texture_uv_channel = slot.texture_uv_channel();
        }
        if let Some(slot) = self.shader_texture_slot(shader, &["occlusion", "occlusion_texture"]) {
            if let Some(reference) = slot.reference.clone() {
                descriptor.occlusion_texture = Some(reference);
            }
            descriptor.occlusion_texture_transform = slot.texture_transform();
            descriptor.occlusion_texture_uv_channel = slot.texture_uv_channel();
        }
        if let Some(slot) = self.shader_texture_slot(shader, &["emissive", "emissive_texture"]) {
            if let Some(reference) = slot.reference.clone() {
                descriptor.emissive_texture = Some(reference);
            }
            descriptor.emissive_texture_transform = slot.texture_transform();
            descriptor.emissive_texture_uv_channel = slot.texture_uv_channel();
        }
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

    pub fn material_option_values(&self) -> &BTreeMap<String, toml::Value> {
        &self.options
    }

    pub fn material_option_bits_for_shader(&self, shader: &ShaderAsset) -> u32 {
        shader
            .material_option_table
            .bits_for_values(self.material_option_values())
    }

    pub fn inherit_parent_values_from(&mut self, parent: &MaterialAsset) {
        let child = self.clone();
        *self = parent.clone();
        self.name = child.name.or_else(|| parent.name.clone());
        self.shader = child.shader;
        self.parent = child.parent;
        self.property_values.extend(child.property_values);
        self.texture_slots.extend(child.texture_slots);
        self.options.extend(child.options);
        self.queue = child.queue.or(parent.queue);
        self.validation_diagnostics
            .extend(child.validation_diagnostics);
        self.refresh_standard_projection();
    }

    pub fn shader_property_overrides(&self) -> impl Iterator<Item = (&String, &toml::Value)> {
        self.property_values
            .iter()
            .filter(|(name, _)| !material_control::is_material_owned_property(name))
    }

    pub fn shader_property_override(&self, name: &str) -> Option<&toml::Value> {
        (!material_control::is_material_owned_property(name))
            .then(|| self.property_values.get(name))
            .flatten()
    }

    pub fn lighting_model(&self) -> RenderMaterialLightingModel {
        self.lighting_model_from_property().unwrap_or_default()
    }

    pub fn cast_shadows(&self) -> bool {
        self.cast_shadows_from_property().unwrap_or(true)
    }

    pub fn receive_shadows(&self) -> bool {
        self.receive_shadows_from_property().unwrap_or(true)
    }

    pub fn render_queue(&self) -> i32 {
        self.render_queue_from_property().unwrap_or_default()
    }

    pub fn render_queue_value(&self) -> Option<RenderQueueValue> {
        let authored_queue = self.render_queue_from_property()?;
        let alpha_mode = RenderMaterialAlphaMode::from(&self.alpha_mode);
        Some(RenderQueueValue::from_authored_queue(
            &alpha_mode,
            authored_queue,
        ))
    }

    pub fn material_queue(&self) -> i32 {
        self.material_queue_from_property().unwrap_or_default()
    }

    pub fn depth_bias(&self) -> f32 {
        self.depth_bias_from_property().unwrap_or_default()
    }

    pub fn taa_reactive_mask_strength(&self) -> f32 {
        self.taa_reactive_mask_strength_from_property()
            .unwrap_or_default()
    }

    pub fn all_texture_slots(&self) -> Vec<(String, &AssetReference)> {
        let mut slots = self
            .schema_v1_pbr_texture_slots()
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

    fn schema_v1_pbr_texture_slots(&self) -> Vec<(&'static str, &AssetReference)> {
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

    fn texture_slots_with_schema_v1_defaults(&self) -> BTreeMap<String, MaterialTextureSlotValue> {
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

    fn shader_texture_slot(
        &self,
        shader: &ShaderAsset,
        aliases: &[&str],
    ) -> Option<&MaterialTextureSlotValue> {
        aliases
            .iter()
            .filter(|alias| shader.texture_slots.iter().any(|slot| slot.name == **alias))
            .find_map(|alias| self.texture_slots.get(*alias))
    }

    fn texture_slot_transform(&self, aliases: &[&str]) -> RenderMaterialTextureTransform {
        aliases
            .iter()
            .find_map(|alias| self.texture_slots.get(*alias))
            .map(MaterialTextureSlotValue::texture_transform)
            .unwrap_or_default()
    }

    fn texture_slot_uv_channel(&self, aliases: &[&str]) -> u32 {
        aliases
            .iter()
            .find_map(|alias| self.texture_slots.get(*alias))
            .map(MaterialTextureSlotValue::texture_uv_channel)
            .unwrap_or_default()
    }

    fn property_overrides_with_schema_v1_defaults(&self) -> BTreeMap<String, toml::Value> {
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
        material_control::sync_material_control_overrides(&mut overrides, &self.property_values);
        overrides
    }

    fn refresh_standard_projection(&mut self) {
        self.base_color =
            override_vec4(&self.property_values, "base_color").unwrap_or([1.0, 1.0, 1.0, 1.0]);
        self.metallic = override_f32(&self.property_values, "metallic").unwrap_or(0.0);
        self.roughness = override_f32(&self.property_values, "roughness").unwrap_or(1.0);
        self.emissive = override_vec3(&self.property_values, "emissive").unwrap_or([0.0, 0.0, 0.0]);
        self.alpha_mode = self
            .property_values
            .get("alpha_mode")
            .and_then(|value| value.clone().try_into().ok())
            .unwrap_or(AlphaMode::Opaque);
        self.double_sided = override_bool(&self.property_values, "double_sided").unwrap_or(false);
        self.base_color_texture = texture_slot_reference(&self.texture_slots, "base_color")
            .or_else(|| texture_slot_reference(&self.texture_slots, "base_color_texture"));
        self.normal_texture = texture_slot_reference(&self.texture_slots, "normal")
            .or_else(|| texture_slot_reference(&self.texture_slots, "normal_texture"));
        self.metallic_roughness_texture =
            texture_slot_reference(&self.texture_slots, "metallic_roughness").or_else(|| {
                texture_slot_reference(&self.texture_slots, "metallic_roughness_texture")
            });
        self.occlusion_texture = texture_slot_reference(&self.texture_slots, "occlusion")
            .or_else(|| texture_slot_reference(&self.texture_slots, "occlusion_texture"));
        self.emissive_texture = texture_slot_reference(&self.texture_slots, "emissive")
            .or_else(|| texture_slot_reference(&self.texture_slots, "emissive_texture"));
    }
}

fn shader_queue_value(queue: ShaderQueueDescriptor) -> RenderQueueValue {
    let base = match queue.segment {
        ShaderQueueSegment::Background => RenderQueueValue::BACKGROUND,
        ShaderQueueSegment::Opaque => RenderQueueValue::GEOMETRY,
        ShaderQueueSegment::AlphaTest => RenderQueueValue::ALPHA_TEST,
        ShaderQueueSegment::Transparent => RenderQueueValue::TRANSPARENT,
        ShaderQueueSegment::Overlay => RenderQueueValue::OVERLAY,
    };
    base.with_material_offset(queue.offset)
}

impl MaterialAsset {
    fn lighting_model_from_property(&self) -> Option<RenderMaterialLightingModel> {
        material_control::lighting_model(&self.property_values)
    }

    fn cast_shadows_from_property(&self) -> Option<bool> {
        material_control::cast_shadows(&self.property_values)
    }

    fn receive_shadows_from_property(&self) -> Option<bool> {
        material_control::receive_shadows(&self.property_values)
    }

    fn render_queue_from_property(&self) -> Option<i32> {
        material_control::render_queue(&self.property_values)
    }

    fn material_queue_from_property(&self) -> Option<i32> {
        material_control::material_queue(&self.property_values)
    }

    fn depth_bias_from_property(&self) -> Option<f32> {
        material_control::depth_bias(&self.property_values)
    }

    fn taa_reactive_mask_strength_from_property(&self) -> Option<f32> {
        material_control::taa_reactive_mask_strength(&self.property_values)
    }

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
