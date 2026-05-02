use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use image::{ImageBuffer, ImageFormat, Rgba};

use crate::asset::pipeline::manager::{AssetManager, ProjectAssetManager};
use crate::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use crate::asset::{
    AlphaMode, AssetReference, AssetUri, MaterialAsset, SceneAsset, SceneCameraAsset,
    SceneEntityAsset, SceneMeshInstanceAsset, SceneMobilityAsset, TransformAsset,
};
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle};

pub(super) struct MaterialTextureCaptureTestAssets {
    pub(super) asset_manager: Arc<ProjectAssetManager>,
    pub(super) root: PathBuf,
    pub(super) base_color_red: ResourceHandle<MaterialMarker>,
    pub(super) base_color_blue: ResourceHandle<MaterialMarker>,
    pub(super) emissive_warm: ResourceHandle<MaterialMarker>,
    pub(super) emissive_cool: ResourceHandle<MaterialMarker>,
    pub(super) rough_dielectric: ResourceHandle<MaterialMarker>,
    pub(super) smooth_metallic: ResourceHandle<MaterialMarker>,
    pub(super) flat_normal: ResourceHandle<MaterialMarker>,
    pub(super) tilted_normal: ResourceHandle<MaterialMarker>,
    pub(super) open_occlusion: ResourceHandle<MaterialMarker>,
    pub(super) blocked_occlusion: ResourceHandle<MaterialMarker>,
}

pub(super) struct MaterialVisibilityCaptureTestAssets {
    pub(super) asset_manager: Arc<ProjectAssetManager>,
    pub(super) root: PathBuf,
    pub(super) single_sided_white: ResourceHandle<MaterialMarker>,
    pub(super) double_sided_white: ResourceHandle<MaterialMarker>,
    pub(super) opaque_white: ResourceHandle<MaterialMarker>,
    pub(super) masked_cutout_white: ResourceHandle<MaterialMarker>,
    pub(super) blended_white: ResourceHandle<MaterialMarker>,
}

pub(super) fn material_capture_test_assets() -> (
    Arc<ProjectAssetManager>,
    PathBuf,
    ResourceHandle<MaterialMarker>,
    ResourceHandle<MaterialMarker>,
) {
    let root = unique_temp_project_root("hybrid_gi_scene_prepare_material_capture");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "HybridGiScenePrepareMaterialCapture",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_triangle_obj(paths.assets_root().join("models").join("triangle.obj"));
    write_material_asset(
        paths
            .assets_root()
            .join("materials")
            .join("black.material.toml"),
        [0.0, 0.0, 0.0, 1.0],
        [0.0, 0.0, 0.0],
    );
    write_material_asset(
        paths
            .assets_root()
            .join("materials")
            .join("emissive.material.toml"),
        [0.0, 0.0, 0.0, 1.0],
        [1.0, 0.2, 0.1],
    );
    write_scene_asset(
        paths.assets_root().join("scenes").join("main.scene.toml"),
        "res://materials/black.material.toml",
    );

    let asset_manager = open_test_project(&root);
    let black_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/black.material.toml");
    let emissive_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/emissive.material.toml");

    (asset_manager, root, black_material, emissive_material)
}

pub(super) fn material_surface_response_test_assets() -> (
    Arc<ProjectAssetManager>,
    PathBuf,
    ResourceHandle<MaterialMarker>,
    ResourceHandle<MaterialMarker>,
    ResourceHandle<MaterialMarker>,
    ResourceHandle<MaterialMarker>,
) {
    let root = unique_temp_project_root("hybrid_gi_scene_prepare_surface_response");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "HybridGiScenePrepareSurfaceResponse",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_triangle_obj(paths.assets_root().join("models").join("triangle.obj"));
    write_material_asset_with_surface(
        paths
            .assets_root()
            .join("materials")
            .join("smooth_white.material.toml"),
        [1.0, 1.0, 1.0, 1.0],
        [0.0, 0.0, 0.0],
        0.0,
        0.05,
    );
    write_material_asset_with_surface(
        paths
            .assets_root()
            .join("materials")
            .join("rough_white.material.toml"),
        [1.0, 1.0, 1.0, 1.0],
        [0.0, 0.0, 0.0],
        0.0,
        0.95,
    );
    write_material_asset_with_surface(
        paths
            .assets_root()
            .join("materials")
            .join("dielectric_red.material.toml"),
        [1.0, 0.2, 0.1, 1.0],
        [0.0, 0.0, 0.0],
        0.0,
        0.2,
    );
    write_material_asset_with_surface(
        paths
            .assets_root()
            .join("materials")
            .join("metallic_red.material.toml"),
        [1.0, 0.2, 0.1, 1.0],
        [0.0, 0.0, 0.0],
        1.0,
        0.2,
    );
    write_scene_asset(
        paths.assets_root().join("scenes").join("main.scene.toml"),
        "res://materials/smooth_white.material.toml",
    );

    let asset_manager = open_test_project(&root);
    let smooth_white = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/smooth_white.material.toml",
    );
    let rough_white = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/rough_white.material.toml",
    );
    let dielectric_red = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/dielectric_red.material.toml",
    );
    let metallic_red = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/metallic_red.material.toml",
    );

    (
        asset_manager,
        root,
        smooth_white,
        rough_white,
        dielectric_red,
        metallic_red,
    )
}

pub(super) fn material_texture_capture_test_assets() -> MaterialTextureCaptureTestAssets {
    let root = unique_temp_project_root("hybrid_gi_scene_prepare_texture_capture");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "HybridGiScenePrepareTextureCapture",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_triangle_obj(paths.assets_root().join("models").join("triangle.obj"));
    write_solid_png(
        paths.assets_root().join("textures").join("base_red.png"),
        [255, 24, 16, 255],
    );
    write_solid_png(
        paths.assets_root().join("textures").join("base_blue.png"),
        [24, 32, 255, 255],
    );
    write_solid_png(
        paths
            .assets_root()
            .join("textures")
            .join("emissive_warm.png"),
        [255, 96, 24, 255],
    );
    write_solid_png(
        paths
            .assets_root()
            .join("textures")
            .join("emissive_cool.png"),
        [24, 96, 255, 255],
    );
    write_solid_png(
        paths
            .assets_root()
            .join("textures")
            .join("mr_rough_dielectric.png"),
        [0, 240, 16, 255],
    );
    write_solid_png(
        paths
            .assets_root()
            .join("textures")
            .join("mr_smooth_metallic.png"),
        [0, 16, 240, 255],
    );
    write_solid_png(
        paths.assets_root().join("textures").join("normal_flat.png"),
        [128, 128, 255, 255],
    );
    write_solid_png(
        paths
            .assets_root()
            .join("textures")
            .join("normal_tilted.png"),
        [255, 128, 128, 255],
    );
    write_solid_png(
        paths
            .assets_root()
            .join("textures")
            .join("occlusion_open.png"),
        [255, 255, 255, 255],
    );
    write_solid_png(
        paths
            .assets_root()
            .join("textures")
            .join("occlusion_blocked.png"),
        [32, 32, 32, 255],
    );

    write_material_asset_with_textures(
        paths
            .assets_root()
            .join("materials")
            .join("base_red_texture.material.toml"),
        [1.0, 1.0, 1.0, 1.0],
        [0.0, 0.0, 0.0],
        0.0,
        0.5,
        Some("res://textures/base_red.png"),
        None,
        None,
        None,
        None,
    );
    write_material_asset_with_textures(
        paths
            .assets_root()
            .join("materials")
            .join("base_blue_texture.material.toml"),
        [1.0, 1.0, 1.0, 1.0],
        [0.0, 0.0, 0.0],
        0.0,
        0.5,
        Some("res://textures/base_blue.png"),
        None,
        None,
        None,
        None,
    );
    write_material_asset_with_textures(
        paths
            .assets_root()
            .join("materials")
            .join("emissive_warm_texture.material.toml"),
        [0.0, 0.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
        0.0,
        1.0,
        None,
        None,
        Some("res://textures/emissive_warm.png"),
        None,
        None,
    );
    write_material_asset_with_textures(
        paths
            .assets_root()
            .join("materials")
            .join("emissive_cool_texture.material.toml"),
        [0.0, 0.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
        0.0,
        1.0,
        None,
        None,
        Some("res://textures/emissive_cool.png"),
        None,
        None,
    );
    write_material_asset_with_textures(
        paths
            .assets_root()
            .join("materials")
            .join("rough_dielectric_texture.material.toml"),
        [1.0, 0.25, 0.1, 1.0],
        [0.0, 0.0, 0.0],
        1.0,
        1.0,
        None,
        Some("res://textures/mr_rough_dielectric.png"),
        None,
        None,
        None,
    );
    write_material_asset_with_textures(
        paths
            .assets_root()
            .join("materials")
            .join("smooth_metallic_texture.material.toml"),
        [1.0, 0.25, 0.1, 1.0],
        [0.0, 0.0, 0.0],
        1.0,
        1.0,
        None,
        Some("res://textures/mr_smooth_metallic.png"),
        None,
        None,
        None,
    );
    write_material_asset_with_textures(
        paths
            .assets_root()
            .join("materials")
            .join("flat_normal_texture.material.toml"),
        [1.0, 1.0, 1.0, 1.0],
        [0.0, 0.0, 0.0],
        0.0,
        0.35,
        None,
        None,
        None,
        Some("res://textures/normal_flat.png"),
        None,
    );
    write_material_asset_with_textures(
        paths
            .assets_root()
            .join("materials")
            .join("tilted_normal_texture.material.toml"),
        [1.0, 1.0, 1.0, 1.0],
        [0.0, 0.0, 0.0],
        0.0,
        0.35,
        None,
        None,
        None,
        Some("res://textures/normal_tilted.png"),
        None,
    );
    write_material_asset_with_textures(
        paths
            .assets_root()
            .join("materials")
            .join("open_occlusion_texture.material.toml"),
        [1.0, 1.0, 1.0, 1.0],
        [0.0, 0.0, 0.0],
        0.0,
        1.0,
        None,
        None,
        None,
        None,
        Some("res://textures/occlusion_open.png"),
    );
    write_material_asset_with_textures(
        paths
            .assets_root()
            .join("materials")
            .join("blocked_occlusion_texture.material.toml"),
        [1.0, 1.0, 1.0, 1.0],
        [0.0, 0.0, 0.0],
        0.0,
        1.0,
        None,
        None,
        None,
        None,
        Some("res://textures/occlusion_blocked.png"),
    );
    write_scene_asset(
        paths.assets_root().join("scenes").join("main.scene.toml"),
        "res://materials/base_red_texture.material.toml",
    );

    let asset_manager = open_test_project(&root);
    MaterialTextureCaptureTestAssets {
        base_color_red: resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/base_red_texture.material.toml",
        ),
        base_color_blue: resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/base_blue_texture.material.toml",
        ),
        emissive_warm: resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/emissive_warm_texture.material.toml",
        ),
        emissive_cool: resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/emissive_cool_texture.material.toml",
        ),
        rough_dielectric: resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/rough_dielectric_texture.material.toml",
        ),
        smooth_metallic: resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/smooth_metallic_texture.material.toml",
        ),
        flat_normal: resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/flat_normal_texture.material.toml",
        ),
        tilted_normal: resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/tilted_normal_texture.material.toml",
        ),
        open_occlusion: resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/open_occlusion_texture.material.toml",
        ),
        blocked_occlusion: resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/blocked_occlusion_texture.material.toml",
        ),
        asset_manager,
        root,
    }
}

pub(super) fn material_visibility_capture_test_assets() -> MaterialVisibilityCaptureTestAssets {
    let root = unique_temp_project_root("hybrid_gi_scene_prepare_visibility_capture");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "HybridGiScenePrepareVisibilityCapture",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_triangle_obj(paths.assets_root().join("models").join("triangle.obj"));
    write_material_asset_with_capture_options(
        paths
            .assets_root()
            .join("materials")
            .join("single_sided_white.material.toml"),
        [1.0, 1.0, 1.0, 1.0],
        [0.0, 0.0, 0.0],
        0.0,
        0.25,
        None,
        None,
        None,
        None,
        None,
        AlphaMode::Opaque,
        false,
    );
    write_material_asset_with_capture_options(
        paths
            .assets_root()
            .join("materials")
            .join("double_sided_white.material.toml"),
        [1.0, 1.0, 1.0, 1.0],
        [0.0, 0.0, 0.0],
        0.0,
        0.25,
        None,
        None,
        None,
        None,
        None,
        AlphaMode::Opaque,
        true,
    );
    write_material_asset_with_capture_options(
        paths
            .assets_root()
            .join("materials")
            .join("opaque_white.material.toml"),
        [1.0, 1.0, 1.0, 1.0],
        [0.0, 0.0, 0.0],
        0.0,
        0.25,
        None,
        None,
        None,
        None,
        None,
        AlphaMode::Opaque,
        false,
    );
    write_material_asset_with_capture_options(
        paths
            .assets_root()
            .join("materials")
            .join("masked_cutout_white.material.toml"),
        [1.0, 1.0, 1.0, 0.2],
        [0.0, 0.0, 0.0],
        0.0,
        0.25,
        None,
        None,
        None,
        None,
        None,
        AlphaMode::Mask { cutoff: 0.5 },
        false,
    );
    write_material_asset_with_capture_options(
        paths
            .assets_root()
            .join("materials")
            .join("blended_white.material.toml"),
        [1.0, 1.0, 1.0, 0.2],
        [0.0, 0.0, 0.0],
        0.0,
        0.25,
        None,
        None,
        None,
        None,
        None,
        AlphaMode::Blend,
        false,
    );
    write_scene_asset(
        paths.assets_root().join("scenes").join("main.scene.toml"),
        "res://materials/opaque_white.material.toml",
    );

    let asset_manager = open_test_project(&root);
    MaterialVisibilityCaptureTestAssets {
        single_sided_white: resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/single_sided_white.material.toml",
        ),
        double_sided_white: resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/double_sided_white.material.toml",
        ),
        opaque_white: resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/opaque_white.material.toml",
        ),
        masked_cutout_white: resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/masked_cutout_white.material.toml",
        ),
        blended_white: resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/blended_white.material.toml",
        ),
        asset_manager,
        root,
    }
}

pub(super) fn model_handle(asset_manager: &ProjectAssetManager) -> ResourceHandle<ModelMarker> {
    resource_handle::<ModelMarker>(asset_manager, "res://models/triangle.obj")
}

fn open_test_project(root: &PathBuf) -> Arc<ProjectAssetManager> {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    asset_manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let mut project = ProjectManager::open(root).unwrap();
    project.scan_and_import().unwrap();
    asset_manager
}

fn resource_handle<T>(asset_manager: &ProjectAssetManager, uri: &str) -> ResourceHandle<T> {
    ResourceHandle::<T>::new(
        asset_manager
            .resolve_asset_id(&AssetUri::parse(uri).unwrap())
            .unwrap_or_else(|| panic!("expected asset id for {uri}")),
    )
}

fn unique_temp_project_root(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("zircon_hybrid_gi_{label}_{unique}"))
}

fn write_material_asset(path: PathBuf, base_color: [f32; 4], emissive: [f32; 3]) {
    write_material_asset_with_surface(path, base_color, emissive, 0.0, 1.0);
}

fn write_material_asset_with_surface(
    path: PathBuf,
    base_color: [f32; 4],
    emissive: [f32; 3],
    metallic: f32,
    roughness: f32,
) {
    write_material_asset_with_textures(
        path, base_color, emissive, metallic, roughness, None, None, None, None, None,
    );
}

fn write_material_asset_with_textures(
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
fn write_material_asset_with_capture_options(
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
    };
    fs::write(path, material.to_toml_string().unwrap()).unwrap();
}

fn write_solid_png(path: PathBuf, rgba: [u8; 4]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    ImageBuffer::<Rgba<u8>, _>::from_fn(2, 2, |_x, _y| Rgba(rgba))
        .save_with_format(path, ImageFormat::Png)
        .unwrap();
}

fn write_triangle_obj(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(
        path,
        "\
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.0 1.0 0.0
vt 0.0 0.0
vt 1.0 0.0
vt 0.0 1.0
vn 0.0 0.0 1.0
f 1/1/1 2/2/1 3/3/1
",
    )
    .unwrap();
}

fn write_scene_asset(path: PathBuf, material_uri: &str) {
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
                }),
                mesh: None,
                directional_light: None,
                point_light: None,
                spot_light: None,
                rigid_body: None,
                collider: None,
                joint: None,
                animation_skeleton: None,
                animation_player: None,
                animation_sequence_player: None,
                animation_graph_player: None,
                animation_state_machine_player: None,
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
                    model: asset_reference("res://models/triangle.obj"),
                    material: asset_reference(material_uri),
                }),
                directional_light: None,
                point_light: None,
                spot_light: None,
                rigid_body: None,
                collider: None,
                joint: None,
                animation_skeleton: None,
                animation_player: None,
                animation_sequence_player: None,
                animation_graph_player: None,
                animation_state_machine_player: None,
            },
        ],
    };
    fs::write(path, scene.to_toml_string().unwrap()).unwrap();
}

fn asset_reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}
