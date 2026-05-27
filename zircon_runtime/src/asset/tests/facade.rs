use std::time::Duration;

use crate::asset::{
    AlphaMode, AssetDependencyReadiness, AssetEvent, AssetLoadState, AssetLoadStates,
    AssetReference, AssetUri, Assets, DependencyLoadState, Handle, MaterialAsset, MeshAsset,
    ProjectAssetManager, RecursiveDependencyLoadState, ShaderAsset, ShaderEntryPointAsset,
    ShaderSourceLanguage, TextureAsset, UiLayoutAsset, UiV2ViewAsset,
};
use crate::core::resource::{
    ResourceDiagnostic, ResourceHandle, ResourceId, ResourceKind, ResourceManager, ResourceRecord,
    ResourceState, TextureMarker, UntypedResourceHandle,
};

fn locator(value: &str) -> AssetUri {
    AssetUri::parse(value).expect("valid asset locator")
}

fn record(locator_text: &str, kind: ResourceKind) -> ResourceRecord {
    let locator = locator(locator_text);
    ResourceRecord::new(ResourceId::from_locator(&locator), kind, locator)
}

fn texture_asset(uri: &str) -> TextureAsset {
    TextureAsset::new_rgba8(locator(uri), 1, 1, vec![255, 0, 0, 255])
}

fn shader_asset(uri: &str) -> ShaderAsset {
    ShaderAsset {
        uri: locator(uri),
        source_language: ShaderSourceLanguage::Wgsl,
        source: "@fragment fn fs_main() {}".to_string(),
        wgsl_source: "@fragment fn fs_main() {}".to_string(),
        import_path: None,
        entry_points: vec![ShaderEntryPointAsset {
            name: "fs_main".to_string(),
            stage: "fragment".to_string(),
        }],
        dependencies: Vec::new(),
        source_files: Vec::new(),
        imports: Vec::new(),
        shader_defs: Vec::new(),
        property_schema: Vec::new(),
        texture_slots: Vec::new(),
        editor: Default::default(),
        pipeline_layout: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}

fn material_asset(shader_uri: &str) -> MaterialAsset {
    MaterialAsset {
        name: Some("Grid".to_string()),
        shader: AssetReference::from_locator(locator(shader_uri)),
        base_color: [0.8, 0.8, 0.8, 1.0],
        base_color_texture: None,
        normal_texture: None,
        metallic: 0.0,
        roughness: 1.0,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}

fn diagnostic_messages(diagnostics: &[ResourceDiagnostic]) -> Vec<&str> {
    diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect()
}

fn dependency_row(rows: &[AssetDependencyReadiness], id: ResourceId) -> &AssetDependencyReadiness {
    rows.iter()
        .find(|row| row.id == id)
        .expect("dependency row")
}

#[test]
fn typed_handle_roundtrips_and_rejects_kind_mismatch() {
    let id = ResourceId::from_stable_label("res://textures/checker.png");
    let handle = Handle::<TextureAsset>::new(id);
    let raw: ResourceHandle<TextureMarker> = handle.into();
    let untyped: UntypedResourceHandle = handle.into();
    let mesh_id = ResourceId::from_stable_label("res://meshes/triangle.zmesh");
    let mesh_untyped: UntypedResourceHandle = Handle::<MeshAsset>::new(mesh_id).into();

    assert_eq!(handle.id(), id);
    assert_eq!(raw.id(), id);
    assert_eq!(untyped.kind(), ResourceKind::Texture);
    assert_eq!(mesh_untyped.kind(), ResourceKind::Mesh);
    assert_eq!(Handle::<TextureAsset>::try_from(untyped).unwrap().id(), id);
    assert!(Handle::<ShaderAsset>::try_from(untyped).is_err());
}

#[test]
fn assets_get_acquire_release_and_kind_filtering_use_resource_manager_storage() {
    let manager = ResourceManager::new();
    let texture_record = record("res://textures/checker.png", ResourceKind::Texture);
    let material_record = record("res://materials/grid.zmaterial", ResourceKind::Material);
    let texture_id = texture_record.id;
    let material_id = material_record.id;
    let texture_handle = manager
        .register_ready(texture_record, texture_asset("res://textures/checker.png"))
        .typed::<TextureMarker>()
        .map(Handle::<TextureAsset>::from_resource_handle)
        .expect("texture handle");
    manager.register_ready(material_record, material_asset("builtin://shader/pbr.wgsl"));

    let textures = Assets::<TextureAsset>::new(manager.clone());
    let wrong_texture_handle = Handle::<TextureAsset>::new(material_id);

    assert!(textures.contains(texture_handle));
    assert!(!textures.contains(wrong_texture_handle));
    assert_eq!(textures.get(texture_handle).unwrap().width, 1);
    assert!(textures.get(wrong_texture_handle).is_none());

    let lease = textures.acquire(texture_handle).expect("texture lease");
    assert_eq!(lease.height, 1);
    assert_eq!(manager.ref_count(texture_id), Some(1));
    drop(lease);
    assert_eq!(manager.ref_count(texture_id), Some(0));
    assert!(textures.get(texture_handle).is_none());
}

#[test]
fn typed_asset_events_filter_by_asset_kind_including_removed_events() {
    let manager = ResourceManager::new();
    let texture_events = Assets::<TextureAsset>::new(manager.clone()).subscribe_events();
    let texture_record = record("res://textures/checker.png", ResourceKind::Texture);
    let shader_record = record("res://shaders/pbr.wgsl", ResourceKind::Shader);
    let texture_locator = texture_record.primary_locator.clone();
    let shader_locator = shader_record.primary_locator.clone();
    let texture_id = texture_record.id;

    manager.register_ready(texture_record, texture_asset("res://textures/checker.png"));
    manager.register_ready(shader_record, shader_asset("res://shaders/pbr.wgsl"));
    manager.remove_by_locator(&shader_locator);
    manager.remove_by_locator(&texture_locator);

    let added = texture_events
        .recv_timeout(Duration::from_secs(1))
        .expect("texture added event");
    assert!(matches!(added, AssetEvent::Added { .. }));
    assert_eq!(added.handle().id(), texture_id);

    let removed = texture_events
        .recv_timeout(Duration::from_secs(1))
        .expect("texture removed event");
    assert!(matches!(removed, AssetEvent::Removed { .. }));
    assert_eq!(removed.handle().id(), texture_id);
    assert!(
        texture_events.try_recv().is_err(),
        "shader events must not leak into texture receiver"
    );
}

#[test]
fn typed_asset_events_preserve_rename_reload_and_remove_order() {
    let manager = ResourceManager::new();
    let texture_events = Assets::<TextureAsset>::new(manager.clone()).subscribe_events();
    let original_locator = locator("res://textures/order.png");
    let renamed_locator = locator("res://textures/order-renamed.png");
    let texture_record = record("res://textures/order.png", ResourceKind::Texture);
    let texture_id = texture_record.id;

    manager.register_ready(texture_record, texture_asset("res://textures/order.png"));
    manager
        .rename(&original_locator, renamed_locator.clone())
        .expect("rename texture");
    manager.start_reload(texture_id, Vec::new());
    manager.fail_reload(texture_id, vec![ResourceDiagnostic::error("reload failed")]);
    manager.remove_by_locator(&renamed_locator);

    let added = texture_events
        .recv_timeout(Duration::from_secs(1))
        .expect("added event");
    assert!(matches!(added, AssetEvent::Added { .. }));

    let renamed = texture_events
        .recv_timeout(Duration::from_secs(1))
        .expect("renamed event");
    match renamed {
        AssetEvent::Renamed {
            handle,
            locator,
            previous_locator,
            ..
        } => {
            assert_eq!(handle.id(), texture_id);
            assert_eq!(locator, Some(renamed_locator.clone()));
            assert_eq!(previous_locator, Some(original_locator));
        }
        other => panic!("expected renamed event, got {other:?}"),
    }

    let modified = texture_events
        .recv_timeout(Duration::from_secs(1))
        .expect("reload modified event");
    assert!(matches!(modified, AssetEvent::Modified { .. }));

    let failed = texture_events
        .recv_timeout(Duration::from_secs(1))
        .expect("reload failed event");
    assert!(matches!(failed, AssetEvent::ReloadFailed { .. }));

    let removed = texture_events
        .recv_timeout(Duration::from_secs(1))
        .expect("removed event");
    assert!(matches!(removed, AssetEvent::Removed { .. }));
    assert!(texture_events.try_recv().is_err());
}

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

#[test]
fn assets_insert_remove_and_project_manager_helpers_use_typed_facade() {
    let manager = ProjectAssetManager::default();
    let resource_manager = manager.resource_manager();
    let texture_locator = locator("res://textures/inserted.png");
    let texture_record = record("res://textures/inserted.png", ResourceKind::Texture);
    let texture_id = texture_record.id;
    let texture_events = manager.subscribe_asset_events::<TextureAsset>();

    let handle = manager
        .assets::<TextureAsset>()
        .insert(texture_record, texture_asset("res://textures/inserted.png"))
        .expect("inserted texture handle");

    assert_eq!(handle.id(), texture_id);
    assert_eq!(
        manager
            .handle::<TextureAsset>(&texture_locator)
            .unwrap()
            .id(),
        texture_id
    );
    assert_eq!(
        manager.recursive_dependency_load_state(handle),
        RecursiveDependencyLoadState::Loaded
    );

    let added = texture_events
        .recv_timeout(Duration::from_secs(1))
        .expect("texture added event");
    assert!(matches!(added, AssetEvent::Added { .. }));
    assert_eq!(added.handle().id(), texture_id);

    assert!(manager
        .assets::<ShaderAsset>()
        .remove_by_locator(&texture_locator)
        .is_none());
    assert_eq!(
        resource_manager.registry().get(texture_id).unwrap().kind,
        ResourceKind::Texture
    );

    let removed = manager
        .assets::<TextureAsset>()
        .remove_by_locator(&texture_locator)
        .expect("removed texture record");
    assert_eq!(removed.id, texture_id);
    assert!(resource_manager.registry().get(texture_id).is_none());
}

#[test]
fn project_asset_manager_load_returns_typed_handle_and_state() {
    let manager = ProjectAssetManager::default();
    let resource_manager = manager.resource_manager();
    let texture_record = record("res://textures/checker.png", ResourceKind::Texture);
    let texture_locator = texture_record.primary_locator.clone();
    let texture_id = texture_record.id;
    resource_manager.register_ready(texture_record, texture_asset("res://textures/checker.png"));

    let handle = manager
        .load::<TextureAsset>(&texture_locator)
        .expect("typed texture load");

    assert_eq!(handle.id(), texture_id);
    assert_eq!(
        manager.assets::<TextureAsset>().get(handle).unwrap().width,
        1
    );
    assert_eq!(manager.load_state(handle), AssetLoadState::Loaded);
    assert!(manager.load::<ShaderAsset>(&texture_locator).is_err());

    let pending_record = ResourceRecord::new(
        ResourceId::from_locator(&locator("res://materials/pending.zmaterial")),
        ResourceKind::Material,
        locator("res://materials/pending.zmaterial"),
    )
    .with_state(ResourceState::Pending);
    let pending_id = pending_record.id;
    resource_manager.register_record(pending_record);
    assert_eq!(
        manager.asset_load_state_by_id::<MaterialAsset>(pending_id),
        AssetLoadState::Loading
    );
}

#[test]
fn project_asset_manager_load_accepts_v2_ui_payload_under_ui_layout_kind() {
    let manager = ProjectAssetManager::default();
    let resource_manager = manager.resource_manager();
    let ui_record = record("res://ui/panel.v2.ui.toml", ResourceKind::UiLayout);
    let ui_locator = ui_record.primary_locator.clone();
    let ui_id = ui_record.id;
    resource_manager.register_ready(ui_record, ui_v2_view_asset());

    let handle = manager
        .load::<UiV2ViewAsset>(&ui_locator)
        .expect("typed v2 ui view load");

    assert_eq!(handle.id(), ui_id);
    assert_eq!(
        manager
            .assets::<UiV2ViewAsset>()
            .get(handle)
            .unwrap()
            .document
            .asset
            .id,
        "runtime.ui.panel"
    );
    assert!(manager.load::<UiLayoutAsset>(&ui_locator).is_err());
}

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

fn ui_v2_view_asset() -> UiV2ViewAsset {
    UiV2ViewAsset::from_toml_str(
        r#"
[asset]
kind = "view"
id = "runtime.ui.panel"
version = 2

[root]
node = "root"

[nodes.root]
component = "Text"
control_id = "PanelRoot"
props = { text = "Panel" }
"#,
    )
    .expect("valid ui v2 view asset")
}

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
