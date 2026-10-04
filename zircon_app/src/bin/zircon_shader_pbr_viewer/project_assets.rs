use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use zircon_runtime::asset::assets::{
    AlphaMode, MaterialAsset, SceneAsset, SceneCameraAsset, SceneDirectionalLightAsset,
    SceneEntityAsset, SceneMeshInstanceAsset, SceneMobilityAsset, TransformAsset,
};
use zircon_runtime::asset::{
    project::AssetMetaDocument, AssetKind, AssetReference, AssetUri, AssetUuid, MeshVertex,
    ModelAsset, ModelPrimitiveAsset, ReferenceResolutionError,
};
use zircon_runtime::core::framework::render::{ProjectionMode, DEFAULT_RENDER_LAYER_MASK};
use zircon_runtime::core::math::{Transform, Vec2, Vec3};
use zircon_runtime_interface::project::{AssetRef, PersistedAssetReference, RelPath};
use zircon_runtime_interface::resource::ResourceScheme;

use crate::camera::{CAMERA_FOV_Y_RADIANS, DEFAULT_CAMERA_RADIUS, SPHERE_CENTER, SPHERE_SCALE};
use crate::material_fixture::ViewerMaterialFixture;
use crate::project_asset_fixture_validation::viewer_material_matches_fixture;

// The viewer verifies shading, not subpixel geometric detail. This keeps the generated
// temporary asset compact while retaining a smooth silhouette at the default viewport size.
const SPHERE_RINGS: usize = 64;
const SPHERE_SEGMENTS: usize = 128;
pub(crate) const VIEWER_PROJECT_ASSET_ROOT: &str = "viewer-assets-v4";
const VIEWER_MODEL_URI: &str = "res://models/single_pbr_sphere.model.toml";
const VIEWER_MATERIAL_URI: &str = "res://materials/single_metal_sphere.zmaterial";
const VIEWER_SCENE_URI: &str = "res://scenes/single_pbr_sphere.scene.toml";
const VIEWER_PROJECT_SOURCE_PATHS: [&str; 3] = [
    "models/single_pbr_sphere.model.toml",
    "materials/single_metal_sphere.zmaterial",
    "scenes/single_pbr_sphere.scene.toml",
];
const VIEWER_PROJECT_ASSET_PATHS: [&str; 6] = [
    "models/single_pbr_sphere.model.toml",
    "models/single_pbr_sphere.model.toml.zmeta",
    "materials/single_metal_sphere.zmaterial",
    "materials/single_metal_sphere.zmaterial.zmeta",
    "scenes/single_pbr_sphere.scene.toml",
    "scenes/single_pbr_sphere.scene.toml.zmeta",
];
static VIEWER_PROJECT_STAGING_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ViewerProjectAssetGenerationReport {
    mesh_generation_samples: u32,
    serialized_source_bytes: u64,
    filesystem_writes: u32,
}

impl ViewerProjectAssetGenerationReport {
    pub(crate) const fn reused() -> Self {
        Self {
            mesh_generation_samples: 0,
            serialized_source_bytes: 0,
            filesystem_writes: 0,
        }
    }

    fn generated(serialized_source_bytes: u64) -> Self {
        Self {
            mesh_generation_samples: 1,
            serialized_source_bytes,
            filesystem_writes: VIEWER_PROJECT_ASSET_PATHS.len() as u32,
        }
    }

    pub(crate) const fn mesh_generation_samples(self) -> u32 {
        self.mesh_generation_samples
    }

    pub(crate) const fn serialized_source_bytes(self) -> u64 {
        self.serialized_source_bytes
    }

    pub(crate) const fn filesystem_writes(self) -> u32 {
        self.filesystem_writes
    }
}

struct ViewerProjectAssetReferences {
    model: AssetReference,
    material: AssetReference,
    scene: AssetReference,
}

impl ViewerProjectAssetReferences {
    fn versioned(material_fixture: ViewerMaterialFixture) -> Result<Self, Box<dyn Error>> {
        let identity_prefix = material_fixture.project_asset_identity_prefix();
        Ok(Self {
            model: viewer_asset_reference(VIEWER_MODEL_URI, &format!("{identity_prefix}/model"))?,
            material: viewer_asset_reference(
                VIEWER_MATERIAL_URI,
                &format!("{identity_prefix}/material"),
            )?,
            scene: viewer_asset_reference(VIEWER_SCENE_URI, &format!("{identity_prefix}/scene"))?,
        })
    }
}

pub(crate) fn viewer_project_assets_are_ready_for_fixture(
    asset_root: &std::path::Path,
    material_fixture: ViewerMaterialFixture,
) -> bool {
    asset_root.is_dir()
        && VIEWER_PROJECT_ASSET_PATHS
            .iter()
            .all(|relative_path| asset_root.join(relative_path).is_file())
        && viewer_material_matches_fixture(
            &asset_root.join("materials/single_metal_sphere.zmaterial"),
            material_fixture,
        )
}

pub(crate) fn write_viewer_project_assets_for_fixture(
    asset_root: &std::path::Path,
    material_fixture: ViewerMaterialFixture,
) -> Result<ViewerProjectAssetGenerationReport, Box<dyn Error>> {
    if viewer_project_assets_are_ready_for_fixture(asset_root, material_fixture) {
        return Ok(ViewerProjectAssetGenerationReport::reused());
    }
    let staging_asset_root = viewer_asset_staging_root(asset_root)?;
    let _ = fs::remove_dir_all(&staging_asset_root);
    fs::create_dir_all(&staging_asset_root)?;

    let references = ViewerProjectAssetReferences::versioned(material_fixture)?;
    write_uv_sphere_model(
        staging_asset_root
            .join("models")
            .join("single_pbr_sphere.model.toml"),
        VIEWER_MODEL_URI,
        SPHERE_RINGS,
        SPHERE_SEGMENTS,
    )?;
    write_viewer_asset_meta(
        &staging_asset_root
            .join("models")
            .join("single_pbr_sphere.model.toml"),
        &references.model,
        AssetKind::Model,
    )?;
    write_viewer_material(
        staging_asset_root
            .join("materials")
            .join("single_metal_sphere.zmaterial"),
        material_fixture,
    )?;
    write_viewer_asset_meta(
        &staging_asset_root
            .join("materials")
            .join("single_metal_sphere.zmaterial"),
        &references.material,
        AssetKind::Material,
    )?;
    write_single_pbr_sphere_scene(
        staging_asset_root
            .join("scenes")
            .join("single_pbr_sphere.scene.toml"),
        &references,
    )?;
    write_viewer_asset_meta(
        &staging_asset_root
            .join("scenes")
            .join("single_pbr_sphere.scene.toml"),
        &references.scene,
        AssetKind::Scene,
    )?;

    let serialized_source_bytes = viewer_project_serialized_source_bytes(&staging_asset_root)?;
    finish_viewer_project_asset_generation(
        staging_asset_root.as_path(),
        asset_root,
        material_fixture,
        serialized_source_bytes,
    )
}

fn finish_viewer_project_asset_generation(
    staging_asset_root: &std::path::Path,
    asset_root: &std::path::Path,
    material_fixture: ViewerMaterialFixture,
    serialized_source_bytes: u64,
) -> Result<ViewerProjectAssetGenerationReport, Box<dyn Error>> {
    // A competing process may win publication after this caller generated the full tree. Preserve
    // the caller's cold-start work in the report even when its private staging tree loses.
    let _published =
        publish_viewer_project_assets(&staging_asset_root, asset_root, material_fixture)?;
    Ok(ViewerProjectAssetGenerationReport::generated(
        serialized_source_bytes,
    ))
}

fn viewer_project_serialized_source_bytes(
    asset_root: &std::path::Path,
) -> Result<u64, Box<dyn Error>> {
    VIEWER_PROJECT_SOURCE_PATHS
        .iter()
        .try_fold(0_u64, |total, relative_path| {
            Ok(total + fs::metadata(asset_root.join(relative_path))?.len())
        })
}

fn viewer_asset_staging_root(asset_root: &std::path::Path) -> Result<PathBuf, Box<dyn Error>> {
    viewer_project_staging_root(asset_root)
}

fn viewer_project_staging_root(asset_root: &std::path::Path) -> Result<PathBuf, Box<dyn Error>> {
    let project_root = asset_root
        .parent()
        .ok_or("viewer asset root has no project parent")?;
    let sequence = VIEWER_PROJECT_STAGING_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    Ok(project_root.join(format!(".zpv4-s-{}-{}", std::process::id(), sequence,)))
}

fn publish_viewer_project_assets(
    staging_asset_root: &std::path::Path,
    asset_root: &std::path::Path,
    material_fixture: ViewerMaterialFixture,
) -> Result<bool, Box<dyn Error>> {
    let mut displaced_stale_root = None;
    let mut retried_after_destination_change = false;

    loop {
        match fs::rename(staging_asset_root, asset_root) {
            Ok(()) => {
                if let Some(displaced_root) = displaced_stale_root {
                    let _ = fs::remove_dir_all(displaced_root);
                }
                return Ok(true);
            }
            Err(_) if viewer_project_assets_are_ready_for_fixture(asset_root, material_fixture) => {
                let _ = fs::remove_dir_all(staging_asset_root);
                if let Some(displaced_root) = displaced_stale_root {
                    let _ = fs::remove_dir_all(displaced_root);
                }
                return Ok(false);
            }
            Err(error) if asset_root.exists() && displaced_stale_root.is_none() => {
                let displaced_root = viewer_project_replaced_root(asset_root)?;
                if fs::rename(asset_root, &displaced_root).is_ok() {
                    displaced_stale_root = Some(displaced_root);
                    continue;
                }
                if viewer_project_assets_are_ready_for_fixture(asset_root, material_fixture) {
                    let _ = fs::remove_dir_all(staging_asset_root);
                    return Ok(false);
                }
                if !asset_root.exists() && !retried_after_destination_change {
                    retried_after_destination_change = true;
                    continue;
                }
                return Err(error.into());
            }
            Err(error) if !asset_root.exists() && !retried_after_destination_change => {
                retried_after_destination_change = true;
                continue;
            }
            Err(error) => {
                if let Some(displaced_root) = displaced_stale_root {
                    if !asset_root.exists() {
                        let _ = fs::rename(displaced_root, asset_root);
                    }
                }
                return Err(error.into());
            }
        }
    }
}

fn viewer_project_replaced_root(asset_root: &std::path::Path) -> Result<PathBuf, Box<dyn Error>> {
    let project_root = asset_root
        .parent()
        .ok_or("viewer asset root has no project parent")?;
    let sequence = VIEWER_PROJECT_STAGING_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    Ok(project_root.join(format!(".zpv4-r-{}-{}", std::process::id(), sequence,)))
}

fn write_viewer_asset_meta(
    source_path: &std::path::Path,
    reference: &AssetReference,
    kind: AssetKind,
) -> Result<(), Box<dyn Error>> {
    let file_name = source_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("viewer asset source has no UTF-8 file name")?;
    AssetMetaDocument::new(reference.uuid, reference.locator.clone(), kind)
        .save(source_path.with_file_name(format!("{file_name}.zmeta")))?;
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
            mesh_sdf: None,
            virtual_geometry: None,
        }],
    };
    fs::write(
        path,
        model
            .to_project_toml_string(|_| {
                Err::<PersistedAssetReference, _>(ReferenceResolutionError::Registry {
                    message: "viewer model has no references".to_string(),
                })
            })
            .map_err(|error| invalid_data(error.to_string()))?,
    )?;
    Ok(())
}

fn write_viewer_material(
    path: PathBuf,
    material_fixture: ViewerMaterialFixture,
) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut material = MaterialAsset {
        name: Some(material_fixture.material_name().to_string()),
        shader: asset_reference("builtin://shader/pbr.wgsl")?,
        parent: None,
        options: Default::default(),
        queue: None,
        base_color: material_fixture.base_color(),
        base_color_texture: None,
        normal_texture: None,
        metallic: material_fixture.metallic(),
        roughness: material_fixture.roughness(),
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
    if let Some(ior) = material_fixture.dielectric_ior() {
        material
            .property_values
            .insert("ior".to_string(), toml::Value::Float(ior));
    }
    fs::write(
        path,
        material
            .to_project_toml_string(|reference| {
                if reference.locator.scheme()
                    == zircon_runtime_interface::resource::ResourceScheme::Builtin
                {
                    Ok(PersistedAssetReference::builtin(reference.locator.clone()))
                } else {
                    Err(ReferenceResolutionError::Registry {
                        message: "viewer material project reference requires registry resolution"
                            .to_string(),
                    })
                }
            })
            .map_err(|error| invalid_data(error.to_string()))?,
    )?;
    Ok(())
}

fn write_single_pbr_sphere_scene(
    path: PathBuf,
    references: &ViewerProjectAssetReferences,
) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let sphere_model = references.model.clone();
    let sphere_material = references.material.clone();
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
            components: Vec::new(),
            script_bindings: Vec::new(),
        },
        zero_intensity_key_light_entity(3, "Zero Intensity Key Light"),
    ];

    fs::write(
        path,
        SceneAsset { entities }
            .to_project_toml_string(|reference| persist_viewer_project_reference(reference))
            .map_err(|error| invalid_data(error.to_string()))?,
    )?;
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
        components: Vec::new(),
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
            casts_shadow: false,
            volumetric: false,
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
        components: Vec::new(),
        script_bindings: Vec::new(),
    }
}

fn asset_reference(uri: &str) -> Result<AssetReference, Box<dyn Error>> {
    Ok(AssetReference::from_locator(AssetUri::parse(uri)?))
}

fn viewer_asset_reference(
    uri: &str,
    identity_label: &str,
) -> Result<AssetReference, Box<dyn Error>> {
    Ok(AssetReference::new(
        AssetUuid::from_stable_label(identity_label),
        AssetUri::parse(uri)?,
    ))
}

fn persist_viewer_project_reference(
    reference: &AssetReference,
) -> Result<PersistedAssetReference, ReferenceResolutionError> {
    if reference.locator.scheme() != ResourceScheme::Res {
        return Err(ReferenceResolutionError::UnsupportedScheme {
            locator: reference.locator.clone(),
        });
    }
    let path_hint = RelPath::parse(format!(
        "{VIEWER_PROJECT_ASSET_ROOT}/{}",
        reference.locator.path()
    ))
    .map_err(|error| ReferenceResolutionError::Registry {
        message: error.to_string(),
    })?;
    let asset_ref = AssetRef::try_new(
        reference.uuid,
        path_hint,
        reference.locator.label().map(str::to_string),
    )
    .map_err(|error| ReferenceResolutionError::Registry {
        message: error.to_string(),
    })?;
    Ok(PersistedAssetReference::project(asset_ref))
}

fn invalid_data(error: String) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, error)
}

#[cfg(test)]
#[path = "tests/project_assets.rs"]
mod tests;
