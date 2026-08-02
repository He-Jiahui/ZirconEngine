use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use zircon_runtime::asset::assets::{
    AlphaMode, MaterialAsset, SceneAsset, SceneCameraAsset, SceneDirectionalLightAsset,
    SceneEntityAsset, SceneMeshInstanceAsset, SceneMobilityAsset, TransformAsset,
};
use zircon_runtime::asset::{
    AssetKind, AssetReference, AssetUri, AssetUuid, MeshVertex, ModelAsset, ModelPrimitiveAsset,
    ReferenceResolutionError, project::AssetMetaDocument,
};
use zircon_runtime::core::framework::render::{DEFAULT_RENDER_LAYER_MASK, ProjectionMode};
use zircon_runtime::core::math::{Transform, Vec2, Vec3};
use zircon_runtime_interface::project::{AssetRef, PersistedAssetReference, RelPath};
use zircon_runtime_interface::resource::ResourceScheme;

use crate::camera::{CAMERA_FOV_Y_RADIANS, DEFAULT_CAMERA_RADIUS, SPHERE_CENTER, SPHERE_SCALE};

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
    fn versioned() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            model: viewer_asset_reference(VIEWER_MODEL_URI, "viewer-project-v4/model")?,
            material: viewer_asset_reference(VIEWER_MATERIAL_URI, "viewer-project-v4/material")?,
            scene: viewer_asset_reference(VIEWER_SCENE_URI, "viewer-project-v4/scene")?,
        })
    }
}

pub(crate) fn viewer_project_assets_are_ready(asset_root: &std::path::Path) -> bool {
    asset_root.is_dir()
        && VIEWER_PROJECT_ASSET_PATHS
            .iter()
            .all(|relative_path| asset_root.join(relative_path).is_file())
}

pub(crate) fn write_viewer_project_assets(
    asset_root: &std::path::Path,
) -> Result<ViewerProjectAssetGenerationReport, Box<dyn Error>> {
    if viewer_project_assets_are_ready(asset_root) {
        return Ok(ViewerProjectAssetGenerationReport::reused());
    }
    let staging_asset_root = viewer_asset_staging_root(asset_root)?;
    let _ = fs::remove_dir_all(&staging_asset_root);
    fs::create_dir_all(&staging_asset_root)?;

    let references = ViewerProjectAssetReferences::versioned()?;
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
    write_perfect_mirror_material(
        staging_asset_root
            .join("materials")
            .join("single_metal_sphere.zmaterial"),
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
        serialized_source_bytes,
    )
}

fn finish_viewer_project_asset_generation(
    staging_asset_root: &std::path::Path,
    asset_root: &std::path::Path,
    serialized_source_bytes: u64,
) -> Result<ViewerProjectAssetGenerationReport, Box<dyn Error>> {
    // A competing process may win publication after this caller generated the full tree. Preserve
    // the caller's cold-start work in the report even when its private staging tree loses.
    let _published = publish_viewer_project_assets(&staging_asset_root, asset_root)?;
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
    Ok(project_root.join(format!(
        ".zircon_shader_pbr_viewer_assets_v4_stage_{}_{}",
        std::process::id(),
        sequence,
    )))
}

fn publish_viewer_project_assets(
    staging_asset_root: &std::path::Path,
    asset_root: &std::path::Path,
) -> Result<bool, Box<dyn Error>> {
    let mut displaced_incomplete_root = None;
    let mut retried_after_destination_change = false;

    loop {
        match fs::rename(staging_asset_root, asset_root) {
            Ok(()) => {
                if let Some(displaced_root) = displaced_incomplete_root {
                    let _ = fs::remove_dir_all(displaced_root);
                }
                return Ok(true);
            }
            Err(_) if viewer_project_assets_are_ready(asset_root) => {
                let _ = fs::remove_dir_all(staging_asset_root);
                if let Some(displaced_root) = displaced_incomplete_root {
                    let _ = fs::remove_dir_all(displaced_root);
                }
                return Ok(false);
            }
            Err(error) if asset_root.exists() && displaced_incomplete_root.is_none() => {
                let displaced_root = viewer_project_incomplete_root(asset_root)?;
                if fs::rename(asset_root, &displaced_root).is_ok() {
                    displaced_incomplete_root = Some(displaced_root);
                    continue;
                }
                if viewer_project_assets_are_ready(asset_root) {
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
                if let Some(displaced_root) = displaced_incomplete_root {
                    if !asset_root.exists() {
                        let _ = fs::rename(displaced_root, asset_root);
                    }
                }
                return Err(error.into());
            }
        }
    }
}

fn viewer_project_incomplete_root(asset_root: &std::path::Path) -> Result<PathBuf, Box<dyn Error>> {
    let project_root = asset_root
        .parent()
        .ok_or("viewer asset root has no project parent")?;
    let sequence = VIEWER_PROJECT_STAGING_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    Ok(project_root.join(format!(
        ".zircon_shader_pbr_viewer_assets_v4_incomplete_{}_{}",
        std::process::id(),
        sequence,
    )))
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
mod tests {
    use super::{
        AssetMetaDocument, AssetReference, MaterialAsset, ReferenceResolutionError, SPHERE_RINGS,
        SPHERE_SEGMENTS, SceneAsset, VIEWER_PROJECT_ASSET_PATHS, VIEWER_PROJECT_ASSET_ROOT,
        VIEWER_PROJECT_SOURCE_PATHS, viewer_project_assets_are_ready,
        write_perfect_mirror_material, write_viewer_project_assets,
    };
    use zircon_runtime::asset::assets::ZMaterialDocument;

    #[test]
    fn viewer_mirror_mesh_stays_within_its_startup_triangle_budget() {
        assert_eq!(SPHERE_RINGS * SPHERE_SEGMENTS * 2, 16_384);
    }

    #[test]
    fn viewer_project_reuse_requires_a_completed_asset_tree() {
        let root = std::env::temp_dir().join(format!(
            "zircon_shader_pbr_viewer_asset_ready_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let asset_root = root.join(VIEWER_PROJECT_ASSET_ROOT);
        std::fs::create_dir_all(&root).expect("test cache root should be created");

        assert!(!viewer_project_assets_are_ready(&asset_root));
        for relative_path in VIEWER_PROJECT_ASSET_PATHS {
            let path = asset_root.join(relative_path);
            std::fs::create_dir_all(path.parent().expect("asset path should have a parent"))
                .expect("test asset parent should be created");
            std::fs::write(path, "fixture\n").expect("test asset should be written");
        }
        assert!(viewer_project_assets_are_ready(&asset_root));

        std::fs::remove_file(asset_root.join(VIEWER_PROJECT_ASSET_PATHS[1]))
            .expect("test asset should be removable");
        assert!(
            !viewer_project_assets_are_ready(&asset_root),
            "a partial cache must regenerate the viewer project assets"
        );

        std::fs::remove_dir_all(&root).expect("test cache root should be removed");
    }

    #[test]
    fn viewer_project_publishes_stable_sidecars_and_scene_references_without_preopening() {
        let root = std::env::temp_dir().join(format!(
            "zircon_shader_pbr_viewer_project_assets_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let asset_root = root.join(VIEWER_PROJECT_ASSET_ROOT);
        std::fs::create_dir_all(&root).expect("test cache root should be created");

        let cold_report = write_viewer_project_assets(&asset_root)
            .expect("viewer project assets should publish without opening a project manager");
        assert!(viewer_project_assets_are_ready(&asset_root));
        assert_eq!(cold_report.mesh_generation_samples(), 1);
        assert_eq!(cold_report.filesystem_writes(), 6);
        assert!(cold_report.serialized_source_bytes() > 0);

        let references = super::ViewerProjectAssetReferences::versioned()
            .expect("viewer references should be well-formed");
        let model_meta =
            AssetMetaDocument::load(asset_root.join("models/single_pbr_sphere.model.toml.zmeta"))
                .expect("model sidecar should be readable");
        let material_meta = AssetMetaDocument::load(
            asset_root.join("materials/single_metal_sphere.zmaterial.zmeta"),
        )
        .expect("material sidecar should be readable");
        assert_eq!(model_meta.uuid, references.model.uuid);
        assert_eq!(material_meta.uuid, references.material.uuid);

        let scene_source =
            std::fs::read_to_string(asset_root.join("scenes/single_pbr_sphere.scene.toml"))
                .expect("viewer scene should be readable");
        let expected_references = [references.model.clone(), references.material.clone()];
        let scene = SceneAsset::from_project_toml_str(&scene_source, |persisted| {
            let project_reference =
                persisted
                    .project_ref()
                    .ok_or_else(|| ReferenceResolutionError::Registry {
                        message: "viewer scene should only contain project references".to_string(),
                    })?;
            let reference = expected_references
                .iter()
                .find(|reference| reference.uuid == project_reference.guid())
                .cloned()
                .ok_or_else(|| ReferenceResolutionError::Registry {
                    message: "viewer scene should retain its stable generated identity".to_string(),
                })?;
            assert_eq!(
                project_reference.path_hint().as_str(),
                format!("{VIEWER_PROJECT_ASSET_ROOT}/{}", reference.locator.path()),
                "viewer scene must preserve the project-relative source path hint"
            );
            Ok(reference)
        })
        .expect("viewer scene should deserialize against its generated sidecars");
        assert_eq!(scene.direct_references(), expected_references.to_vec());

        let warm_report = write_viewer_project_assets(&asset_root)
            .expect("ready viewer project assets should be reused without writes");
        assert_eq!(
            warm_report,
            super::ViewerProjectAssetGenerationReport::reused()
        );

        std::fs::remove_dir_all(&root).expect("test cache root should be removed");
    }

    #[test]
    fn viewer_project_replaces_an_incomplete_versioned_tree_without_deleting_a_ready_one() {
        let root = std::env::temp_dir().join(format!(
            "zircon_shader_pbr_viewer_incomplete_project_assets_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let asset_root = root.join(VIEWER_PROJECT_ASSET_ROOT);
        std::fs::create_dir_all(&asset_root).expect("incomplete asset root should be created");
        std::fs::write(asset_root.join("stale.partial"), "incomplete\n")
            .expect("incomplete asset marker should be written");

        let report = write_viewer_project_assets(&asset_root)
            .expect("an incomplete versioned tree should be replaced by a complete tree");
        assert!(viewer_project_assets_are_ready(&asset_root));
        assert_eq!(report.mesh_generation_samples(), 1);
        assert_eq!(report.filesystem_writes(), 6);
        assert!(
            !asset_root.join("stale.partial").exists(),
            "the published tree must contain only the current generated artifacts"
        );
        let incomplete_roots = std::fs::read_dir(&root)
            .expect("project root should be readable")
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".zircon_shader_pbr_viewer_assets_v4_incomplete_")
            })
            .count();
        assert_eq!(
            incomplete_roots, 0,
            "the displaced incomplete tree should be removed after publication"
        );

        std::fs::remove_dir_all(&root).expect("test cache root should be removed");
    }

    #[test]
    fn viewer_asset_writer_leaves_project_open_and_import_to_the_runtime_manager() {
        const SOURCE: &str = include_str!("project_assets.rs");
        let writer = SOURCE
            .split_once("pub(crate) fn write_viewer_project_assets")
            .and_then(|(_, source)| source.split_once("fn viewer_asset_staging_root"))
            .map(|(writer, _)| writer)
            .expect("viewer asset writer should retain a bounded implementation");

        assert!(writer.contains("publish_viewer_project_assets"));
        assert!(writer.contains("write_viewer_asset_meta"));
        assert!(writer.contains("ViewerProjectAssetGenerationReport::generated"));
        assert_eq!(VIEWER_PROJECT_SOURCE_PATHS.len(), 3);
        assert!(
            !SOURCE.contains("remove_dir_all(asset_root)"),
            "a competing publication must never delete a completed immutable cache"
        );
        assert!(
            !writer.contains("ProjectManager::open(") && !writer.contains("scan_and_import("),
            "the runtime AssetManager owns the sole project open and import generation"
        );
    }

    #[test]
    fn competing_viewer_project_publish_reuses_the_completed_immutable_asset_tree() {
        let root = std::env::temp_dir().join(format!(
            "zircon_shader_pbr_viewer_publish_contention_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let asset_root = root.join(VIEWER_PROJECT_ASSET_ROOT);
        for relative_path in VIEWER_PROJECT_ASSET_PATHS {
            let path = asset_root.join(relative_path);
            std::fs::create_dir_all(path.parent().expect("asset path should have a parent"))
                .expect("completed asset parent should be created");
            std::fs::write(path, "completed\n").expect("completed asset should be written");
        }
        let staging_root = root.join("competing-staging-root");
        std::fs::create_dir_all(&staging_root).expect("competing staging root should be created");
        std::fs::write(staging_root.join("partial.asset"), "partial\n")
            .expect("competing staging payload should be written");

        assert!(
            !super::publish_viewer_project_assets(&staging_root, &asset_root)
                .expect("a completed immutable asset root should win publication"),
            "a competing publisher must reuse the already-complete versioned tree"
        );
        assert!(viewer_project_assets_are_ready(&asset_root));
        assert!(
            !staging_root.exists(),
            "a losing publisher must discard only its private staging tree"
        );

        std::fs::remove_dir_all(&root).expect("test cache root should be removed");
    }

    #[test]
    fn losing_cold_publication_keeps_its_completed_generation_report() {
        let root = std::env::temp_dir().join(format!(
            "zircon_shader_pbr_viewer_generation_report_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let asset_root = root.join(VIEWER_PROJECT_ASSET_ROOT);
        for relative_path in VIEWER_PROJECT_ASSET_PATHS {
            let path = asset_root.join(relative_path);
            std::fs::create_dir_all(path.parent().expect("asset path should have a parent"))
                .expect("completed asset parent should be created");
            std::fs::write(path, "completed\n").expect("completed asset should be written");
        }
        let staging_root = root.join("losing-staging-root");
        std::fs::create_dir_all(&staging_root).expect("losing staging root should be created");
        std::fs::write(staging_root.join("generated.asset"), "generated\n")
            .expect("generated staging payload should be written");

        let report = super::finish_viewer_project_asset_generation(&staging_root, &asset_root, 123)
            .expect("a losing cold publication should retain its completed work report");
        assert_eq!(report.mesh_generation_samples(), 1);
        assert_eq!(report.filesystem_writes(), 6);
        assert_eq!(report.serialized_source_bytes(), 123);
        assert!(
            !staging_root.exists(),
            "a losing publication should discard only its private staging tree"
        );
        assert!(viewer_project_assets_are_ready(&asset_root));

        std::fs::remove_dir_all(&root).expect("test cache root should be removed");
    }

    #[test]
    fn viewer_mirror_material_matches_environment_only_prewarm_variant() {
        let root = std::env::temp_dir().join(format!(
            "zircon_shader_pbr_viewer_mirror_material_test_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let material_path = root.join("single_metal_sphere.zmaterial");

        write_perfect_mirror_material(material_path.clone())
            .expect("viewer mirror material should be generated");
        let source = std::fs::read_to_string(&material_path)
            .expect("viewer mirror material should be readable");
        let document: toml::Value =
            toml::from_str(&source).expect("viewer mirror material should remain valid TOML");
        let overrides = document
            .get("overrides")
            .and_then(toml::Value::as_table)
            .expect("viewer mirror material should keep its PBR overrides");

        assert!(
            source.contains("builtin://shader/pbr.wgsl"),
            "viewer mirror material must use the builtin PBR shader warmed by the renderer"
        );
        assert_eq!(
            overrides
                .get("lighting_model")
                .and_then(toml::Value::as_str),
            Some("pbr")
        );
        assert_eq!(
            overrides
                .get("receive_shadows")
                .and_then(toml::Value::as_bool),
            Some(false),
            "the viewer must reuse the prewarmed no-shadow-receiver Base variant"
        );
        assert_eq!(
            overrides.get("metallic").and_then(toml::Value::as_float),
            Some(1.0)
        );
        assert_eq!(
            overrides.get("roughness").and_then(toml::Value::as_float),
            Some(0.0)
        );
        assert!(
            document.get("textures").is_none(),
            "the environment-only viewer prewarms the static no-texture material variant"
        );

        let material = ZMaterialDocument::from_project_toml_str(&source, |reference| {
            reference
                .builtin_locator()
                .cloned()
                .map(AssetReference::from_locator)
                .ok_or_else(|| ReferenceResolutionError::Registry {
                    message: "viewer material test expects a builtin shader reference".to_string(),
                })
        })
        .map(MaterialAsset::from_zmaterial_document)
        .expect("viewer mirror material should deserialize into its runtime asset");
        let descriptor = material.standard_material_descriptor();
        assert!(
            !descriptor.receive_shadows,
            "the runtime material descriptor must keep the no-shadow-receiver prewarm key"
        );
        assert!(
            descriptor.base_color_texture.is_none()
                && descriptor.normal_texture.is_none()
                && descriptor.metallic_roughness_texture.is_none()
                && descriptor.occlusion_texture.is_none()
                && descriptor.emissive_texture.is_none(),
            "the runtime material descriptor must retain the prewarm's static no-texture key"
        );

        std::fs::remove_dir_all(&root).expect("test cache root should be removed");
    }
}
