use std::sync::Arc;

use crate::asset::{
    AlphaMode, AssetReference, AssetUri, MaterialAsset, MaterialTextureSlotValue,
    ProjectAssetManager, ShaderAsset, ShaderEntryPointAsset, ShaderSourceLanguage,
    ShaderTextureSlotAsset, TextureAsset, TextureAssetDescriptor,
};
use crate::core::framework::render::{
    RenderMaterialDiagnosticSource, RenderMaterialFallbackReason, RenderMaterialPropertyValue,
    RenderMaterialTextureSlotFallbackReason, RenderMaterialValidationError,
    RenderShaderBindGroupLayoutDescriptor, RenderShaderBindingDescriptor,
    RenderShaderBindingResourceType, RenderShaderDefinitionValue,
    RenderShaderPipelineLayoutDescriptor, RenderShaderStage,
};
use crate::core::resource::{
    MaterialMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
};
use crate::graphics::backend::RenderBackend;

use super::resources::ResourceStreamer;

mod material_runtime;
mod readiness_diagnostics;
mod texture_slot_diagnostics;

fn missing_refs_material() -> MaterialAsset {
    material_with_refs(
        "res://shaders/missing-streamer.wgsl",
        Some("res://textures/missing-streamer.png"),
    )
}

fn material_with_refs(shader_uri: &str, base_color_texture_uri: Option<&str>) -> MaterialAsset {
    MaterialAsset {
        name: Some("StreamerMissingRefs".to_string()),
        shader: asset_reference(shader_uri),
        base_color: [1.0, 1.0, 1.0, 1.0],
        base_color_texture: base_color_texture_uri.map(asset_reference),
        normal_texture: None,
        metallic: 0.0,
        roughness: 1.0,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}

fn pbr_material_with_all_texture_slots() -> MaterialAsset {
    MaterialAsset {
        name: Some("PbrKey".to_string()),
        shader: asset_reference("builtin://shader/pbr.wgsl"),
        base_color: [0.25, 0.5, 0.75, 0.8],
        base_color_texture: Some(asset_reference("res://textures/base.png")),
        normal_texture: Some(asset_reference("res://textures/normal.png")),
        metallic: 0.35,
        roughness: 0.65,
        metallic_roughness_texture: Some(asset_reference("res://textures/mr.png")),
        occlusion_texture: Some(asset_reference("res://textures/occlusion.png")),
        emissive: [0.1, 0.2, 0.3],
        emissive_texture: Some(asset_reference("res://textures/emissive.png")),
        alpha_mode: AlphaMode::Mask { cutoff: 0.42 },
        double_sided: true,
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}

fn rgba_texture(uri: &str) -> TextureAsset {
    TextureAsset::new_rgba8(locator(uri), 1, 1, vec![255, 255, 255, 255])
}

fn container_texture(uri: &str) -> TextureAsset {
    TextureAsset::new_container(
        locator(uri),
        1,
        1,
        "bc7_rgba_unorm_srgb",
        vec![1, 2, 3, 4],
        1,
        1,
    )
}

fn wgsl_shader(uri: &str) -> ShaderAsset {
    ShaderAsset {
        uri: locator(uri),
        source_language: ShaderSourceLanguage::Wgsl,
        source: "@fragment fn fs_main() -> @location(0) vec4f { return vec4f(1.0); }".to_string(),
        wgsl_source: "".to_string(),
        import_path: None,
        entry_points: Vec::new(),
        dependencies: Vec::new(),
        source_files: Vec::new(),
        imports: Vec::new(),
        shader_defs: Vec::new(),
        property_schema: Vec::new(),
        texture_slots: Vec::new(),
        editor: Default::default(),
        pipeline_layout: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}

fn shader_with_texture_slot(uri: &str, slot: &str) -> ShaderAsset {
    let mut shader = wgsl_shader(uri);
    shader.texture_slots = vec![ShaderTextureSlotAsset {
        name: slot.to_string(),
        kind: "texture2d".to_string(),
        required: false,
        default: Some("white".to_string()),
        sampler: Some("linear_repeat".to_string()),
        group: None,
        label: None,
        editor: Default::default(),
    }];
    shader
}

fn shader_with_property_schema(uri: &str) -> ShaderAsset {
    let mut shader = wgsl_shader(uri);
    shader.property_schema = vec![
        shader_property("custom_gain", "float", None),
        shader_property("use_rim", "bool", None),
        shader_property(
            "rim_color",
            "vec4",
            Some(toml::Value::Array(vec![
                toml::Value::Float(0.25),
                toml::Value::Float(0.5),
                toml::Value::Float(0.75),
                toml::Value::Float(1.0),
            ])),
        ),
    ];
    shader
}

fn shader_with_required_material_contract(uri: &str) -> ShaderAsset {
    let mut shader = wgsl_shader(uri);
    shader.property_schema = vec![
        shader_property("required_gain", "float", None),
        shader_property("roughness_bias", "float", None),
    ];
    shader.texture_slots = vec![ShaderTextureSlotAsset {
        name: "required_mask".to_string(),
        kind: "texture2d".to_string(),
        required: true,
        default: Some("white".to_string()),
        sampler: Some("linear_repeat".to_string()),
        group: None,
        label: None,
        editor: Default::default(),
    }];
    shader
}

fn shader_property(
    name: &str,
    kind: &str,
    default: Option<toml::Value>,
) -> crate::asset::ShaderMaterialPropertyAsset {
    crate::asset::ShaderMaterialPropertyAsset {
        name: name.to_string(),
        kind: kind.to_string(),
        required: default.is_none(),
        default,
        editor: Default::default(),
    }
}

fn glsl_without_runtime_wgsl(uri: &str) -> ShaderAsset {
    ShaderAsset {
        uri: locator(uri),
        source_language: ShaderSourceLanguage::Glsl,
        source: "void main() {}".to_string(),
        wgsl_source: "".to_string(),
        import_path: None,
        entry_points: Vec::new(),
        dependencies: Vec::new(),
        source_files: Vec::new(),
        imports: Vec::new(),
        shader_defs: Vec::new(),
        property_schema: Vec::new(),
        texture_slots: Vec::new(),
        editor: Default::default(),
        pipeline_layout: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}

fn texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-render-product-streamer-test-texture-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

fn locator(uri: &str) -> AssetUri {
    AssetUri::parse(uri).unwrap()
}

fn asset_reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(locator(uri))
}
