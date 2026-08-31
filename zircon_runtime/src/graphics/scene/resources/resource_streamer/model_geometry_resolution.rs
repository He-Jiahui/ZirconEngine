use std::collections::HashMap;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::{AssetReference, MeshAsset, ModelAsset, ModelPrimitiveAsset};
use crate::core::framework::render::RenderMeshBounds;
use crate::core::resource::{ResourceId, ResourceState};
use crate::graphics::scene::resources::prepared::PreparedGeometryDeformation;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics::scene::resources) struct ModelMeshDependencyState {
    locator: crate::asset::AssetUri,
    resource_id: Option<ResourceId>,
    revision: Option<u64>,
    state: Option<ResourceState>,
}

pub(super) struct ResolvedModelGeometry {
    pub(super) primitives: Vec<ModelPrimitiveAsset>,
    pub(super) local_bounds: RenderMeshBounds,
    pub(super) deformation: PreparedGeometryDeformation,
    pub(super) dependency_states: Vec<ModelMeshDependencyState>,
}

pub(super) fn resolve_model_geometry(
    asset_manager: &ProjectAssetManager,
    model: &ModelAsset,
) -> ResolvedModelGeometry {
    let mut mesh_assets = HashMap::<&crate::asset::AssetUri, Option<MeshAsset>>::new();
    let mut dependency_states = Vec::new();
    let mut deformation = PreparedGeometryDeformation::default();
    let primitives = model
        .primitives
        .iter()
        .map(|primitive| {
            let Some(reference) = primitive.mesh.as_ref() else {
                deformation.include_primitive(primitive);
                return primitive.clone();
            };
            let mesh = mesh_assets.entry(&reference.locator).or_insert_with(|| {
                dependency_states.push(dependency_state(asset_manager, reference));
                load_referenced_mesh_asset(asset_manager, reference)
            });
            if let Some(mesh) = mesh.as_ref() {
                deformation.include_mesh_asset(mesh);
                mesh.to_model_primitive()
                    .unwrap_or_else(|_| primitive.clone())
            } else {
                deformation.include_primitive(primitive);
                primitive.clone()
            }
        })
        .collect::<Vec<_>>();
    let local_bounds = RenderMeshBounds::from_positions(
        primitives
            .iter()
            .flat_map(|primitive| primitive.vertices.iter().map(|vertex| vertex.position)),
    );
    ResolvedModelGeometry {
        primitives,
        local_bounds,
        deformation,
        dependency_states,
    }
}

pub(super) fn model_dependencies_are_current(
    asset_manager: &ProjectAssetManager,
    dependencies: &[ModelMeshDependencyState],
) -> bool {
    dependencies.iter().all(|expected| {
        let reference = AssetReference::from_locator(expected.locator.clone());
        dependency_state(asset_manager, &reference) == *expected
    })
}

pub(super) fn model_geometry_revision(
    model_id: ResourceId,
    source_revision: u64,
    dependencies: &[ModelMeshDependencyState],
) -> u64 {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"zircon.prepared-model-geometry");
    hasher.update(model_id.to_string().as_bytes());
    hasher.update(&source_revision.to_le_bytes());
    for dependency in dependencies {
        hasher.update(dependency.locator.to_string().as_bytes());
        hasher.update(
            dependency
                .resource_id
                .map_or_else(|| "missing".to_string(), |id| id.to_string())
                .as_bytes(),
        );
        hasher.update(&dependency.revision.unwrap_or_default().to_le_bytes());
        hasher.update(&[resource_state_tag(dependency.state)]);
    }
    let mut revision_bytes = [0_u8; 8];
    revision_bytes.copy_from_slice(&hasher.finalize().as_bytes()[..8]);
    u64::from_le_bytes(revision_bytes).max(1)
}

fn dependency_state(
    asset_manager: &ProjectAssetManager,
    reference: &AssetReference,
) -> ModelMeshDependencyState {
    let resource_manager = asset_manager.resource_manager();
    let registry = resource_manager.registry();
    let record = registry.get_by_locator(&reference.locator);
    ModelMeshDependencyState {
        locator: reference.locator.clone(),
        resource_id: record.as_ref().map(|record| record.id()),
        revision: record.as_ref().map(|record| record.revision),
        state: record.as_ref().map(|record| record.state),
    }
}

fn load_referenced_mesh_asset(
    asset_manager: &ProjectAssetManager,
    reference: &AssetReference,
) -> Option<MeshAsset> {
    let id = asset_manager
        .resource_manager()
        .registry()
        .get_by_locator(&reference.locator)
        .map(|record| record.id())?;
    asset_manager.load_mesh_asset(id).ok()
}

const fn resource_state_tag(state: Option<ResourceState>) -> u8 {
    match state {
        None => 0,
        Some(ResourceState::Pending) => 1,
        Some(ResourceState::Ready) => 2,
        Some(ResourceState::Error) => 3,
        Some(ResourceState::Reloading) => 4,
    }
}

#[cfg(test)]
mod tests {
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
}
