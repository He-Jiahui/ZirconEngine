use std::collections::HashMap;

use crate::asset::ProjectAssetManager;
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

impl ModelMeshDependencyState {
    pub(super) fn source_snapshot(&self) -> crate::graphics::VirtualGeometrySourceDependency {
        crate::graphics::VirtualGeometrySourceDependency {
            locator: self.locator.clone(),
            resource_id: self.resource_id,
            revision: self.revision,
            state: self.state,
        }
    }

    pub(super) fn resource_id(&self) -> Option<ResourceId> {
        self.resource_id
    }
}

pub(super) struct ResolvedModelGeometry {
    pub(super) primitives: Vec<ModelPrimitiveAsset>,
    pub(super) local_bounds: RenderMeshBounds,
    pub(super) deformation: PreparedGeometryDeformation,
    pub(super) dependency_states: Vec<ModelMeshDependencyState>,
    pub(super) dependency_keys: Vec<(
        ResourceId,
        Option<crate::core::resource::ResourceActiveVersionKey>,
    )>,
}

/// Resolve all external meshes against one captured active projection. A captured resident
/// that drifts or has the wrong actual type never falls back to an inline primitive.
pub(super) fn resolve_model_geometry(
    asset_manager: &ProjectAssetManager,
    model: &ModelAsset,
    projection: &crate::core::resource::ResourceProjectionSnapshot,
) -> Result<ResolvedModelGeometry, crate::core::resource::ResourceActiveSnapshotError> {
    use crate::core::resource::{MeshMarker, ResourceActiveSnapshotError, ResourceHandle};
    let mut mesh_assets =
        HashMap::<&crate::asset::AssetUri, Option<std::sync::Arc<MeshAsset>>>::new();
    let mut dependency_states = Vec::new();
    let mut dependency_keys = Vec::new();
    let mut deformation = PreparedGeometryDeformation::default();
    let mut primitives = Vec::with_capacity(model.primitives.len());
    for primitive in &model.primitives {
        let Some(reference) = primitive.mesh.as_ref() else {
            deformation.include_primitive(primitive);
            primitives.push(primitive.clone());
            continue;
        };
        if !mesh_assets.contains_key(&reference.locator) {
            let state = dependency_state(projection, reference);
            if let Some(id) = state.resource_id {
                dependency_keys.push((id, projection.active().key(id)));
            }
            dependency_states.push(state);
            let mesh = if let Some(id) = projection.active().id_for_locator(&reference.locator) {
                let row = projection
                    .active()
                    .row(id)
                    .expect("active locator has a row");
                let snapshot = asset_manager
                    .resource_manager()
                    .snapshot_expected::<_, MeshAsset>(
                        ResourceHandle::<MeshMarker>::new(id),
                        &row.key,
                    )?;
                Some(std::sync::Arc::clone(snapshot.resource()))
            } else {
                None
            };
            mesh_assets.insert(&reference.locator, mesh);
        }
        if let Some(mesh) = mesh_assets.get(&reference.locator).and_then(Option::as_ref) {
            let resolved = mesh
                .to_model_primitive()
                .map_err(|_| ResourceActiveSnapshotError::KindConflict)?;
            deformation.include_mesh_asset(mesh);
            primitives.push(resolved);
        } else {
            // Only captured absence permits the existing inline fallback.
            deformation.include_primitive(primitive);
            primitives.push(primitive.clone());
        }
    }
    let local_bounds = RenderMeshBounds::from_positions(
        primitives
            .iter()
            .flat_map(|primitive| primitive.vertices.iter().map(|vertex| vertex.position)),
    );
    Ok(ResolvedModelGeometry {
        primitives,
        local_bounds,
        deformation,
        dependency_states,
        dependency_keys,
    })
}

/// Cold dependency loading precedes the captured projection used to stage geometry.
pub(super) fn ensure_cold_model_meshes(
    asset_manager: &ProjectAssetManager,
    model: &ModelAsset,
) -> Result<(), crate::graphics::types::GraphicsError> {
    for reference in model
        .primitives
        .iter()
        .filter_map(|primitive| primitive.mesh.as_ref())
    {
        let projection = asset_manager.resource_manager().projection_snapshot();
        if projection
            .active()
            .id_for_locator(&reference.locator)
            .is_some()
        {
            continue;
        }
        let Some(row) = projection
            .management()
            .row_by_locator(&reference.locator.to_string())
        else {
            continue;
        };
        if row.kind != crate::core::resource::ResourceKind::Mesh {
            continue;
        }
        let ready = projection
            .readiness()
            .row_identity(row.id)
            .is_some_and(|identity| identity.row().record.state == ResourceState::Ready);
        if ready {
            asset_manager
                .ensure_resident(row.id)
                .map_err(|error| crate::graphics::types::GraphicsError::Asset(error.to_string()))?;
        }
    }
    Ok(())
}

pub(super) fn model_dependencies_are_current(
    asset_manager: &ProjectAssetManager,
    dependencies: &[ModelMeshDependencyState],
) -> bool {
    dependency_states_match(
        &asset_manager.resource_manager().projection_snapshot(),
        dependencies,
    )
}

pub(super) fn dependency_states_match(
    projection: &crate::core::resource::ResourceProjectionSnapshot,
    dependencies: &[ModelMeshDependencyState],
) -> bool {
    dependencies.iter().all(|expected| {
        let reference = AssetReference::from_locator(expected.locator.clone());
        dependency_state(projection, &reference) == *expected
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
    projection: &crate::core::resource::ResourceProjectionSnapshot,
    reference: &AssetReference,
) -> ModelMeshDependencyState {
    if let Some(id) = projection.active().id_for_locator(&reference.locator) {
        let row = projection
            .active()
            .row(id)
            .expect("active locator has a row");
        return ModelMeshDependencyState {
            locator: reference.locator.clone(),
            resource_id: Some(id),
            revision: Some(row.record.revision),
            state: Some(row.record.state),
        };
    }
    let identity = projection
        .management()
        .row_by_locator(&reference.locator.to_string())
        .and_then(|row| projection.readiness().row_identity(row.id));
    let record = identity.as_ref().map(|identity| &identity.row().record);
    ModelMeshDependencyState {
        locator: reference.locator.clone(),
        resource_id: record.map(|record| record.id),
        revision: record.map(|record| record.revision),
        state: record.map(|record| record.state),
    }
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
#[path = "../resources/tests/model_geometry_resolution.rs"]
mod tests;
