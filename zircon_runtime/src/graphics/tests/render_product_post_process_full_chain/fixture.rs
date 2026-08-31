use std::collections::BTreeMap;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::{
    AlphaMode, AssetReference, AssetUri, MaterialAsset, RGBA8_UNORM_FORMAT, TextureAsset,
    TextureAssetDescriptor,
};
use crate::core::framework::render::{
    AntiAliasSettings, COLOR_LUT_SIZE_DEFAULT, DEFAULT_RENDER_LAYER_MASK, FallbackSkyboxKind,
    PreviewEnvironmentExtract, RenderBloomSettings, RenderBlurSettings,
    RenderChromaticAberrationSettings, RenderColorGradingSettings, RenderColorLookupSettings,
    RenderColorLookupTextureLayout, RenderDepthOfFieldSettings, RenderDitherSettings,
    RenderDynamicResolutionSettings, RenderExposureSettings, RenderFilmGrainSettings,
    RenderFogSettings, RenderFrameExtract, RenderImageColorSpace, RenderLayerSet,
    RenderMeshSnapshot, RenderMotionBlurSettings, RenderOverlayExtract,
    RenderParticlePreviousSpriteSnapshot, RenderParticleSpriteSnapshot, RenderPipelineHandle,
    RenderPostProcessEffectStackSettings, RenderQualityProfile, RenderSceneGeometryExtract,
    RenderSceneSnapshot, RenderScreenSpaceReflectionSettings, RenderTonemapOperator,
    RenderTonemapSettings, RenderVignetteSettings, RenderWorldSnapshotHandle,
    ViewportCameraSnapshot,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Transform, UVec2, Vec2, Vec3, Vec4};
use crate::core::resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
    TextureMarker,
};

pub(super) fn full_chain_product_profile(
    profile_name: &str,
    full_chain_enabled: bool,
) -> RenderQualityProfile {
    RenderQualityProfile::new(profile_name)
        .with_pipeline_asset(RenderPipelineHandle::new(1))
        .with_clustered_lighting(false)
        .with_screen_space_ambient_occlusion(false)
        .with_temporal_history(full_chain_enabled)
        .with_bloom(full_chain_enabled)
        .with_color_grading(full_chain_enabled)
        .with_reflection_probes(false)
        .with_baked_lighting(false)
        .with_particle_rendering(true)
        .with_anti_alias(full_chain_enabled)
}

pub(super) fn full_chain_product_extract(
    viewport_size: UVec2,
    receiver_material: ResourceId,
    caster_material: ResourceId,
    user_lut: ResourceHandle<TextureMarker>,
    full_chain_enabled: bool,
) -> RenderFrameExtract {
    let particles = full_chain_particle_sprites();
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(937),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot {
                    transform: Transform::looking_at(
                        Vec3::new(0.0, -3.4, 2.35),
                        Vec3::new(0.0, 0.0, 0.18),
                        Vec3::Y,
                    ),
                    ..ViewportCameraSnapshot::default()
                },
                meshes: vec![
                    full_chain_mesh(
                        937_100,
                        Transform {
                            scale: Vec3::new(3.4, 2.4, 0.04),
                            ..Transform::default()
                        },
                        receiver_material,
                    ),
                    full_chain_mesh(
                        937_101,
                        Transform {
                            translation: Vec3::new(-0.42, 0.08, 0.58),
                            scale: Vec3::new(0.38, 0.38, 0.72),
                            ..Transform::default()
                        },
                        caster_material,
                    ),
                    full_chain_mesh(
                        937_102,
                        Transform {
                            translation: Vec3::new(0.62, 0.22, 0.34),
                            scale: Vec3::new(0.28, 0.28, 0.42),
                            ..Transform::default()
                        },
                        caster_material,
                    ),
                ],
                directional_lights: vec![
                    crate::core::framework::render::RenderDirectionalLightSnapshot {
                        node_id: 937_200,
                        light_id: 937_200,
                        layer_mask: RenderLayerSet::from_scene_schema_v1_mask(
                            DEFAULT_RENDER_LAYER_MASK,
                        ),
                        direction: Vec3::new(0.45, 0.25, -1.0).normalize(),
                        color: Vec3::ONE,
                        intensity: 1.4,
                        mobility: crate::core::framework::scene::Mobility::Dynamic,
                        shadow: None,
                    },
                ],
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            environment: crate::core::framework::render::EnvironmentExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: true,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::new(0.01, 0.012, 0.018, 1.0),
            },
            virtual_geometry_debug: None,
        },
    )
    .with_viewport_size(viewport_size);
    extract.particles.emitters = vec![937];
    extract.particles.sprites = particles.clone();

    if full_chain_enabled {
        extract.particles.previous_sprites = particles
            .iter()
            .map(full_chain_previous_particle_sprite)
            .collect();
        extract.post_process.bloom = RenderBloomSettings {
            threshold: 0.2,
            intensity: 1.2,
            radius: 1.0,
        };
        extract.post_process.exposure = RenderExposureSettings::histogram();
        extract.post_process.color_grading = RenderColorGradingSettings {
            exposure: 1.15,
            contrast: 1.08,
            saturation: 0.9,
            gamma: 0.96,
            tint: Vec3::new(1.05, 0.82, 0.72),
        };
        extract.post_process.effect_stack = full_chain_effect_stack(user_lut);
        extract.view.anti_alias = AntiAliasSettings::auto();
        extract.view.camera.dynamic_resolution = RenderDynamicResolutionSettings::fixed_scale(0.5);
        extract.view.sync_selected_descriptor_camera_payload();
    } else {
        extract.view.anti_alias = AntiAliasSettings::off();
    }
    extract
}

fn full_chain_effect_stack(
    user_lut: ResourceHandle<TextureMarker>,
) -> RenderPostProcessEffectStackSettings {
    RenderPostProcessEffectStackSettings {
        tonemap: RenderTonemapSettings {
            operator: RenderTonemapOperator::Aces,
            exposure_bias: 0.25,
            white_point: 1.1,
        },
        color_lookup: RenderColorLookupSettings {
            texture: Some(user_lut),
            texture_layout: RenderColorLookupTextureLayout::Texture2dStrip {
                size: COLOR_LUT_SIZE_DEFAULT,
            },
            intensity: 0.8,
        },
        blur: RenderBlurSettings { radius: 6.0 },
        depth_of_field: RenderDepthOfFieldSettings {
            focus_distance: 28.0,
            focus_range: 0.05,
            aperture: 1.0,
            focal_length_mm: 120.0,
            max_blur_radius: 8.0,
            bokeh_blade_count: 7,
            bokeh_rotation_radians: 0.35,
        },
        motion_blur: RenderMotionBlurSettings {
            shutter_angle: 1.0,
            samples: 12,
        },
        screen_space_reflection: RenderScreenSpaceReflectionSettings {
            intensity: 4.0,
            thickness: 0.65,
            max_ray_distance: 80.0,
            max_steps: 64,
            temporal_blend_factor: 0.0,
            roughness_mip_bias: -0.5,
        },
        vignette: RenderVignetteSettings {
            intensity: 0.45,
            smoothness: 0.5,
            roundness: 1.0,
        },
        grain: RenderFilmGrainSettings {
            intensity: 0.2,
            response: 0.85,
        },
        dither: RenderDitherSettings {
            intensity: 0.08,
            scale: 1.0,
        },
        chromatic_aberration: RenderChromaticAberrationSettings {
            intensity: 0.12,
            sample_spread: 1.5,
        },
        fog: RenderFogSettings {
            density: 0.35,
            height_falloff: 0.25,
            color: Vec3::new(0.06, 0.1, 0.22),
        },
    }
}

fn full_chain_mesh(node_id: u64, transform: Transform, material: ResourceId) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id,
        stable_instance_key: node_id << 16,
        transform_revision: 0,
        transform,
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label("builtin://cube")),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(material),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Dynamic,
        static_state: Default::default(),
        common: crate::core::framework::render::RendererCommon {
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK),
            ..Default::default()
        },
    }
}

pub(super) fn full_chain_material(
    name: &str,
    base_color: [f32; 4],
    metallic: f32,
    roughness: f32,
    emissive: [f32; 3],
    cast_shadows: bool,
    receive_shadows: bool,
) -> MaterialAsset {
    let mut property_values = BTreeMap::new();
    property_values.insert(
        "cast_shadows".to_string(),
        toml::Value::Boolean(cast_shadows),
    );
    property_values.insert(
        "receive_shadows".to_string(),
        toml::Value::Boolean(receive_shadows),
    );

    MaterialAsset {
        name: Some(name.to_string()),
        shader: AssetReference::from_locator(AssetUri::parse("builtin://shader/pbr.wgsl").unwrap()),
        parent: None,
        options: Default::default(),
        queue: None,
        base_color,
        base_color_texture: None,
        normal_texture: None,
        metallic,
        roughness,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive,
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
        property_values,
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}

pub(super) fn register_full_chain_material(
    asset_manager: &ProjectAssetManager,
    locator: &str,
    material: MaterialAsset,
) -> ResourceId {
    let material_uri = AssetUri::parse(locator).unwrap();
    let material_id = ResourceId::from_locator(&material_uri);
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material,
        )
        .expect("full-chain product material insert");
    material_id
}

fn full_chain_particle_sprites() -> Vec<RenderParticleSpriteSnapshot> {
    [
        (
            1,
            Vec3::new(-0.55, -0.20, -2.35),
            Vec4::new(1.0, 0.46, 0.12, 0.94),
        ),
        (
            2,
            Vec3::new(0.10, 0.24, -2.55),
            Vec4::new(0.12, 0.82, 1.0, 0.9),
        ),
        (
            3,
            Vec3::new(0.62, -0.02, -2.75),
            Vec4::new(0.95, 0.92, 0.18, 0.92),
        ),
        (
            4,
            Vec3::new(-0.05, -0.52, -2.95),
            Vec4::new(0.88, 0.10, 1.0, 0.88),
        ),
    ]
    .into_iter()
    .map(
        |(stable_sprite_key, position, color)| RenderParticleSpriteSnapshot {
            entity: 937,
            stable_sprite_key,
            position,
            size: 0.42,
            aspect_ratio: 1.0,
            billboard_offset: Vec2::ZERO,
            rotation: stable_sprite_key as f32 * 0.12,
            sort_order: stable_sprite_key as i32,
            color,
            intensity: 1.0,
            depth_test: true,
            render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
            material: None,
            texture: None,
        },
    )
    .collect()
}

fn full_chain_previous_particle_sprite(
    sprite: &RenderParticleSpriteSnapshot,
) -> RenderParticlePreviousSpriteSnapshot {
    RenderParticlePreviousSpriteSnapshot {
        entity: sprite.entity,
        stable_sprite_key: sprite.stable_sprite_key,
        position: sprite.position - Vec3::new(0.82, 0.0, 0.0),
        size: sprite.size,
        aspect_ratio: sprite.aspect_ratio,
        billboard_offset: sprite.billboard_offset,
        rotation: sprite.rotation,
        billboard_basis: None,
    }
}

pub(super) fn insert_user_lut_texture(
    asset_manager: &ProjectAssetManager,
    uri: &str,
) -> ResourceHandle<TextureMarker> {
    let size = COLOR_LUT_SIZE_DEFAULT;
    let width = size * size;
    let height = size;
    let texture_uri = AssetUri::parse(uri).unwrap();
    let texture_id = ResourceId::from_locator(&texture_uri);
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(texture_id, ResourceKind::Texture, texture_uri.clone()),
            TextureAsset::new_rgba8(texture_uri, width, height, user_lut_strip_rgba8(size))
                .with_descriptor(user_lut_texture_descriptor()),
        )
        .expect("full-chain user LUT texture insert");
    ResourceHandle::<TextureMarker>::new(texture_id)
}

fn user_lut_texture_descriptor() -> TextureAssetDescriptor {
    let mut descriptor = TextureAssetDescriptor::rgba8_srgb();
    descriptor.format = RGBA8_UNORM_FORMAT.to_string();
    descriptor.color_space = RenderImageColorSpace::Linear;
    descriptor
}

fn user_lut_strip_rgba8(size: u32) -> Vec<u8> {
    let mut rgba = Vec::with_capacity((size * size * size * 4) as usize);
    for green in 0..size {
        for blue in 0..size {
            for red in 0..size {
                let source_color = [
                    lut_axis_value(red, size),
                    lut_axis_value(green, size),
                    lut_axis_value(blue, size),
                ];
                let expected = expected_user_lut_color(source_color);
                rgba.push(linear_channel_to_u8(expected[0]));
                rgba.push(linear_channel_to_u8(expected[1]));
                rgba.push(linear_channel_to_u8(expected[2]));
                rgba.push(255);
            }
        }
    }
    rgba
}

fn expected_user_lut_color(source_color: [f32; 3]) -> [f32; 3] {
    [
        1.0 - source_color[0],
        source_color[1] * 0.5,
        source_color[2],
    ]
}

fn lut_axis_value(index: u32, size: u32) -> f32 {
    if size <= 1 {
        0.0
    } else {
        index as f32 / (size - 1) as f32
    }
}

fn linear_channel_to_u8(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}
