use crate::asset::{
    AlphaMode, AssetReference, AssetUri, ImportedAsset, MaterialAsset, MeshVertex, ModelAsset,
    ModelPrimitiveAsset, ShaderAsset, ShaderDependencyAsset, ShaderEntryPointAsset,
    ShaderSourceLanguage, TextureAsset, TexturePayload, VirtualGeometryAsset,
};
use crate::core::framework::render::{
    RenderImageColorSpace, RenderImageFallbackKind, RenderImageUsage, RenderMaterialFallbackReason,
    RenderMaterialValidationError, RenderMeshKind, RenderMeshTopology,
    RenderShaderBindGroupLayoutDescriptor, RenderShaderBindingDescriptor,
    RenderShaderBindingResourceType, RenderShaderPipelineLayoutDescriptor, RenderShaderStage,
};
use crate::core::math::{Vec2, Vec3};
use crate::core::resource::ResourceKind;

#[test]
fn render_product_assets_texture_metadata_exposes_image_contract() {
    let texture = TextureAsset {
        uri: locator("res://textures/albedo.ktx2"),
        width: 128,
        height: 64,
        rgba: Vec::new(),
        payload: TexturePayload::Container {
            format: "bc7_rgba_unorm_srgb".to_string(),
            bytes: vec![1, 2, 3, 4],
            mip_count: 7,
            array_layers: 2,
        },
    };

    let descriptor = texture.render_image_descriptor();

    assert_eq!(descriptor.width, 128);
    assert_eq!(descriptor.height, 64);
    assert_eq!(descriptor.format, "bc7_rgba_unorm_srgb");
    assert_eq!(descriptor.color_space, RenderImageColorSpace::Srgb);
    assert_eq!(
        descriptor.usage,
        vec![RenderImageUsage::Sampled, RenderImageUsage::CopyDst]
    );
    assert_eq!(descriptor.mip_count, 7);
    assert_eq!(descriptor.array_layer_count, 2);
    assert_eq!(descriptor.fallback, RenderImageFallbackKind::MissingImage);
}

#[test]
fn render_product_assets_model_metadata_exposes_mesh_bounds_and_vg_presence() {
    let model = ModelAsset {
        uri: locator("res://models/triangle.model.toml"),
        primitives: vec![ModelPrimitiveAsset {
            vertices: vec![
                MeshVertex::new(Vec3::ZERO, Vec3::Y, Vec2::ZERO),
                MeshVertex::new(Vec3::X, Vec3::Y, Vec2::X),
                MeshVertex::new(Vec3::Y, Vec3::Y, Vec2::Y),
            ],
            indices: vec![0, 1, 2],
            virtual_geometry: Some(VirtualGeometryAsset::default()),
        }],
    };

    let descriptors = model.render_mesh_descriptors();
    let descriptor = &descriptors[0];

    assert_eq!(descriptor.topology, RenderMeshTopology::TriangleList);
    assert_eq!(descriptor.primitive_kind, RenderMeshKind::Planar2d);
    assert!(descriptor.suitable_for_2d);
    assert!(descriptor.suitable_for_3d);
    assert_eq!(descriptor.vertex_count, 3);
    assert_eq!(descriptor.index_count, 3);
    assert_eq!(descriptor.primitive_count, 1);
    assert!(descriptor.has_virtual_geometry_payload);
    assert_eq!(descriptor.bounds.min, [0.0, 0.0, 0.0]);
    assert_eq!(descriptor.bounds.max, [1.0, 1.0, 0.0]);
}

#[test]
fn render_product_assets_shader_selects_runtime_wgsl_and_entry_contracts() {
    let shader = ShaderAsset {
        uri: locator("res://shaders/runtime.shader"),
        source_language: ShaderSourceLanguage::Glsl,
        source: "void main() {}".to_string(),
        wgsl_source: "@fragment fn fs_main() {}".to_string(),
        entry_points: vec![ShaderEntryPointAsset {
            name: "fs_main".to_string(),
            stage: "fragment".to_string(),
        }],
        dependencies: vec![ShaderDependencyAsset {
            kind: ResourceKind::Texture,
            reference: asset_reference("res://textures/blue-noise.png"),
        }],
        pipeline_layout: RenderShaderPipelineLayoutDescriptor {
            bind_groups: vec![RenderShaderBindGroupLayoutDescriptor {
                group: 0,
                label: Some("material".to_string()),
                bindings: vec![
                    RenderShaderBindingDescriptor {
                        binding: 0,
                        label: Some("material_uniforms".to_string()),
                        resource_type: RenderShaderBindingResourceType::UniformBuffer,
                        visibility: vec![RenderShaderStage::Fragment],
                    },
                    RenderShaderBindingDescriptor {
                        binding: 1,
                        label: Some("base_color_texture".to_string()),
                        resource_type: RenderShaderBindingResourceType::Texture,
                        visibility: vec![RenderShaderStage::Fragment],
                    },
                ],
            }],
            push_constant_ranges: vec!["draw_index:0..4".to_string()],
        },
        validation_diagnostics: Vec::new(),
    };
    let fallback_wgsl = ShaderAsset {
        uri: locator("res://shaders/source.wgsl"),
        source_language: ShaderSourceLanguage::Wgsl,
        source: "@vertex fn vs_main() -> @builtin(position) vec4f { return vec4f(); }".to_string(),
        wgsl_source: "".to_string(),
        entry_points: Vec::new(),
        dependencies: Vec::new(),
        pipeline_layout: Default::default(),
        validation_diagnostics: Vec::new(),
    };
    let non_wgsl_without_runtime = ShaderAsset {
        uri: locator("res://shaders/source.glsl"),
        source_language: ShaderSourceLanguage::Glsl,
        source: "void main() {}".to_string(),
        wgsl_source: "".to_string(),
        entry_points: Vec::new(),
        dependencies: Vec::new(),
        pipeline_layout: Default::default(),
        validation_diagnostics: Vec::new(),
    };

    assert_eq!(
        shader.runtime_wgsl_source(),
        Some("@fragment fn fs_main() {}")
    );
    assert_eq!(
        fallback_wgsl.runtime_wgsl_source(),
        Some(fallback_wgsl.source.as_str())
    );
    assert_eq!(non_wgsl_without_runtime.runtime_wgsl_source(), None);
    assert_eq!(
        shader.entry_point_descriptors()[0].stage,
        RenderShaderStage::Fragment
    );
    assert_eq!(
        shader.variant_keys()[0].entry_point.as_deref(),
        Some("fs_main")
    );
    assert_eq!(shader.dependencies().len(), 1);
    assert_eq!(shader.dependencies()[0].kind, ResourceKind::Texture);
    assert_eq!(
        shader.dependencies()[0].reference.locator,
        locator("res://textures/blue-noise.png")
    );
    let layout = shader.pipeline_layout_descriptor();
    assert_eq!(layout.bind_groups.len(), 1);
    assert_eq!(layout.bind_groups[0].bindings.len(), 2);
    assert_eq!(
        layout.bind_groups[0].bindings[1].resource_type,
        RenderShaderBindingResourceType::Texture
    );
    assert_eq!(
        layout.push_constant_ranges,
        vec!["draw_index:0..4".to_string()]
    );
}

#[test]
fn render_product_assets_material_dependencies_validation_and_readiness_are_structured() {
    let material = MaterialAsset {
        name: Some("Mask".to_string()),
        shader: asset_reference("res://shaders/pbr.wgsl"),
        base_color: [1.0, 0.5, 0.25, 1.0],
        base_color_texture: Some(asset_reference("res://textures/base.png")),
        normal_texture: Some(asset_reference("res://textures/normal.png")),
        metallic: 0.2,
        roughness: 0.8,
        metallic_roughness_texture: Some(asset_reference("res://textures/mr.png")),
        occlusion_texture: Some(asset_reference("res://textures/occlusion.png")),
        emissive: [0.1, 0.2, 0.3],
        emissive_texture: Some(asset_reference("res://textures/emissive.png")),
        alpha_mode: AlphaMode::Mask { cutoff: 0.4 },
        double_sided: true,
    };

    let dependencies = material.dependency_set();
    let standard = material.standard_material_descriptor();
    let color = material.color_material_descriptor();
    let readiness = material.readiness_report();
    let direct_references = ImportedAsset::Material(material.clone()).direct_references();

    assert_eq!(
        dependencies.shader.locator,
        locator("res://shaders/pbr.wgsl")
    );
    assert_eq!(dependencies.textures.len(), 5);
    assert_eq!(direct_references.len(), 6);
    assert!(readiness.is_ready());
    assert!(!readiness.uses_fallback());
    assert!(readiness.validation_errors.is_empty());
    assert_eq!(standard.dependencies, dependencies);
    assert_eq!(
        standard.base_color_texture.as_ref().unwrap().locator,
        locator("res://textures/base.png")
    );
    assert!(!standard.unlit);
    assert!(standard.double_sided);
    assert_eq!(
        color.texture.as_ref().unwrap().locator,
        locator("res://textures/base.png")
    );
    assert!(color.unlit);
}

#[test]
fn render_product_assets_material_readiness_reports_unresolved_dependencies_and_fallbacks() {
    let material = MaterialAsset {
        name: Some("MissingRefs".to_string()),
        shader: asset_reference("res://shaders/missing.wgsl"),
        base_color: [1.0, 1.0, 1.0, 1.0],
        base_color_texture: Some(asset_reference("res://textures/missing-base.png")),
        normal_texture: Some(asset_reference("res://textures/normal.png")),
        metallic: 0.0,
        roughness: 1.0,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
    };

    let report = material.readiness_report_with_resolution(
        |shader| shader.locator == locator("res://shaders/pbr.wgsl"),
        |texture| texture.locator == locator("res://textures/normal.png"),
    );

    assert!(!report.is_ready());
    assert!(report.uses_fallback());
    assert_eq!(report.validation_errors.len(), 2);
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::UnresolvedShaderReference { .. }
    )));
    assert!(report.validation_errors.iter().any(|error| matches!(
        error,
        RenderMaterialValidationError::UnresolvedTextureReference { slot, .. }
            if slot == "base_color_texture"
    )));
    assert_eq!(report.fallback_usages.len(), 2);
    assert!(report.fallback_usages.iter().any(|usage| matches!(
        &usage.reason,
        RenderMaterialFallbackReason::Shader { reference }
            if reference.locator == locator("res://shaders/missing.wgsl")
    )));
    assert!(report.fallback_usages.iter().any(|usage| matches!(
        &usage.reason,
        RenderMaterialFallbackReason::Texture { slot, reference }
            if slot == "base_color_texture"
                && reference.locator == locator("res://textures/missing-base.png")
    )));
}

#[test]
fn render_product_assets_material_rejects_invalid_alpha_mask_cutoff() {
    for cutoff in [f32::NAN, -0.01, 1.01] {
        let material = MaterialAsset {
            name: Some("InvalidMask".to_string()),
            shader: asset_reference("res://shaders/pbr.wgsl"),
            base_color: [1.0, 1.0, 1.0, 1.0],
            base_color_texture: None,
            normal_texture: None,
            metallic: 0.0,
            roughness: 1.0,
            metallic_roughness_texture: None,
            occlusion_texture: None,
            emissive: [0.0, 0.0, 0.0],
            emissive_texture: None,
            alpha_mode: AlphaMode::Mask { cutoff },
            double_sided: false,
        };

        let errors = material.readiness_report().validation_errors;
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            errors[0],
            RenderMaterialValidationError::InvalidMaskCutoff { .. }
        ));
    }
}

fn locator(uri: &str) -> AssetUri {
    AssetUri::parse(uri).unwrap()
}

fn asset_reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(locator(uri))
}
