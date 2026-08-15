use std::sync::Arc;

use super::*;

#[test]
fn readiness_generation_reuses_stable_snapshot_and_updates_only_reverse_closure() {
    let manager = ProjectAssetManager::default();
    let resources = manager.resource_manager();

    let texture = record(
        "res://textures/readiness-generation.png",
        ResourceKind::Texture,
    );
    let texture_id = texture.id;
    manager
        .assets::<TextureAsset>()
        .insert(
            texture,
            texture_asset("res://textures/readiness-generation.png"),
        )
        .unwrap();

    let mut shader = record(
        "res://shaders/readiness-generation.wgsl",
        ResourceKind::Shader,
    );
    shader.dependency_ids = vec![texture_id];
    let shader_id = shader.id;
    manager
        .assets::<ShaderAsset>()
        .insert(
            shader,
            shader_asset("res://shaders/readiness-generation.wgsl"),
        )
        .unwrap();

    let mut material = record(
        "res://materials/readiness-generation.zmaterial",
        ResourceKind::Material,
    );
    material.dependency_ids = vec![shader_id];
    let material_id = material.id;
    let material_handle = manager
        .assets::<MaterialAsset>()
        .insert(
            material,
            material_asset("res://shaders/readiness-generation.wgsl"),
        )
        .unwrap();

    let published = resources.readiness_generation();
    let dependency_revision = published.dependency_revision(material_id).unwrap();
    assert!(manager
        .load_states(material_handle)
        .is_loaded_with_dependencies());
    assert!(Arc::ptr_eq(&published, &resources.readiness_generation()));

    resources.start_reload(texture_id, Vec::new()).unwrap();

    let reloaded = resources.readiness_generation();
    assert!(!Arc::ptr_eq(&published, &reloaded));
    assert_eq!(reloaded.diagnostics().changed_row_count, 3);
    assert_ne!(
        reloaded.dependency_revision(material_id),
        Some(dependency_revision)
    );
    assert_eq!(
        manager
            .load_states(material_handle)
            .recursive_dependency_load_state,
        RecursiveDependencyLoadState::Reloading
    );
    assert!(Arc::ptr_eq(&reloaded, &resources.readiness_generation()));
}

#[test]
fn readiness_generation_deduplicates_shared_dependency_rows_and_recomputes_one_closure() {
    let manager = ProjectAssetManager::default();
    let resources = manager.resource_manager();

    let texture = record("res://textures/readiness-shared.png", ResourceKind::Texture);
    let texture_id = texture.id;
    manager
        .assets::<TextureAsset>()
        .insert(
            texture,
            texture_asset("res://textures/readiness-shared.png"),
        )
        .unwrap();

    let mut shader_ids = Vec::new();
    for suffix in ["a", "b"] {
        let uri = format!("res://shaders/readiness-shared-{suffix}.wgsl");
        let mut shader = record(&uri, ResourceKind::Shader);
        shader.dependency_ids = vec![texture_id];
        shader_ids.push(shader.id);
        manager
            .assets::<ShaderAsset>()
            .insert(shader, shader_asset(&uri))
            .unwrap();
    }

    let mut material = record(
        "res://materials/readiness-shared.zmaterial",
        ResourceKind::Material,
    );
    material.dependency_ids = shader_ids;
    let material_handle = manager
        .assets::<MaterialAsset>()
        .insert(
            material,
            material_asset("res://shaders/readiness-shared-a.wgsl"),
        )
        .unwrap();

    let report = manager.readiness_report(material_handle);
    assert_eq!(report.dependencies.len(), 3);
    let shared = dependency_row(&report.dependencies, texture_id);
    assert_eq!(shared.depth, 2);
    assert!(!shared.direct);

    resources.start_reload(texture_id, Vec::new()).unwrap();
    assert_eq!(
        resources
            .readiness_generation()
            .diagnostics()
            .changed_row_count,
        4
    );
}
