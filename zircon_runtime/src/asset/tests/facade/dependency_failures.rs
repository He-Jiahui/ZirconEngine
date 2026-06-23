use super::*;

#[test]
fn recursive_dependency_load_state_marks_missing_dependency_as_failed() {
    let manager = ProjectAssetManager::default();
    let missing_id = ResourceId::from_stable_label("missing dependency");
    let mut material = record("res://materials/missing.zmaterial", ResourceKind::Material);
    material.dependency_ids = vec![missing_id];
    let material_handle = manager
        .assets::<MaterialAsset>()
        .insert(
            material,
            material_asset("res://shaders/missing-dependency.wgsl"),
        )
        .expect("material handle");

    assert_eq!(
        manager.recursive_dependency_load_state(material_handle),
        RecursiveDependencyLoadState::Failed
    );
}

#[test]
fn readiness_report_marks_missing_dependency_records_as_failed_rows() {
    let manager = ProjectAssetManager::default();
    let missing_id = ResourceId::from_stable_label("readiness missing dependency");
    let mut material = record(
        "res://materials/report-missing.zmaterial",
        ResourceKind::Material,
    );
    material.dependency_ids = vec![missing_id];
    let material_handle = manager
        .assets::<MaterialAsset>()
        .insert(
            material,
            material_asset("res://shaders/report-missing-dependency.wgsl"),
        )
        .expect("material handle");

    let report = manager.readiness_report(material_handle);

    assert_eq!(
        report.load_states.dependency_load_state,
        DependencyLoadState::Failed
    );
    assert_eq!(
        report.load_states.recursive_dependency_load_state,
        RecursiveDependencyLoadState::Failed
    );
    assert_eq!(report.dependencies.len(), 1);
    let row = &report.dependencies[0];
    assert_eq!(row.id, missing_id);
    assert_eq!(row.locator, None);
    assert_eq!(row.kind, None);
    assert_eq!(row.revision, None);
    assert_eq!(row.depth, 1);
    assert!(row.direct);
    assert_eq!(row.load_state, AssetLoadState::Failed);
    assert!(diagnostic_messages(&row.diagnostics)
        .iter()
        .any(|message| message.contains("missing asset dependency record")));
}

#[test]
fn dependency_load_state_applies_direct_precedence_and_missing_records() {
    let manager = ProjectAssetManager::default();
    let resource_manager = manager.resource_manager();
    let loaded_texture = record("res://textures/direct-loaded.png", ResourceKind::Texture);
    let loaded_id = loaded_texture.id;
    manager
        .assets::<TextureAsset>()
        .insert(
            loaded_texture,
            texture_asset("res://textures/direct-loaded.png"),
        )
        .expect("loaded texture handle");
    let non_resident_texture = record(
        "res://textures/direct-non-resident.png",
        ResourceKind::Texture,
    );
    let non_resident_id = non_resident_texture.id;
    let non_resident_handle = manager
        .assets::<TextureAsset>()
        .insert(
            non_resident_texture,
            texture_asset("res://textures/direct-non-resident.png"),
        )
        .expect("non-resident texture handle");
    let non_resident_lease = manager
        .assets::<TextureAsset>()
        .acquire(non_resident_handle)
        .expect("non-resident texture lease");
    drop(non_resident_lease);
    let pending = record("res://textures/direct-pending.png", ResourceKind::Texture)
        .with_state(ResourceState::Pending);
    let pending_id = pending.id;
    resource_manager.register_record(pending);
    let reloading = record("res://textures/direct-reloading.png", ResourceKind::Texture);
    let reloading_id = reloading.id;
    manager
        .assets::<TextureAsset>()
        .insert(
            reloading,
            texture_asset("res://textures/direct-reloading.png"),
        )
        .expect("reloading texture handle");
    resource_manager.start_reload(reloading_id, Vec::new());
    let missing_id = ResourceId::from_stable_label("direct missing dependency");
    let mut material = record("res://materials/direct.zmaterial", ResourceKind::Material);
    material.dependency_ids = vec![loaded_id, pending_id, reloading_id, missing_id];
    let material_handle = manager
        .assets::<MaterialAsset>()
        .insert(material, material_asset("res://shaders/direct.wgsl"))
        .expect("material handle");

    assert_eq!(
        manager.dependency_load_state(material_handle),
        DependencyLoadState::Failed,
        "missing direct dependencies outrank loading and reloading states"
    );

    let mut material_without_missing = record(
        "res://materials/direct-no-missing.zmaterial",
        ResourceKind::Material,
    );
    material_without_missing.dependency_ids = vec![loaded_id, pending_id, reloading_id];
    let material_without_missing_handle = manager
        .assets::<MaterialAsset>()
        .insert(
            material_without_missing,
            material_asset("res://shaders/direct.wgsl"),
        )
        .expect("material handle");

    assert_eq!(
        manager.dependency_load_state(material_without_missing_handle),
        DependencyLoadState::Reloading,
        "reloading outranks pending/loading when no dependency failed"
    );

    let mut material_with_loading = record(
        "res://materials/direct-loading.zmaterial",
        ResourceKind::Material,
    );
    material_with_loading.dependency_ids = vec![loaded_id, non_resident_id, pending_id];
    let material_with_loading_handle = manager
        .assets::<MaterialAsset>()
        .insert(
            material_with_loading,
            material_asset("res://shaders/direct.wgsl"),
        )
        .expect("material handle");

    assert_eq!(
        manager.dependency_load_state(material_with_loading_handle),
        DependencyLoadState::Loading,
        "loading outranks not-loaded and loaded direct dependencies"
    );

    let mut material_with_not_loaded = record(
        "res://materials/direct-not-loaded.zmaterial",
        ResourceKind::Material,
    );
    material_with_not_loaded.dependency_ids = vec![loaded_id, non_resident_id];
    let material_with_not_loaded_handle = manager
        .assets::<MaterialAsset>()
        .insert(
            material_with_not_loaded,
            material_asset("res://shaders/direct.wgsl"),
        )
        .expect("material handle");

    assert_eq!(
        manager.dependency_load_state(material_with_not_loaded_handle),
        DependencyLoadState::NotLoaded,
        "not-loaded direct dependencies outrank loaded dependencies"
    );
}
