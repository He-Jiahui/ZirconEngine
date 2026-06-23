use super::*;

#[test]
fn recursive_dependency_load_state_walks_nested_resource_dependencies() {
    let manager = ProjectAssetManager::default();
    let resource_manager = manager.resource_manager();
    let texture = record("res://textures/checker.png", ResourceKind::Texture);
    let texture_id = texture.id;
    let texture_handle = manager
        .assets::<TextureAsset>()
        .insert(texture, texture_asset("res://textures/checker.png"))
        .expect("texture handle");
    let mut shader = record("res://shaders/pbr.wgsl", ResourceKind::Shader);
    shader.dependency_ids = vec![texture_id];
    let shader_id = shader.id;
    let _shader_handle = manager
        .assets::<ShaderAsset>()
        .insert(shader, shader_asset("res://shaders/pbr.wgsl"))
        .expect("shader handle");
    let mut material = record("res://materials/grid.zmaterial", ResourceKind::Material);
    material.dependency_ids = vec![shader_id];
    let material_handle = manager
        .assets::<MaterialAsset>()
        .insert(material, material_asset("res://shaders/pbr.wgsl"))
        .expect("material handle");

    assert_eq!(
        manager.recursive_dependency_load_state(material_handle),
        RecursiveDependencyLoadState::Loaded
    );
    assert_eq!(
        manager.dependency_load_state(material_handle),
        DependencyLoadState::Loaded
    );

    resource_manager.start_reload(texture_id, Vec::new());
    assert_eq!(
        manager.dependency_load_state(material_handle),
        DependencyLoadState::Loaded,
        "direct dependency state should not include nested texture dependencies"
    );
    assert_eq!(
        manager.recursive_dependency_load_state(material_handle),
        RecursiveDependencyLoadState::Reloading
    );

    resource_manager.fail_reload(
        texture_id,
        vec![ResourceDiagnostic::error("texture failed")],
    );
    assert_eq!(
        manager.recursive_dependency_load_state(material_handle),
        RecursiveDependencyLoadState::Failed
    );

    let texture_payload = manager
        .assets::<TextureAsset>()
        .acquire(texture_handle)
        .expect("texture payload");
    drop(texture_payload);
    assert_eq!(
        manager.recursive_dependency_load_state(material_handle),
        RecursiveDependencyLoadState::Failed,
        "failed dependencies take precedence over unloaded dependencies"
    );
}

#[test]
fn load_states_separate_root_direct_and_recursive_dependency_state() {
    let manager = ProjectAssetManager::default();
    let resource_manager = manager.resource_manager();
    let texture = record("res://textures/nested.png", ResourceKind::Texture);
    let texture_id = texture.id;
    let texture_handle = manager
        .assets::<TextureAsset>()
        .insert(texture, texture_asset("res://textures/nested.png"))
        .expect("texture handle");
    let mut shader = record("res://shaders/nested.wgsl", ResourceKind::Shader);
    shader.dependency_ids = vec![texture_id];
    let shader_id = shader.id;
    manager
        .assets::<ShaderAsset>()
        .insert(shader, shader_asset("res://shaders/nested.wgsl"))
        .expect("shader handle");
    let mut material = record("res://materials/nested.zmaterial", ResourceKind::Material);
    material.dependency_ids = vec![shader_id];
    let material_handle = manager
        .assets::<MaterialAsset>()
        .insert(material, material_asset("res://shaders/nested.wgsl"))
        .expect("material handle");

    assert_eq!(
        manager.load_states(material_handle),
        AssetLoadStates {
            load_state: AssetLoadState::Loaded,
            dependency_load_state: DependencyLoadState::Loaded,
            recursive_dependency_load_state: RecursiveDependencyLoadState::Loaded,
        }
    );
    assert!(manager.is_loaded(material_handle));
    assert!(manager.is_loaded_with_direct_dependencies(material_handle));
    assert!(manager.is_loaded_with_dependencies(material_handle));

    resource_manager.start_reload(texture_id, Vec::new());
    assert_eq!(
        manager.dependency_load_state(material_handle),
        DependencyLoadState::Loaded,
        "direct dependency stays loaded when only nested dependency reloads"
    );
    assert_eq!(
        manager.recursive_dependency_load_state(material_handle),
        RecursiveDependencyLoadState::Reloading
    );
    assert!(manager.is_loaded_with_direct_dependencies(material_handle));
    assert!(!manager.is_loaded_with_dependencies(material_handle));

    let texture_payload = manager
        .assets::<TextureAsset>()
        .acquire(texture_handle)
        .expect("texture payload");
    drop(texture_payload);
    assert_eq!(
        manager.dependency_load_state(material_handle),
        DependencyLoadState::Loaded,
        "direct dependency aggregation does not walk grandchildren"
    );
}

#[test]
fn readiness_report_exposes_loaded_dependency_rows_and_record_diagnostics() {
    let manager = ProjectAssetManager::default();
    let texture_diagnostic = ResourceDiagnostic::error("texture importer warning");
    let texture = record("res://textures/report.png", ResourceKind::Texture)
        .with_diagnostics(vec![texture_diagnostic.clone()]);
    let texture_id = texture.id;
    manager
        .assets::<TextureAsset>()
        .insert(texture, texture_asset("res://textures/report.png"))
        .expect("texture handle");

    let shader = record("res://shaders/report.wgsl", ResourceKind::Shader);
    let shader_id = shader.id;
    manager
        .assets::<ShaderAsset>()
        .insert(shader, shader_asset("res://shaders/report.wgsl"))
        .expect("shader handle");

    let root_diagnostic = ResourceDiagnostic::error("material shader contract warning");
    let mut material = record("res://materials/report.zmaterial", ResourceKind::Material)
        .with_diagnostics(vec![root_diagnostic.clone()]);
    material.dependency_ids = vec![shader_id, texture_id];
    let material_handle = manager
        .assets::<MaterialAsset>()
        .insert(material, material_asset("res://shaders/report.wgsl"))
        .expect("material handle");

    let report = manager.readiness_report(material_handle);

    assert_eq!(report.load_states.load_state, AssetLoadState::Loaded);
    assert!(report.is_loaded_with_dependencies());
    assert_eq!(report.root.diagnostics, vec![root_diagnostic]);
    assert_eq!(report.dependencies.len(), 2);
    let texture_row = report
        .dependencies
        .iter()
        .find(|row| row.id == texture_id)
        .expect("texture dependency row");
    assert_eq!(texture_row.depth, 1);
    assert!(texture_row.direct);
    assert_eq!(texture_row.load_state, AssetLoadState::Loaded);
    assert_eq!(texture_row.diagnostics, vec![texture_diagnostic]);
}

#[test]
fn readiness_report_and_load_states_roundtrip_for_tooling_snapshots() {
    let manager = ProjectAssetManager::default();
    let texture_diagnostic = ResourceDiagnostic::error("texture importer warning");
    let texture = record(
        "res://textures/report-serializable.png",
        ResourceKind::Texture,
    )
    .with_diagnostics(vec![texture_diagnostic]);
    let texture_id = texture.id;
    manager
        .assets::<TextureAsset>()
        .insert(
            texture,
            texture_asset("res://textures/report-serializable.png"),
        )
        .expect("texture handle");

    let root_diagnostic = ResourceDiagnostic::error("material shader contract warning");
    let mut material = record(
        "res://materials/report-serializable.zmaterial",
        ResourceKind::Material,
    )
    .with_diagnostics(vec![root_diagnostic]);
    material.dependency_ids = vec![texture_id];
    let material_handle = manager
        .assets::<MaterialAsset>()
        .insert(
            material,
            material_asset("res://shaders/report-serializable.wgsl"),
        )
        .expect("material handle");

    let report = manager.readiness_report(material_handle);
    let json = serde_json::to_string(&report).expect("serializable readiness report");
    let decoded: crate::asset::AssetReadinessReport =
        serde_json::from_str(&json).expect("deserializable readiness report");

    assert_eq!(decoded, report);
    assert!(json.contains("\"load_state\":\"loaded\""));
    assert!(json.contains("\"dependency_load_state\":\"loaded\""));
    assert!(json.contains("\"recursive_dependency_load_state\":\"loaded\""));
}

#[test]
fn readiness_report_keeps_shallowest_direct_dependency_row_and_terminates_cycles() {
    let manager = ProjectAssetManager::default();

    let mut texture = record("res://textures/report-cycle.png", ResourceKind::Texture);
    let texture_id = texture.id;
    let mut shader = record("res://shaders/report-cycle.wgsl", ResourceKind::Shader);
    let shader_id = shader.id;
    texture.dependency_ids = vec![shader_id];
    shader.dependency_ids = vec![texture_id];

    manager
        .assets::<TextureAsset>()
        .insert(texture, texture_asset("res://textures/report-cycle.png"))
        .expect("texture handle");
    manager
        .assets::<ShaderAsset>()
        .insert(shader, shader_asset("res://shaders/report-cycle.wgsl"))
        .expect("shader handle");

    let mut material = record(
        "res://materials/report-cycle.zmaterial",
        ResourceKind::Material,
    );
    material.dependency_ids = vec![shader_id, texture_id];
    let material_handle = manager
        .assets::<MaterialAsset>()
        .insert(material, material_asset("res://shaders/report-cycle.wgsl"))
        .expect("material handle");

    let report = manager.readiness_report(material_handle);

    assert_eq!(report.dependencies.len(), 2);
    let shader_row = dependency_row(&report.dependencies, shader_id);
    assert_eq!(shader_row.depth, 1);
    assert!(shader_row.direct);
    let texture_row = dependency_row(&report.dependencies, texture_id);
    assert_eq!(
        texture_row.depth, 1,
        "direct edge must win over nested cycle path"
    );
    assert!(texture_row.direct);
}

#[test]
fn dependency_load_state_reports_first_level_dependency_changes() {
    let manager = ProjectAssetManager::default();
    let resource_manager = manager.resource_manager();
    let texture = record("res://textures/checker.png", ResourceKind::Texture);
    let texture_id = texture.id;
    let _texture_handle = manager
        .assets::<TextureAsset>()
        .insert(texture, texture_asset("res://textures/checker.png"))
        .expect("texture handle");
    let mut shader = record("res://shaders/pbr.wgsl", ResourceKind::Shader);
    shader.dependency_ids = vec![texture_id];
    let shader_id = shader.id;
    let _shader_handle = manager
        .assets::<ShaderAsset>()
        .insert(shader, shader_asset("res://shaders/pbr.wgsl"))
        .expect("shader handle");
    let mut material = record("res://materials/grid.zmaterial", ResourceKind::Material);
    material.dependency_ids = vec![shader_id];
    let material_handle = manager
        .assets::<MaterialAsset>()
        .insert(material, material_asset("res://shaders/pbr.wgsl"))
        .expect("material handle");

    resource_manager.start_reload(shader_id, Vec::new());

    assert_eq!(
        manager.dependency_load_state(material_handle),
        DependencyLoadState::Reloading
    );
}
