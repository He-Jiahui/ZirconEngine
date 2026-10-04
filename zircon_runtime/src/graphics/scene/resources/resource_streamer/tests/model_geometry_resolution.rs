use super::*;

#[test]
fn runtime93_borrowed_mesh_locator_cache_deduplicates_equal_locators() {
    let first = crate::asset::AssetUri::parse("res://meshes/shared.zmesh").unwrap();
    let second = crate::asset::AssetUri::parse("res://meshes/shared.zmesh").unwrap();
    let mut mesh_assets = HashMap::<&crate::asset::AssetUri, u32>::new();

    mesh_assets.entry(&first).or_insert(11);
    mesh_assets.entry(&second).or_insert(22);

    assert_eq!(mesh_assets.len(), 1);
    assert_eq!(mesh_assets[&first], 11);
}

#[test]
fn composite_geometry_revision_changes_when_external_mesh_revision_changes() {
    let model_id = ResourceId::from_stable_label("tests/model-with-external-mesh");
    let original = dependency_state(7);
    let reloaded = dependency_state(8);

    let original_revision = model_geometry_revision(model_id, 3, &[original]);
    let reloaded_revision = model_geometry_revision(model_id, 3, &[reloaded]);

    assert_ne!(original_revision, reloaded_revision);
}

fn dependency_state(revision: u64) -> ModelMeshDependencyState {
    ModelMeshDependencyState {
        locator: crate::asset::AssetUri::parse("res://meshes/external.zmesh").unwrap(),
        resource_id: Some(ResourceId::from_stable_label("tests/external-mesh")),
        revision: Some(revision),
        state: Some(ResourceState::Ready),
    }
}
