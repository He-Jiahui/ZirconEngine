use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use image::{ImageBuffer, ImageFormat, Rgba};
use zircon_asset::{
    AlphaMode, AssetReference, AssetUri, MaterialAsset, ProjectManager, ProjectManifest,
    ProjectPaths, SceneAsset, SceneCameraAsset, SceneEntityAsset, SceneMeshInstanceAsset,
    SceneMobilityAsset, TransformAsset,
};
use zircon_resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};

use crate::default_render_layer_mask;

pub(super) fn model_handle(label: &str) -> ResourceHandle<ModelMarker> {
    ResourceHandle::new(ResourceId::from_stable_label(label))
}

pub(super) fn material_handle(label: &str) -> ResourceHandle<MaterialMarker> {
    ResourceHandle::new(ResourceId::from_stable_label(label))
}

fn asset_reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}

pub(super) fn project_model_handle(
    project: &ProjectManager,
    uri: &str,
) -> ResourceHandle<ModelMarker> {
    let uri = AssetUri::parse(uri).unwrap();
    ResourceHandle::new(project.asset_id_for_uri(&uri).unwrap())
}

pub(super) fn project_material_handle(
    project: &ProjectManager,
    uri: &str,
) -> ResourceHandle<MaterialMarker> {
    let uri = AssetUri::parse(uri).unwrap();
    ResourceHandle::new(project.asset_id_for_uri(&uri).unwrap())
}

pub(super) fn unique_temp_project_root(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("zircon_scene_{label}_{unique}"))
}

pub(super) fn create_test_project(root: &PathBuf) -> ProjectManager {
    let paths = ProjectPaths::from_root(root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "SceneSandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_valid_wgsl(paths.assets_root().join("shaders").join("pbr.wgsl"));
    write_checker_png(paths.assets_root().join("textures").join("checker.png"));
    write_triangle_obj(paths.assets_root().join("models").join("triangle.obj"));
    write_default_material(
        paths
            .assets_root()
            .join("materials")
            .join("grid.material.toml"),
    );
    write_default_scene(paths.assets_root().join("scenes").join("main.scene.toml"));

    let mut project = ProjectManager::open(root).unwrap();
    project.scan_and_import().unwrap();
    project
}

fn write_valid_wgsl(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(
        path,
        r#"
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4f {
    let x = f32(i32(vertex_index) - 1);
    return vec4f(x, 0.0, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4f {
    return vec4f(1.0, 0.4, 0.2, 1.0);
}
"#,
    )
    .unwrap();
}

fn write_checker_png(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    ImageBuffer::<Rgba<u8>, _>::from_fn(2, 2, |x, y| {
        if (x + y) % 2 == 0 {
            Rgba([255, 255, 255, 255])
        } else {
            Rgba([0, 0, 0, 255])
        }
    })
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

fn write_default_material(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let material = MaterialAsset {
        name: Some("Grid".to_string()),
        shader: asset_reference("res://shaders/pbr.wgsl"),
        base_color: [0.8, 0.8, 0.8, 1.0],
        base_color_texture: Some(asset_reference("res://textures/checker.png")),
        normal_texture: None,
        metallic: 0.1,
        roughness: 0.8,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
    };
    fs::write(path, material.to_toml_string().unwrap()).unwrap();
}

fn write_default_scene(path: PathBuf) {
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
                render_layer_mask: default_render_layer_mask(),
                mobility: SceneMobilityAsset::Dynamic,
                camera: Some(SceneCameraAsset {
                    fov_y_radians: 1.0471976,
                    z_near: 0.1,
                    z_far: 200.0,
                }),
                mesh: None,
                directional_light: None,
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
                render_layer_mask: default_render_layer_mask(),
                mobility: SceneMobilityAsset::Dynamic,
                camera: None,
                mesh: Some(SceneMeshInstanceAsset {
                    model: asset_reference("res://models/triangle.obj"),
                    material: asset_reference("res://materials/grid.material.toml"),
                }),
                directional_light: None,
            },
        ],
    };
    fs::write(path, scene.to_toml_string().unwrap()).unwrap();
}
