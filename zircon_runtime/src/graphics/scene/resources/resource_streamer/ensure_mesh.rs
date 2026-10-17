use std::sync::Arc;
use std::hash::{Hash, Hasher};
use crate::asset::MeshAsset;
use crate::core::resource::{MeshMarker, ResourceHandle, ResourceId, ResourceProjectionSnapshot, ResourceActiveSnapshotError};
use crate::graphics::types::GraphicsError;
use crate::graphics::virtual_geometry_runtime_provider::VirtualGeometrySourceAdmission;
use super::super::prepared::{mesh_sdf_seed_from_primitives, PreparedGeometryDeformation, PreparedMesh};
use super::super::GpuMeshResource;
use super::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn ensure_mesh(&mut self, device: &wgpu::Device, handle: ResourceHandle<MeshMarker>) -> Result<(), GraphicsError> {
        let assets = self.asset_manager()?;
        assets.ensure_resident(handle.id()).map_err(|error| GraphicsError::Asset(error.to_string()))?;
        let projection = assets.resource_manager().projection_snapshot();
        let Some(prepared) = self.stage_mesh(device, handle.id(), &projection)
            .map_err(|error| GraphicsError::Asset(format!("mesh publication deferred: {error:?}")))? else { return Ok(()); };
        if assets.resource_manager().projection_snapshot().active().key(handle.id()).as_ref() != Some(&prepared.source_key) {
            return Err(GraphicsError::Asset("mesh publication changed during upload preparation".into()));
        }
        self.meshes.insert(handle.id(), prepared);
        Ok(())
    }

    pub(crate) fn admit_mesh_for_virtual_geometry(&mut self, handle: ResourceHandle<MeshMarker>) -> Result<Arc<VirtualGeometrySourceAdmission>, GraphicsError> {
        let assets = self.asset_manager()?;
        assets.ensure_resident(handle.id()).map_err(|error| GraphicsError::Asset(error.to_string()))?;
        let projection = assets.resource_manager().projection_snapshot();
        if let Some(prepared) = self.meshes.get(&handle.id()).filter(|prepared| {
            projection.active().key(handle.id()).as_ref() == Some(&prepared.source_key)
        }) { return Ok(Arc::clone(&prepared.admission)); }
        let prepared = self.build_prepared_mesh(handle.id(), &projection)
            .map_err(|error| GraphicsError::Asset(format!("mesh publication deferred: {error:?}")))?;
        if assets.resource_manager().projection_snapshot().active().key(handle.id()).as_ref() != Some(&prepared.source_key) {
            return Err(GraphicsError::Asset("mesh publication changed during preparation".into()));
        }
        let admission = Arc::clone(&prepared.admission);
        self.meshes.insert(handle.id(), prepared);
        Ok(admission)
    }

    pub(super) fn stage_mesh(&self, device: &wgpu::Device, id: ResourceId, projection: &ResourceProjectionSnapshot) -> Result<Option<PreparedMesh>, ResourceActiveSnapshotError> {
        if self.meshes.get(&id).is_some_and(|prepared| {
            projection.active().key(id).as_ref() == Some(&prepared.source_key) && prepared.resource.is_some()
        }) { return Ok(None); }
        let mut prepared = self.build_prepared_mesh(id, projection)?;
        let source = prepared.admission.primitive(0).ok_or(ResourceActiveSnapshotError::KindConflict)?;
        prepared.resource = Some(Arc::new(GpuMeshResource::from_admitted_primitive(device, source)));
        Ok(Some(prepared))
    }

    fn build_prepared_mesh(&self, id: ResourceId, projection: &ResourceProjectionSnapshot) -> Result<PreparedMesh, ResourceActiveSnapshotError> {
        let row = projection.active().row(id).ok_or(ResourceActiveSnapshotError::Stale)?;
        let snapshot = self.asset_manager().map_err(|_| ResourceActiveSnapshotError::Stale)?
            .resource_manager().snapshot_expected::<_, MeshAsset>(ResourceHandle::<MeshMarker>::new(id), &row.key)?;
        let asset = Arc::clone(snapshot.resource());
        let local_bounds = asset.bounds().map_err(|_| ResourceActiveSnapshotError::KindConflict)?;
        let primitive = asset.to_model_primitive().map_err(|_| ResourceActiveSnapshotError::KindConflict)?;
        let deformation = PreparedGeometryDeformation::from_mesh_asset(&asset);
        let mesh_sdf = mesh_sdf_seed_from_primitives(std::slice::from_ref(&primitive));
        let mut hasher = std::hash::DefaultHasher::new();
        row.record.revision.hash(&mut hasher);
        row.key.hash(&mut hasher);
        let revision = hasher.finish().max(1);
        let admission = Arc::new(VirtualGeometrySourceAdmission::new(id, row.record.revision, Vec::new(), vec![primitive]));
        Ok(PreparedMesh {
            revision, source_key: row.key.clone(), local_bounds, deformation, mesh_sdf,
            asset, admission, resource: None,
        })
    }
}
