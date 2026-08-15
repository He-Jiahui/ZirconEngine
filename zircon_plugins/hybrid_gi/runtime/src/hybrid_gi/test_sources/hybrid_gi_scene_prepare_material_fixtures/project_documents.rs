use std::fs;
use std::path::PathBuf;

use zircon_runtime::asset::{
    AlphaMode, AssetReference, AssetUri, MaterialAsset, MeshVertex, ModelAsset,
    ModelPrimitiveAsset, ReferenceResolutionError, SceneAsset, SceneCameraAsset, SceneEntityAsset,
    SceneMeshInstanceAsset, SceneMobilityAsset, TransformAsset,
};
use zircon_runtime::core::math::{Vec2, Vec3};
use zircon_runtime_interface::project::{AssetRef, PersistedAssetReference, RelPath};
use zircon_runtime_interface::resource::ResourceScheme;

pub(super) fn write_material_asset(
    asset_root: &RelPath,
    path: PathBuf,
    base_color: [f32; 4],
    emissive: [f32; 3],
) {
    write_material_asset_with_surface(asset_root, path, base_color, emissive, 0.0, 1.0);
}

pub(super) fn write_material_asset_with_surface(
    asset_root: &RelPath,
    path: PathBuf,
    base_color: [f32; 4],
    emissive: [f32; 3],
    metallic: f32,
    roughness: f32,
) {
    write_material_asset_with_textures(
        asset_root, path, base_color, emissive, metallic, roughness, None, None, None, None, None,
    );
}

#[allow(clippy::too_many_arguments)]
pub(super) fn write_material_asset_with_textures(
    asset_root: &RelPath,
    path: PathBuf,
    base_color: [f32; 4],
    emissive: [f32; 3],
    metallic: f32,
    roughness: f32,
    base_color_texture: Option<&str>,
    metallic_roughness_texture: Option<&str>,
    emissive_texture: Option<&str>,
    normal_texture: Option<&str>,
    occlusion_texture: Option<&str>,
) {
    write_material_asset_with_capture_options(
        asset_root,
        path,
        base_color,
        emissive,
        metallic,
        roughness,
        base_color_texture,
        metallic_roughness_texture,
        emissive_texture,
        normal_texture,
        occlusion_texture,
        AlphaMode::Opaque,
        false,
    );
}

#[allow(clippy::too_many_arguments)]
pub(super) fn write_material_asset_with_capture_options(
    asset_root: &RelPath,
    path: PathBuf,
    base_color: [f32; 4],
    emissive: [f32; 3],
    metallic: f32,
    roughness: f32,
    base_color_texture: Option<&str>,
    metallic_roughness_texture: Option<&str>,
    emissive_texture: Option<&str>,
    normal_texture: Option<&str>,
    occlusion_texture: Option<&str>,
    alpha_mode: AlphaMode,
    double_sided: bool,
) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let material = MaterialAsset {
        name: Some("HybridGiScenePrepare".to_string()),
        shader: asset_reference("builtin://shader/pbr.wgsl"),
        parent: None,
        options: Default::default(),
        queue: None,
        base_color,
        base_color_texture: base_color_texture.map(asset_reference),
        normal_texture: normal_texture.map(asset_reference),
        metallic,
        roughness,
        metallic_roughness_texture: metallic_roughness_texture.map(asset_reference),
        occlusion_texture: occlusion_texture.map(asset_reference),
        emissive,
        emissive_texture: emissive_texture.map(asset_reference),
        alpha_mode,
        double_sided,
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    };
    fs::write(
        path,
        material
            .to_project_toml_string(|reference| persist_fixture_reference(asset_root, reference))
            .unwrap(),
    )
    .unwrap();
}

pub(super) fn write_solid_ppm(path: PathBuf, rgba: [u8; 4]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(
        path,
        format!(
            "P3\n2 2\n255\n{} {} {}\n{} {} {}\n{} {} {}\n{} {} {}\n",
            rgba[0],
            rgba[1],
            rgba[2],
            rgba[0],
            rgba[1],
            rgba[2],
            rgba[0],
            rgba[1],
            rgba[2],
            rgba[0],
            rgba[1],
            rgba[2]
        ),
    )
    .unwrap();
}

pub(super) fn write_triangle_model(asset_root: &RelPath, path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let model = ModelAsset {
        uri: AssetUri::parse("res://models/triangle.model.toml").unwrap(),
        primitives: vec![ModelPrimitiveAsset {
            vertices: vec![
                MeshVertex::new(Vec3::ZERO, Vec3::Z, Vec2::ZERO),
                MeshVertex::new(Vec3::X, Vec3::Z, Vec2::X),
                MeshVertex::new(Vec3::Y, Vec3::Z, Vec2::Y),
            ],
            indices: vec![0, 1, 2],
            mesh: None,
            mesh_sdf: None,
            virtual_geometry: None,
        }],
    };
    fs::write(
        path,
        model
            .to_project_toml_string(|reference| persist_fixture_reference(asset_root, reference))
            .unwrap(),
    )
    .unwrap();
}

pub(super) fn write_scene_asset(asset_root: &RelPath, path: PathBuf, material_uri: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let scene = SceneAsset {
        entities: vec![
            SceneEntityAsset {
                entity: 1,
                name: "Camera".to_string(),
                parent: None,
                transform: TransformAsset {
                    translation: [0.0, 2.0, 5.0],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [1.0, 1.0, 1.0],
                },
                active: true,
                render_layer_mask: 0x0000_0001,
                mobility: SceneMobilityAsset::Dynamic,
                camera: Some(SceneCameraAsset {
                    fov_y_radians: 1.0471976,
                    z_near: 0.1,
                    z_far: 200.0,
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
            },
            SceneEntityAsset {
                entity: 2,
                name: "Triangle".to_string(),
                parent: None,
                transform: TransformAsset {
                    translation: [0.0, 0.0, 0.0],
                    rotation: [0.0, 0.0, 0.0, 1.0],
                    scale: [1.0, 1.0, 1.0],
                },
                active: true,
                render_layer_mask: 0x0000_0001,
                mobility: SceneMobilityAsset::Dynamic,
                camera: None,
                mesh: Some(SceneMeshInstanceAsset {
                    model: asset_reference("res://models/triangle.model.toml"),
                    mesh: None,
                    material: asset_reference(material_uri),
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
        ],
    };
    fs::write(
        path,
        scene
            .to_project_toml_string(|reference| persist_fixture_reference(asset_root, reference))
            .unwrap(),
    )
    .unwrap();
}

fn asset_reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}

fn persist_fixture_reference(
    asset_root: &RelPath,
    reference: &AssetReference,
) -> Result<PersistedAssetReference, ReferenceResolutionError> {
    match reference.locator.scheme() {
        ResourceScheme::Builtin => {
            return Ok(PersistedAssetReference::builtin(reference.locator.clone()));
        }
        ResourceScheme::Res => {}
        _ => {
            return Err(ReferenceResolutionError::UnsupportedScheme {
                locator: reference.locator.clone(),
            });
        }
    }

    let path_hint = RelPath::parse(format!(
        "{}/{}",
        asset_root.as_str(),
        reference.locator.path()
    ))
    .map_err(|source| ReferenceResolutionError::Path {
        path: reference.locator.to_string(),
        source,
    })?;
    let reference = AssetRef::try_new(
        reference.uuid,
        path_hint,
        reference.locator.label().map(str::to_owned),
    )
    .map_err(|source| ReferenceResolutionError::AssetRef { source })?;
    Ok(PersistedAssetReference::project(reference))
}
