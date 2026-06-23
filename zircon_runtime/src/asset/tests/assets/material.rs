use std::collections::BTreeMap;

use crate::asset::{
    AlphaMode, AssetReference, AssetUri, AssetUuid, MaterialAsset,
    MaterialAssetManagementRecordSet, MaterialTextureSlotValue, ShaderAsset, ShaderEntryPointAsset,
    ShaderMaterialPropertyAsset, ShaderSourceLanguage, ShaderTextureSlotAsset,
};
use crate::core::framework::render::{
    RenderMaterialDiagnosticSource, RenderMaterialLightingModel, RenderMaterialTextureTransform,
    RenderMaterialValidationError, RenderQueueValue, RenderShaderDefinitionValue,
};
use crate::core::resource::ResourceId;

mod asset_serialization;
mod management_records;
mod override_validation;
mod owned_descriptor;
mod shader_readiness;

fn shader_contract() -> ShaderAsset {
    ShaderAsset {
        uri: AssetUri::parse("res://shaders/mismatch.zshader").unwrap(),
        source_language: ShaderSourceLanguage::Wgsl,
        source: "@fragment fn fs_main() -> @location(0) vec4f { return vec4f(1.0); }".to_string(),
        wgsl_source: String::new(),
        import_path: None,
        entry_points: Vec::new(),
        dependencies: Vec::new(),
        source_files: Vec::new(),
        imports: Vec::new(),
        shader_defs: Vec::new(),
        property_schema: vec![
            ShaderMaterialPropertyAsset {
                name: "base_color".to_string(),
                kind: "vec4".to_string(),
                required: true,
                default: None,
                editor: Default::default(),
            },
            ShaderMaterialPropertyAsset {
                name: "emissive_power".to_string(),
                kind: "float".to_string(),
                required: true,
                default: None,
                editor: Default::default(),
            },
        ],
        texture_slots: vec![ShaderTextureSlotAsset {
            name: "base_color".to_string(),
            kind: "texture2d".to_string(),
            required: false,
            default: Some("white".to_string()),
            sampler: Some("linear_repeat".to_string()),
            group: Some("Surface".to_string()),
            label: Some("Base Color".to_string()),
            editor: Default::default(),
        }],
        editor: Default::default(),
        pipeline_layout: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}

fn asset_reference(label: &str, uri: &str) -> AssetReference {
    AssetReference::new(
        AssetUuid::from_stable_label(label),
        AssetUri::parse(uri).unwrap(),
    )
}
