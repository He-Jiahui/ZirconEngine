use super::ProjectAssetManagementGeneration;
use crate::asset::{
    MaterialAssetManagementRecordSet, MeshAssetManagementRecordSet, ModelAssetManagementRecordSet,
    SceneAssetManagementRecordSet, SceneEntityManagementRecordSet, ShaderAssetManagementRecordSet,
};
use crate::core::resource::{ResourceKind, ResourceManagementGeneration};
use std::sync::Arc;

#[test]
fn empty_generation_has_asset_only_identity_and_indexes() {
    let generation = ProjectAssetManagementGeneration::empty();

    let resources = Arc::new(ResourceManagementGeneration::default());
    assert!(generation.resource_generation_identity().is_none());
    assert!(!generation.is_for_generations(0, &resources.identity()));
    for kind in [
        ResourceKind::Model,
        ResourceKind::Mesh,
        ResourceKind::Scene,
        ResourceKind::Material,
        ResourceKind::Shader,
        ResourceKind::Texture,
    ] {
        assert!(generation.ids_by_kind(kind).is_empty());
    }
    assert!(generation.model_record_set().records.is_empty());
}

#[test]
fn asset_generation_remains_renderer_detail_free() {
    let generation = ProjectAssetManagementGeneration::empty();
    assert!(generation.material_record_set().records.is_empty());
    assert!(generation.resource_generation_identity().is_none());
}

#[test]
fn empty_active_project_generation_is_distinct_from_closed_projection() {
    let generation = ProjectAssetManagementGeneration::from_record_sets(
        Some(7),
        Some(Arc::new(ResourceManagementGeneration::default()).identity()),
        ModelAssetManagementRecordSet::from_records(Vec::new()),
        MeshAssetManagementRecordSet::from_results(Vec::new()),
        SceneAssetManagementRecordSet::from_records(Vec::new()),
        SceneEntityManagementRecordSet::from_records(Vec::new()),
        MaterialAssetManagementRecordSet::from_records(Vec::new()),
        ShaderAssetManagementRecordSet::from_records(Vec::new()),
    );

    assert!(generation.is_empty());
    assert!(generation.has_project_generation());
    assert!(!ProjectAssetManagementGeneration::empty().has_project_generation());
}

#[test]
fn astra_m10_cache_matches_the_publication_and_project_identity() {
    let resources = Arc::new(ResourceManagementGeneration::default());
    let same_publication = Arc::clone(&resources);
    let distinct_publication = Arc::new(ResourceManagementGeneration::default());
    assert_eq!(resources.diagnostics(), distinct_publication.diagnostics());

    let generation = ProjectAssetManagementGeneration::from_record_sets(
        Some(7),
        Some(resources.identity()),
        ModelAssetManagementRecordSet::from_records(Vec::new()),
        MeshAssetManagementRecordSet::from_results(Vec::new()),
        SceneAssetManagementRecordSet::from_records(Vec::new()),
        SceneEntityManagementRecordSet::from_records(Vec::new()),
        MaterialAssetManagementRecordSet::from_records(Vec::new()),
        ShaderAssetManagementRecordSet::from_records(Vec::new()),
    );

    assert!(generation.is_for_generations(7, &same_publication.identity()));
    assert!(!generation.is_for_generations(8, &resources.identity()));
    assert!(!generation.is_for_generations(7, &distinct_publication.identity()));
    assert_eq!(
        generation.resource_generation_identity(),
        Some(&resources.identity())
    );

    let retained = Arc::downgrade(&resources);
    drop(resources);
    drop(same_publication);
    assert!(retained.upgrade().is_some());
    drop(generation);
    assert!(retained.upgrade().is_none());
}
