use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use crate::asset::AssetUri;
use zircon_runtime_interface::project::RelPath;

use super::{
    write_checker_png, write_default_animation_clip, write_default_animation_graph,
    write_default_animation_sequence, write_default_animation_skeleton,
    write_default_animation_state_machine, write_default_material, write_default_physics_material,
    write_default_scene, write_triangle_obj, write_triangle_zmesh, write_valid_wgsl,
};

pub(in crate::scene::tests) fn unique_temp_project_root(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("zircon_scene_{label}_{unique}"))
}

pub(in crate::scene::tests) fn create_test_project(root: &Path) -> ProjectManager {
    let paths = ProjectPaths::from_root(root).unwrap();
    paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
    ProjectManifest::new(
        "SceneSandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let assets = paths.asset_root(&RelPath::project_assets());
    write_valid_wgsl(assets.join("shaders/pbr.wgsl"));
    write_checker_png(assets.join("textures/checker.png"));
    write_triangle_obj(assets.join("models/triangle.obj"));
    write_triangle_zmesh(assets.join("meshes/triangle.zmesh"));
    write_default_physics_material(assets.join("physics/default.physics_material.toml"));
    write_default_animation_skeleton(assets.join("animation/hero.skeleton.zranim"));
    write_default_animation_clip(assets.join("animation/hero.clip.zranim"));
    write_default_animation_sequence(assets.join("animation/hero.sequence.zranim"));
    write_default_animation_graph(assets.join("animation/hero.graph.zranim"));
    write_default_animation_state_machine(assets.join("animation/hero.state_machine.zranim"));

    let mut project = ProjectManager::open(root).unwrap();
    project
        .register_first_wave_plugin_fixture_importers_for_test()
        .unwrap();
    project.scan_and_import().unwrap();

    write_default_material(assets.join("materials/grid.zmaterial"), &project);
    project.scan_and_import().unwrap();

    write_default_scene(assets.join("scenes/main.scene.toml"), &project);
    project.scan_and_import().unwrap();
    project
}
