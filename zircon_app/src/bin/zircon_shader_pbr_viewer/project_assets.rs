use std::error::Error;
use std::fs;
use std::path::PathBuf;

use zircon_runtime::asset::assets::{
    AlphaMode, MaterialAsset, SceneAsset, SceneCameraAsset, SceneDirectionalLightAsset,
    SceneEntityAsset, SceneMeshInstanceAsset, SceneMobilityAsset, TransformAsset,
};
use zircon_runtime::asset::project::ProjectPaths;
use zircon_runtime::asset::{
    AssetReference, AssetUri, MeshVertex, ModelAsset, ModelPrimitiveAsset,
};
use zircon_runtime::core::framework::render::{ProjectionMode, DEFAULT_RENDER_LAYER_MASK};
use zircon_runtime::core::math::{Transform, Vec2, Vec3};

use crate::camera::{CAMERA_FOV_Y_RADIANS, DEFAULT_CAMERA_RADIUS, SPHERE_CENTER, SPHERE_SCALE};

const SPHERE_RINGS: usize = 96;
const SPHERE_SEGMENTS: usize = 192;

pub(crate) fn write_viewer_project_assets(paths: &ProjectPaths) -> Result<(), Box<dyn Error>> {
    write_uv_sphere_model(
        paths
            .assets_root()
            .join("models")
            .join("single_pbr_sphere.model.toml"),
        "res://models/single_pbr_sphere.model.toml",
        SPHERE_RINGS,
        SPHERE_SEGMENTS,
    )?;
    write_perfect_mirror_material(
        paths
            .assets_root()
            .join("materials")
            .join("single_metal_sphere.zmaterial"),
    )?;
    write_single_pbr_sphere_scene(
        paths
            .assets_root()
            .join("scenes")
            .join("single_pbr_sphere.scene.toml"),
    )?;
    Ok(())
}

fn write_uv_sphere_model(
    path: PathBuf,
    model_uri: &str,
    rings: usize,
    segments: usize,
) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let rings = rings.max(3);
    let segments = segments.max(6);
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for ring in 0..=rings {
        let theta = std::f32::consts::PI * ring as f32 / rings as f32;
        let y = theta.cos();
        let radius = theta.sin();
        for segment in 0..=segments {
            let phi = std::f32::consts::TAU * segment as f32 / segments as f32;
            let x = radius * phi.cos();
            let z = radius * phi.sin();
            vertices.push(
                MeshVertex::new(
                    Vec3::new(x, y, z),
                    Vec3::new(x, y, z),
                    Vec2::new(segment as f32 / segments as f32, ring as f32 / rings as f32),
                )
                .with_tangent([-phi.sin(), 0.0, phi.cos(), 1.0]),
            );
        }
    }
    for ring in 0..rings {
        for segment in 0..segments {
            let a = (ring * (segments + 1) + segment) as u32;
            let b = a + 1;
            let c = a + (segments + 1) as u32;
            let d = c + 1;
            // Winding matches outward radial normals so the mirror shader samples the visible shell.
            indices.extend_from_slice(&[a, b, c, b, d, c]);
        }
    }

    let model = ModelAsset {
        uri: AssetUri::parse(model_uri)?,
        primitives: vec![ModelPrimitiveAsset {
            vertices,
            indices,
            mesh: None,
            virtual_geometry: None,
        }],
    };
    fs::write(path, model.to_toml_string()?)?;
    Ok(())
}

fn write_perfect_mirror_material(path: PathBuf) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut material = MaterialAsset {
        name: Some("Interactive Perfect Mirror Sphere".to_string()),
        shader: asset_reference("builtin://shader/pbr.wgsl")?,
        parent: None,
        options: Default::default(),
        queue: None,
        base_color: [1.0, 1.0, 1.0, 1.0],
        base_color_texture: None,
        normal_texture: None,
        metallic: 1.0,
        roughness: 0.0,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    };
    material.property_values.insert(
        "lighting_model".to_string(),
        toml::Value::String("pbr".to_string()),
    );
    material
        .property_values
        .insert("receive_shadows".to_string(), toml::Value::Boolean(false));
    fs::write(path, material.to_toml_string()?)?;
    Ok(())
}

fn write_single_pbr_sphere_scene(path: PathBuf) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let sphere_model = asset_reference("res://models/single_pbr_sphere.model.toml")?;
    let sphere_material = asset_reference("res://materials/single_metal_sphere.zmaterial")?;
    let entities = vec![
        camera_entity(1, "Camera"),
        SceneEntityAsset {
            entity: 2,
            name: "PBR Sphere".to_string(),
            parent: None,
            transform: TransformAsset {
                translation: SPHERE_CENTER.to_array(),
                rotation: [0.0, 0.0, 0.0, 1.0],
                scale: SPHERE_SCALE,
            },
            active: true,
            render_layer_mask: DEFAULT_RENDER_LAYER_MASK,
            mobility: SceneMobilityAsset::Dynamic,
            camera: None,
            mesh: Some(SceneMeshInstanceAsset {
                model: sphere_model,
                mesh: None,
                material: sphere_material,
                render_queue: 0,
                material_queue: 0,
                order_in_layer: 0,
                depth_bias: 0.0,
                morph_weights: Vec::new(),
                primitives: Vec::new(),
                lods: Vec::new(),
            }),
            ambient_light: None,
            directional_light: None,
            point_light: None,
            rect_light: None,
            spot_light: None,
            post_process_volume: None,
            rigid_body: None,
            collider: None,
            joint: None,
            animation_skeleton: None,
            animation_player: None,
            animation_sequence_player: None,
            animation_graph_player: None,
            animation_state_machine_player: None,
            terrain: None,
            tilemap: None,
            prefab_instance: None,
            script_bindings: Vec::new(),
        },
        zero_intensity_key_light_entity(3, "Zero Intensity Key Light"),
    ];

    fs::write(path, SceneAsset { entities }.to_toml_string()?)?;
    Ok(())
}

fn camera_entity(entity: u64, name: &str) -> SceneEntityAsset {
    let transform = Transform::looking_at(
        Vec3::new(0.0, 0.0, DEFAULT_CAMERA_RADIUS),
        SPHERE_CENTER,
        Vec3::Y,
    );
    SceneEntityAsset {
        entity,
        name: name.to_string(),
        parent: None,
        transform: TransformAsset {
            translation: transform.translation.to_array(),
            rotation: transform.rotation.to_array(),
            scale: [1.0, 1.0, 1.0],
        },
        active: true,
        render_layer_mask: DEFAULT_RENDER_LAYER_MASK,
        mobility: SceneMobilityAsset::Dynamic,
        camera: Some(SceneCameraAsset {
            projection_mode: ProjectionMode::Perspective,
            fov_y_radians: CAMERA_FOV_Y_RADIANS,
            ortho_size: 3.4,
            z_near: 0.1,
            z_far: 100.0,
            post_process_settings: None,
            ..SceneCameraAsset::default()
        }),
        mesh: None,
        ambient_light: None,
        directional_light: None,
        point_light: None,
        rect_light: None,
        spot_light: None,
        post_process_volume: None,
        rigid_body: None,
        collider: None,
        joint: None,
        animation_skeleton: None,
        animation_player: None,
        animation_sequence_player: None,
        animation_graph_player: None,
        animation_state_machine_player: None,
        terrain: None,
        tilemap: None,
        prefab_instance: None,
        script_bindings: Vec::new(),
    }
}

fn zero_intensity_key_light_entity(entity: u64, name: &str) -> SceneEntityAsset {
    SceneEntityAsset {
        entity,
        name: name.to_string(),
        parent: None,
        transform: TransformAsset {
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        },
        active: true,
        render_layer_mask: DEFAULT_RENDER_LAYER_MASK,
        mobility: SceneMobilityAsset::Dynamic,
        camera: None,
        mesh: None,
        ambient_light: None,
        directional_light: Some(SceneDirectionalLightAsset {
            direction: [-0.35, -0.55, -0.76],
            color: [1.0, 0.96, 0.88],
            intensity: 0.0,
        }),
        point_light: None,
        rect_light: None,
        spot_light: None,
        post_process_volume: None,
        rigid_body: None,
        collider: None,
        joint: None,
        animation_skeleton: None,
        animation_player: None,
        animation_sequence_player: None,
        animation_graph_player: None,
        animation_state_machine_player: None,
        terrain: None,
        tilemap: None,
        prefab_instance: None,
        script_bindings: Vec::new(),
    }
}

fn asset_reference(uri: &str) -> Result<AssetReference, Box<dyn Error>> {
    Ok(AssetReference::from_locator(AssetUri::parse(uri)?))
}
