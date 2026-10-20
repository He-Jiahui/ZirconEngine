use super::super::prepared::{mesh_sdf_seed_from_primitives, PreparedModel};
use super::super::GpuModelResource;
use super::model_geometry_resolution::{
    ensure_cold_model_meshes, model_geometry_revision, resolve_model_geometry,
};
use super::ResourceStreamer;
use crate::asset::ModelAsset;
use crate::core::resource::{
    ModelMarker, ResourceActiveSnapshotError, ResourceHandle, ResourceId,
    ResourceProjectionSnapshot,
};
use crate::graphics::types::GraphicsError;
use crate::graphics::virtual_geometry_runtime_provider::VirtualGeometrySourceAdmission;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

impl ResourceStreamer {
    pub(crate) fn admit_model_for_virtual_geometry(
        &mut self,
        handle: ResourceHandle<ModelMarker>,
    ) -> Result<Arc<VirtualGeometrySourceAdmission>, GraphicsError> {
        let id = handle.id();
        let model = self
            .load_model_asset(id)
            .ok_or_else(|| GraphicsError::Asset(format!("failed to load model asset {id:?}")))?;
        let assets = self.asset_manager()?;
        ensure_cold_model_meshes(&assets, &model)?;
        let projection = assets.resource_manager().projection_snapshot();
        if let Some(prepared) = self
            .models
            .get(&id)
            .filter(|prepared| model_is_current(prepared, &projection))
        {
            return Ok(Arc::clone(&prepared.admission));
        }
        let prepared = self
            .build_prepared_model(id, model, &projection)
            .map_err(|error| {
                GraphicsError::Asset(format!("model publication deferred: {error:?}"))
            })?;
        if !prepared_model_keys_match(&prepared, &assets.resource_manager().projection_snapshot()) {
            return Err(GraphicsError::Asset(
                "model publication changed during preparation".into(),
            ));
        }
        let admission = Arc::clone(&prepared.admission);
        self.models.insert(id, prepared);
        Ok(admission)
    }

    pub(crate) fn ensure_model(
        &mut self,
        device: &wgpu::Device,
        handle: ResourceHandle<ModelMarker>,
    ) -> Result<(), GraphicsError> {
        let id = handle.id();
        let assets = self.asset_manager()?;
        let projection = assets.resource_manager().projection_snapshot();
        if self.models.get(&id).is_some_and(|prepared| {
            model_is_current(prepared, &projection) && prepared.resource.is_some()
        }) {
            return Ok(());
        }
        // Preserve the real source-Arc cache route; a cache hit never clones ModelAsset bytes.
        let model = self
            .load_model_asset(id)
            .ok_or_else(|| GraphicsError::Asset(format!("failed to load model asset {id:?}")))?;
        ensure_cold_model_meshes(&assets, &model)?;
        let projection = assets.resource_manager().projection_snapshot();
        // CPU extraction may already hold this admission; GPU upload must retain its identity.
        let mut prepared = match self.models.get(&id) {
            Some(prepared) if model_is_current(prepared, &projection) => prepared.clone(),
            _ => self
                .build_prepared_model(id, model, &projection)
                .map_err(|error| {
                    GraphicsError::Asset(format!("model publication deferred: {error:?}"))
                })?,
        };
        prepared.resource = Some(Arc::new(GpuModelResource::from_source_admission(
            device,
            id,
            &prepared.admission,
        )));
        if !prepared_model_keys_match(&prepared, &assets.resource_manager().projection_snapshot()) {
            return Err(GraphicsError::Asset(
                "model publication changed during upload preparation".into(),
            ));
        }
        self.models.insert(id, prepared);
        Ok(())
    }

    pub(super) fn stage_model(
        &self,
        device: &wgpu::Device,
        id: ResourceId,
        projection: &ResourceProjectionSnapshot,
    ) -> Result<Option<PreparedModel>, ResourceActiveSnapshotError> {
        if self.models.get(&id).is_some_and(|prepared| {
            model_is_current(prepared, projection) && prepared.resource.is_some()
        }) {
            return Ok(None);
        }
        let mut prepared = match self.models.get(&id) {
            Some(prepared) if model_is_current(prepared, projection) => prepared.clone(),
            _ => {
                let row = projection
                    .active()
                    .row(id)
                    .ok_or(ResourceActiveSnapshotError::Stale)?;
                let snapshot = self
                    .asset_manager()
                    .map_err(|_| ResourceActiveSnapshotError::Stale)?
                    .resource_manager()
                    .snapshot_expected::<_, ModelAsset>(
                        ResourceHandle::<ModelMarker>::new(id),
                        &row.key,
                    )?;
                self.build_prepared_model(id, Arc::clone(snapshot.resource()), projection)?
            }
        };
        prepared.resource = Some(Arc::new(GpuModelResource::from_source_admission(
            device,
            id,
            &prepared.admission,
        )));
        Ok(Some(prepared))
    }

    fn build_prepared_model(
        &self,
        id: ResourceId,
        model: Arc<ModelAsset>,
        projection: &ResourceProjectionSnapshot,
    ) -> Result<PreparedModel, ResourceActiveSnapshotError> {
        let assets = self
            .asset_manager()
            .map_err(|_| ResourceActiveSnapshotError::Stale)?;
        let row = projection
            .active()
            .row(id)
            .ok_or(ResourceActiveSnapshotError::Stale)?;
        let actual = assets
            .resource_manager()
            .snapshot_expected::<_, ModelAsset>(ResourceHandle::<ModelMarker>::new(id), &row.key)?;
        if !Arc::ptr_eq(actual.resource(), &model) {
            return Err(ResourceActiveSnapshotError::Stale);
        }
        let resolved = resolve_model_geometry(&assets, &model, projection)?;
        let source_revision = row.record.revision;
        let mut hasher = std::hash::DefaultHasher::new();
        model_geometry_revision(id, source_revision, &resolved.dependency_states).hash(&mut hasher);
        row.key.hash(&mut hasher);
        resolved.dependency_keys.hash(&mut hasher);
        let revision = hasher.finish().max(1);
        let local_bounds = resolved.local_bounds;
        let mesh_sdf = mesh_sdf_seed_from_primitives(&resolved.primitives);
        let admission = Arc::new(VirtualGeometrySourceAdmission::new(
            id,
            source_revision,
            resolved
                .dependency_states
                .iter()
                .map(|dependency| dependency.source_snapshot())
                .collect(),
            resolved.primitives,
        ));
        Ok(PreparedModel {
            revision,
            source_revision,
            source_key: row.key.clone(),
            mesh_dependency_states: resolved.dependency_states,
            mesh_dependency_keys: resolved.dependency_keys,
            local_bounds,
            deformation: resolved.deformation,
            mesh_sdf,
            asset: model,
            admission,
            resource: None,
        })
    }
}

pub(super) fn prepared_model_keys_match(
    prepared: &PreparedModel,
    projection: &ResourceProjectionSnapshot,
) -> bool {
    projection.active().key(prepared.source_key.id()).as_ref() == Some(&prepared.source_key)
        && prepared
            .mesh_dependency_keys
            .iter()
            .all(|(id, key)| projection.active().key(*id).as_ref() == key.as_ref())
        && super::model_geometry_resolution::dependency_states_match(
            projection,
            &prepared.mesh_dependency_states,
        )
}

fn model_is_current(prepared: &PreparedModel, projection: &ResourceProjectionSnapshot) -> bool {
    prepared_model_keys_match(prepared, projection)
}
