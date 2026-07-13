use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::pipeline::manager::{AssetManager, ProjectAssetManager};
use zircon_runtime::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use zircon_runtime::asset::{AlphaMode, AssetUri};
use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle};

#[path = "hybrid_gi_scene_prepare_material_fixtures/project_documents.rs"]
mod project_documents;

use project_documents::{
    write_material_asset, write_material_asset_with_capture_options,
    write_material_asset_with_surface, write_material_asset_with_textures, write_scene_asset,
    write_solid_ppm, write_triangle_model,
};

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
    let manifest = ProjectManifest::new(
        "HybridGiScenePrepareMaterialCapture",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    );
    paths.ensure_layout(&manifest.asset_roots).unwrap();
    let asset_root = manifest.primary_asset_root().unwrap().clone();
    manifest.save(paths.manifest_path()).unwrap();

    write_triangle_model(
        &asset_root,
        paths
            .asset_root(&asset_root)
            .join("models")
            .join("triangle.model.toml"),
    );
    write_material_asset(
        &asset_root,
        paths
            .asset_root(&asset_root)
            .join("materials")
            .join("black.zmaterial"),
        [0.0, 0.0, 0.0, 1.0],
        [0.0, 0.0, 0.0],
    );
    write_material_asset(
        &asset_root,
        paths
            .asset_root(&asset_root)
            .join("materials")
            .join("emissive.zmaterial"),
        [0.0, 0.0, 0.0, 1.0],
        [1.0, 0.2, 0.1],
    );
    write_scene_asset(
        &asset_root,
        paths
            .asset_root(&asset_root)
            .join("scenes")
            .join("main.scene.toml"),
        "res://materials/black.zmaterial",
    );

    let asset_manager = open_test_project(&root);
    let black_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/black.zmaterial");
    let emissive_material =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/emissive.zmaterial");

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
    let manifest = ProjectManifest::new(
        "HybridGiScenePrepareSurfaceResponse",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    );
    paths.ensure_layout(&manifest.asset_roots).unwrap();
    let asset_root = manifest.primary_asset_root().unwrap().clone();
    manifest.save(paths.manifest_path()).unwrap();

    write_triangle_model(
        &asset_root,
        paths
            .asset_root(&asset_root)
            .join("models")
            .join("triangle.model.toml"),
    );
    write_material_asset_with_surface(
        &asset_root,
        paths
            .asset_root(&asset_root)
            .join("materials")
            .join("smooth_white.zmaterial"),
        [1.0, 1.0, 1.0, 1.0],
        [0.0, 0.0, 0.0],
        0.0,
        0.05,
    );
    write_material_asset_with_surface(
        &asset_root,
        paths
            .asset_root(&asset_root)
            .join("materials")
            .join("rough_white.zmaterial"),
        [1.0, 1.0, 1.0, 1.0],
        [0.0, 0.0, 0.0],
        0.0,
        0.95,
    );
    write_material_asset_with_surface(
        &asset_root,
        paths
            .asset_root(&asset_root)
            .join("materials")
            .join("dielectric_red.zmaterial"),
        [1.0, 0.2, 0.1, 1.0],
        [0.0, 0.0, 0.0],
        0.0,
        0.2,
    );
    write_material_asset_with_surface(
        &asset_root,
        paths
            .asset_root(&asset_root)
            .join("materials")
            .join("metallic_red.zmaterial"),
        [1.0, 0.2, 0.1, 1.0],
        [0.0, 0.0, 0.0],
        1.0,
        0.2,
    );
    write_scene_asset(
        &asset_root,
        paths
            .asset_root(&asset_root)
            .join("scenes")
            .join("main.scene.toml"),
        "res://materials/smooth_white.zmaterial",
    );

    let asset_manager = open_test_project(&root);
    let smooth_white =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/smooth_white.zmaterial");
    let rough_white =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/rough_white.zmaterial");
    let dielectric_red = resource_handle::<MaterialMarker>(
        &asset_manager,
        "res://materials/dielectric_red.zmaterial",
    );
    let metallic_red =
        resource_handle::<MaterialMarker>(&asset_manager, "res://materials/metallic_red.zmaterial");

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
    let manifest = ProjectManifest::new(
        "HybridGiScenePrepareTextureCapture",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    );
    paths.ensure_layout(&manifest.asset_roots).unwrap();
    let asset_root = manifest.primary_asset_root().unwrap().clone();
    manifest.save(paths.manifest_path()).unwrap();

    write_triangle_model(
        &asset_root,
        paths
            .asset_root(&asset_root)
            .join("models")
            .join("triangle.model.toml"),
    );
    write_solid_ppm(
        paths
            .asset_root(&asset_root)
            .join("textures")
            .join("base_red.ppm"),
        [255, 24, 16, 255],
    );
    write_solid_ppm(
        paths
            .asset_root(&asset_root)
            .join("textures")
            .join("base_blue.ppm"),
        [24, 32, 255, 255],
    );
    write_solid_ppm(
        paths
            .asset_root(&asset_root)
            .join("textures")
            .join("emissive_warm.ppm"),
        [255, 96, 24, 255],
    );
    write_solid_ppm(
        paths
            .asset_root(&asset_root)
            .join("textures")
            .join("emissive_cool.ppm"),
        [24, 96, 255, 255],
    );
    write_solid_ppm(
        paths
            .asset_root(&asset_root)
            .join("textures")
            .join("mr_rough_dielectric.ppm"),
        [0, 240, 16, 255],
    );
    write_solid_ppm(
        paths
            .asset_root(&asset_root)
            .join("textures")
            .join("mr_smooth_metallic.ppm"),
        [0, 16, 240, 255],
    );
    write_solid_ppm(
        paths
            .asset_root(&asset_root)
            .join("textures")
            .join("normal_flat.ppm"),
        [128, 128, 255, 255],
    );
    write_solid_ppm(
        paths
            .asset_root(&asset_root)
            .join("textures")
            .join("normal_tilted.ppm"),
        [255, 128, 128, 255],
    );
    write_solid_ppm(
        paths
            .asset_root(&asset_root)
            .join("textures")
            .join("occlusion_open.ppm"),
        [255, 255, 255, 255],
    );
    write_solid_ppm(
        paths
            .asset_root(&asset_root)
            .join("textures")
            .join("occlusion_blocked.ppm"),
        [32, 32, 32, 255],
    );

    write_material_asset_with_textures(
        &asset_root,
        paths
            .asset_root(&asset_root)
            .join("materials")
            .join("base_red_texture.zmaterial"),
        [1.0, 1.0, 1.0, 1.0],
        [0.0, 0.0, 0.0],
        0.0,
        0.5,
        Some("res://textures/base_red.ppm"),
        None,
        None,
        None,
        None,
    );
    write_material_asset_with_textures(
        &asset_root,
        paths
            .asset_root(&asset_root)
            .join("materials")
            .join("base_blue_texture.zmaterial"),
        [1.0, 1.0, 1.0, 1.0],
        [0.0, 0.0, 0.0],
        0.0,
        0.5,
        Some("res://textures/base_blue.ppm"),
        None,
        None,
        None,
        None,
    );
    write_material_asset_with_textures(
        &asset_root,
        paths
            .asset_root(&asset_root)
            .join("materials")
            .join("emissive_warm_texture.zmaterial"),
        [0.0, 0.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
        0.0,
        1.0,
        None,
        None,
        Some("res://textures/emissive_warm.ppm"),
        None,
        None,
    );
    write_material_asset_with_textures(
        &asset_root,
        paths
            .asset_root(&asset_root)
            .join("materials")
            .join("emissive_cool_texture.zmaterial"),
        [0.0, 0.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
        0.0,
        1.0,
        None,
        None,
        Some("res://textures/emissive_cool.ppm"),
        None,
        None,
    );
    write_material_asset_with_textures(
        &asset_root,
        paths
            .asset_root(&asset_root)
            .join("materials")
            .join("rough_dielectric_texture.zmaterial"),
        [1.0, 0.25, 0.1, 1.0],
        [0.0, 0.0, 0.0],
        1.0,
        1.0,
        None,
        Some("res://textures/mr_rough_dielectric.ppm"),
        None,
        None,
        None,
    );
    write_material_asset_with_textures(
        &asset_root,
        paths
            .asset_root(&asset_root)
            .join("materials")
            .join("smooth_metallic_texture.zmaterial"),
        [1.0, 0.25, 0.1, 1.0],
        [0.0, 0.0, 0.0],
        1.0,
        1.0,
        None,
        Some("res://textures/mr_smooth_metallic.ppm"),
        None,
        None,
        None,
    );
    write_material_asset_with_textures(
        &asset_root,
        paths
            .asset_root(&asset_root)
            .join("materials")
            .join("flat_normal_texture.zmaterial"),
        [1.0, 1.0, 1.0, 1.0],
        [0.0, 0.0, 0.0],
        0.0,
        0.35,
        None,
        None,
        None,
        Some("res://textures/normal_flat.ppm"),
        None,
    );
    write_material_asset_with_textures(
        &asset_root,
        paths
            .asset_root(&asset_root)
            .join("materials")
            .join("tilted_normal_texture.zmaterial"),
        [1.0, 1.0, 1.0, 1.0],
        [0.0, 0.0, 0.0],
        0.0,
        0.35,
        None,
        None,
        None,
        Some("res://textures/normal_tilted.ppm"),
        None,
    );
    write_material_asset_with_textures(
        &asset_root,
        paths
            .asset_root(&asset_root)
            .join("materials")
            .join("open_occlusion_texture.zmaterial"),
        [1.0, 1.0, 1.0, 1.0],
        [0.0, 0.0, 0.0],
        0.0,
        1.0,
        None,
        None,
        None,
        None,
        Some("res://textures/occlusion_open.ppm"),
    );
    write_material_asset_with_textures(
        &asset_root,
        paths
            .asset_root(&asset_root)
            .join("materials")
            .join("blocked_occlusion_texture.zmaterial"),
        [1.0, 1.0, 1.0, 1.0],
        [0.0, 0.0, 0.0],
        0.0,
        1.0,
        None,
        None,
        None,
        None,
        Some("res://textures/occlusion_blocked.ppm"),
    );
    write_scene_asset(
        &asset_root,
        paths
            .asset_root(&asset_root)
            .join("scenes")
            .join("main.scene.toml"),
        "res://materials/base_red_texture.zmaterial",
    );

    let asset_manager = open_test_project(&root);
    MaterialTextureCaptureTestAssets {
        base_color_red: resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/base_red_texture.zmaterial",
        ),
        base_color_blue: resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/base_blue_texture.zmaterial",
        ),
        emissive_warm: resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/emissive_warm_texture.zmaterial",
        ),
        emissive_cool: resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/emissive_cool_texture.zmaterial",
        ),
        rough_dielectric: resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/rough_dielectric_texture.zmaterial",
        ),
        smooth_metallic: resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/smooth_metallic_texture.zmaterial",
        ),
        flat_normal: resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/flat_normal_texture.zmaterial",
        ),
        tilted_normal: resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/tilted_normal_texture.zmaterial",
        ),
        open_occlusion: resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/open_occlusion_texture.zmaterial",
        ),
        blocked_occlusion: resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/blocked_occlusion_texture.zmaterial",
        ),
        asset_manager,
        root,
    }
}

pub(super) fn material_visibility_capture_test_assets() -> MaterialVisibilityCaptureTestAssets {
    let root = unique_temp_project_root("hybrid_gi_scene_prepare_visibility_capture");
    let paths = ProjectPaths::from_root(&root).unwrap();
    let manifest = ProjectManifest::new(
        "HybridGiScenePrepareVisibilityCapture",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    );
    paths.ensure_layout(&manifest.asset_roots).unwrap();
    let asset_root = manifest.primary_asset_root().unwrap().clone();
    manifest.save(paths.manifest_path()).unwrap();

    write_triangle_model(
        &asset_root,
        paths
            .asset_root(&asset_root)
            .join("models")
            .join("triangle.model.toml"),
    );
    write_material_asset_with_capture_options(
        &asset_root,
        paths
            .asset_root(&asset_root)
            .join("materials")
            .join("single_sided_white.zmaterial"),
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
        &asset_root,
        paths
            .asset_root(&asset_root)
            .join("materials")
            .join("double_sided_white.zmaterial"),
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
        &asset_root,
        paths
            .asset_root(&asset_root)
            .join("materials")
            .join("opaque_white.zmaterial"),
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
        &asset_root,
        paths
            .asset_root(&asset_root)
            .join("materials")
            .join("masked_cutout_white.zmaterial"),
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
        &asset_root,
        paths
            .asset_root(&asset_root)
            .join("materials")
            .join("blended_white.zmaterial"),
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
        &asset_root,
        paths
            .asset_root(&asset_root)
            .join("scenes")
            .join("main.scene.toml"),
        "res://materials/opaque_white.zmaterial",
    );

    let asset_manager = open_test_project(&root);
    MaterialVisibilityCaptureTestAssets {
        single_sided_white: resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/single_sided_white.zmaterial",
        ),
        double_sided_white: resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/double_sided_white.zmaterial",
        ),
        opaque_white: resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/opaque_white.zmaterial",
        ),
        masked_cutout_white: resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/masked_cutout_white.zmaterial",
        ),
        blended_white: resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/blended_white.zmaterial",
        ),
        asset_manager,
        root,
    }
}

pub(super) fn model_handle(asset_manager: &ProjectAssetManager) -> ResourceHandle<ModelMarker> {
    resource_handle::<ModelMarker>(asset_manager, "res://models/triangle.model.toml")
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
