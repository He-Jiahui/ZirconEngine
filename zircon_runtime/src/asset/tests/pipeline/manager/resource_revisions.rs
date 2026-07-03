use super::*;

#[test]
fn resource_server_reimport_bumps_revision_and_publishes_updated_event() {
    let root = unique_temp_project_root("asset_manager_resource_revision");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_valid_wgsl(paths.assets_root().join("shaders").join("pbr.wgsl"));
    write_checker_png(paths.assets_root().join("textures").join("checker.png"));
    write_triangle_obj(paths.assets_root().join("models").join("triangle.obj"));
    let material_path = paths.assets_root().join("materials").join("grid.zmaterial");
    write_default_material(material_path.clone());
    write_default_scene(paths.assets_root().join("scenes").join("main.scene.toml"));

    let manager = project_asset_manager_with_first_wave_plugin_fixtures();
    manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let resource_changes = manager.subscribe_resource_changes();
    let baseline_revision = manager
        .resource_revision("res://materials/grid.zmaterial")
        .expect("baseline revision");

    let mut material =
        MaterialAsset::from_toml_str(&fs::read_to_string(&material_path).unwrap()).unwrap();
    material.base_color = [0.6, 0.2, 0.9, 1.0];
    fs::write(&material_path, material.to_toml_string().unwrap()).unwrap();

    manager
        .import_asset("res://materials/grid.zmaterial")
        .unwrap();

    let next_status = manager
        .resource_status("res://materials/grid.zmaterial")
        .expect("material resource status");
    assert_eq!(next_status.state, ResourceState::Ready);
    assert!(next_status.revision > baseline_revision);

    let deadline = Instant::now() + Duration::from_secs(5);
    let mut updated = None;
    while Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if let Ok(event) = resource_changes.recv_timeout(remaining.min(Duration::from_millis(250)))
        {
            if event.kind == ResourceEventKind::Updated
                && event
                    .locator
                    .as_ref()
                    .is_some_and(|locator| locator.to_string() == "res://materials/grid.zmaterial")
            {
                updated = Some(event);
                break;
            }
        }
    }

    let updated = updated.expect("updated resource event");
    assert_eq!(updated.revision, next_status.revision);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importing_one_asset_does_not_bump_unrelated_resource_revisions() {
    let root = unique_temp_project_root("asset_manager_unrelated_revision");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_valid_wgsl(paths.assets_root().join("shaders").join("pbr.wgsl"));
    write_checker_png(paths.assets_root().join("textures").join("checker.png"));
    let model_path = paths.assets_root().join("models").join("triangle.obj");
    write_triangle_obj(model_path);
    let material_path = paths.assets_root().join("materials").join("grid.zmaterial");
    write_default_material(material_path.clone());
    write_default_scene(paths.assets_root().join("scenes").join("main.scene.toml"));

    let manager = project_asset_manager_with_first_wave_plugin_fixtures();
    manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();

    let baseline_material_revision = manager
        .resource_revision("res://materials/grid.zmaterial")
        .expect("material revision");
    let baseline_model_revision = manager
        .resource_revision("res://models/triangle.obj")
        .expect("model revision");

    let mut material =
        MaterialAsset::from_toml_str(&fs::read_to_string(&material_path).unwrap()).unwrap();
    material.base_color = [0.1, 0.6, 0.8, 1.0];
    fs::write(&material_path, material.to_toml_string().unwrap()).unwrap();

    manager
        .import_asset("res://materials/grid.zmaterial")
        .unwrap();

    assert!(
        manager
            .resource_revision("res://materials/grid.zmaterial")
            .expect("updated material revision")
            > baseline_material_revision
    );
    assert_eq!(
        manager.resource_revision("res://models/triangle.obj"),
        Some(baseline_model_revision),
        "reimporting one asset must not bump unrelated resource revisions",
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn shader_reimport_exports_updated_revision_for_prewarm_registry() {
    let root = unique_temp_project_root("asset_manager_shader_revision_export");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    let shader_path = paths.assets_root().join("shaders").join("pbr.wgsl");
    write_valid_wgsl(shader_path.clone());
    write_checker_png(paths.assets_root().join("textures").join("checker.png"));
    write_triangle_obj(paths.assets_root().join("models").join("triangle.obj"));
    write_default_material(paths.assets_root().join("materials").join("grid.zmaterial"));
    write_default_scene(paths.assets_root().join("scenes").join("main.scene.toml"));

    let manager = project_asset_manager_with_first_wave_plugin_fixtures();
    manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();
    let baseline_revision = manager
        .resource_revision("res://shaders/pbr.wgsl")
        .expect("baseline shader revision");

    fs::write(
        &shader_path,
        r#"
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4f {
    let x = f32(i32(vertex_index) - 1);
    return vec4f(x, 0.2, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4f {
    return vec4f(0.2, 0.8, 1.0, 1.0);
}
"#,
    )
    .unwrap();

    manager.import_asset("res://shaders/pbr.wgsl").unwrap();

    let shader_status = manager
        .resource_status("res://shaders/pbr.wgsl")
        .expect("updated shader resource status");
    assert_eq!(shader_status.kind, ResourceKind::Shader);
    assert_eq!(shader_status.state, ResourceState::Ready);
    assert!(shader_status.revision > baseline_revision);

    let exported_record = manager
        .resource_manager()
        .ready_records_for_kind(ResourceKind::Shader)
        .into_iter()
        .find(|record| record.primary_locator.to_string() == "res://shaders/pbr.wgsl")
        .expect("edited shader ready record should be exported");
    assert_eq!(exported_record.id, shader_status.id);
    assert_eq!(exported_record.revision, shader_status.revision);
    assert_eq!(exported_record.state, ResourceState::Ready);

    let _ = fs::remove_dir_all(root);
}
