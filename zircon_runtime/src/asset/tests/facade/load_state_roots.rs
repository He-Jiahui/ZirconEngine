use super::*;

#[test]
fn asset_load_state_maps_resource_state_runtime_state_and_payload_residency() {
    let manager = ResourceManager::new();
    let textures = Assets::<TextureAsset>::new(manager.clone());
    let missing = Handle::<TextureAsset>::new(ResourceId::new());
    assert_eq!(textures.load_state(missing), AssetLoadState::NotLoaded);

    let pending = record("res://textures/pending.png", ResourceKind::Texture);
    let pending_handle = Handle::<TextureAsset>::new(pending.id);
    manager.register_record(pending);
    assert_eq!(textures.load_state(pending_handle), AssetLoadState::Loading);

    let ready = record("res://textures/ready.png", ResourceKind::Texture);
    let ready_handle = manager
        .register_ready(ready, texture_asset("res://textures/ready.png"))
        .typed::<TextureMarker>()
        .map(Handle::<TextureAsset>::from_resource_handle)
        .expect("ready texture handle");
    assert_eq!(textures.load_state(ready_handle), AssetLoadState::Loaded);

    let lease = textures.acquire(ready_handle).expect("lease");
    drop(lease);
    assert_eq!(textures.load_state(ready_handle), AssetLoadState::NotLoaded);

    let reloading = record("res://textures/reloading.png", ResourceKind::Texture);
    let reloading_id = reloading.id;
    let reloading_handle = manager
        .register_ready(reloading, texture_asset("res://textures/reloading.png"))
        .typed::<TextureMarker>()
        .map(Handle::<TextureAsset>::from_resource_handle)
        .expect("reloading texture handle");
    manager.start_reload(reloading_id, vec![ResourceDiagnostic::error("reload")]);
    assert_eq!(
        textures.load_state(reloading_handle),
        AssetLoadState::Reloading
    );
    assert!(textures.load_state(reloading_handle).is_loading_class());

    manager.fail_reload(reloading_id, vec![ResourceDiagnostic::error("failed")]);
    assert_eq!(
        textures.load_state(reloading_handle),
        AssetLoadState::Failed
    );
    assert_eq!(textures.get(reloading_handle).unwrap().width, 1);
}

#[test]
fn asset_load_state_requires_typed_payload_not_just_matching_record_kind() {
    let manager = ResourceManager::new();
    let textures = Assets::<TextureAsset>::new(manager.clone());
    let handle = manager
        .register_ready(
            record("res://textures/wrong-payload.png", ResourceKind::Texture),
            texture_asset("res://textures/wrong-payload.png"),
        )
        .typed::<TextureMarker>()
        .map(Handle::<TextureAsset>::from_resource_handle)
        .expect("texture handle");

    assert!(manager.store_payload(
        handle.id(),
        shader_asset("res://shaders/wrong-payload.wgsl")
    ));

    assert!(textures.get(handle).is_none());
    assert_eq!(textures.load_state(handle), AssetLoadState::NotLoaded);
}

#[test]
fn load_states_for_missing_wrong_kind_and_non_resident_roots_do_not_restore_payloads() {
    let manager = ProjectAssetManager::default();
    let resource_manager = manager.resource_manager();
    let missing = Handle::<TextureAsset>::new(ResourceId::new());

    assert_eq!(
        manager.load_states(missing),
        AssetLoadStates {
            load_state: AssetLoadState::NotLoaded,
            dependency_load_state: DependencyLoadState::NotLoaded,
            recursive_dependency_load_state: RecursiveDependencyLoadState::NotLoaded,
        }
    );

    let material_record = record(
        "res://materials/wrong-kind.zmaterial",
        ResourceKind::Material,
    );
    let wrong_kind = Handle::<TextureAsset>::new(material_record.id);
    resource_manager.register_ready(
        material_record,
        material_asset("res://shaders/wrong-kind.wgsl"),
    );
    assert_eq!(
        manager.load_states(wrong_kind),
        AssetLoadStates {
            load_state: AssetLoadState::NotLoaded,
            dependency_load_state: DependencyLoadState::NotLoaded,
            recursive_dependency_load_state: RecursiveDependencyLoadState::NotLoaded,
        }
    );

    let texture_record = record("res://textures/non-resident.png", ResourceKind::Texture);
    let non_resident = manager
        .assets::<TextureAsset>()
        .insert(
            texture_record,
            texture_asset("res://textures/non-resident.png"),
        )
        .expect("texture handle");
    let lease = manager
        .assets::<TextureAsset>()
        .acquire(non_resident)
        .expect("resident lease");
    drop(lease);

    assert_eq!(manager.load_state(non_resident), AssetLoadState::NotLoaded);
    assert_eq!(
        manager.load_states(non_resident),
        AssetLoadStates {
            load_state: AssetLoadState::NotLoaded,
            dependency_load_state: DependencyLoadState::Loaded,
            recursive_dependency_load_state: RecursiveDependencyLoadState::NotLoaded,
        }
    );
    assert!(!manager.is_loaded(non_resident));
    assert!(!manager.is_loaded_with_direct_dependencies(non_resident));
    assert!(!manager.is_loaded_with_dependencies(non_resident));
    assert!(resource_manager.get_untyped(non_resident.id()).is_none());
}

#[test]
fn readiness_report_marks_missing_and_wrong_kind_roots_without_restoring_payloads() {
    let manager = ProjectAssetManager::default();
    let resource_manager = manager.resource_manager();
    let missing = Handle::<TextureAsset>::new(ResourceId::new());

    let missing_report = manager.readiness_report(missing);
    assert_eq!(missing_report.root.id, missing.id());
    assert_eq!(missing_report.root.load_state, AssetLoadState::NotLoaded);
    assert_eq!(missing_report.load_states, manager.load_states(missing));
    assert!(missing_report.dependencies.is_empty());
    assert!(diagnostic_messages(&missing_report.root.diagnostics)
        .iter()
        .any(|message| message.contains("missing asset record")));

    let material_record = record(
        "res://materials/report-wrong-kind.zmaterial",
        ResourceKind::Material,
    );
    let wrong_kind = Handle::<TextureAsset>::new(material_record.id);
    resource_manager.register_ready(
        material_record,
        material_asset("res://shaders/report-wrong-kind.wgsl"),
    );
    let wrong_kind_report = manager.readiness_report(wrong_kind);

    assert_eq!(wrong_kind_report.root.kind, Some(ResourceKind::Material));
    assert_eq!(wrong_kind_report.root.load_state, AssetLoadState::NotLoaded);
    assert_eq!(
        wrong_kind_report.load_states,
        manager.load_states(wrong_kind)
    );
    assert!(wrong_kind_report.dependencies.is_empty());
    assert!(diagnostic_messages(&wrong_kind_report.root.diagnostics)
        .iter()
        .any(|message| message.contains("not Texture")));

    let texture_record = record(
        "res://textures/report-non-resident.png",
        ResourceKind::Texture,
    );
    let non_resident = manager
        .assets::<TextureAsset>()
        .insert(
            texture_record,
            texture_asset("res://textures/report-non-resident.png"),
        )
        .expect("texture handle");
    let lease = manager
        .assets::<TextureAsset>()
        .acquire(non_resident)
        .expect("resident lease");
    drop(lease);

    let non_resident_report = manager.readiness_report(non_resident);
    assert_eq!(
        non_resident_report.root.load_state,
        AssetLoadState::NotLoaded
    );
    assert_eq!(
        non_resident_report.load_states,
        manager.load_states(non_resident)
    );
    assert!(resource_manager.get_untyped(non_resident.id()).is_none());
}
