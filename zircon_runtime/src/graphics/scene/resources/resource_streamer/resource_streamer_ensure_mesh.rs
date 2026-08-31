use std::sync::Arc;

use crate::asset::MeshAsset;
use crate::core::resource::{MeshMarker, ResourceHandle};

use crate::graphics::types::GraphicsError;

use super::super::GpuMeshResource;
use super::super::prepared::{
    PreparedGeometryDeformation, PreparedMesh, mesh_sdf_seed_from_primitives,
};
use super::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn ensure_mesh(
        &mut self,
        device: &wgpu::Device,
        handle: ResourceHandle<MeshMarker>,
    ) -> Result<(), GraphicsError> {
        let id = handle.id();
        let revision = self.resource_revision(id)?;
        if self
            .meshes
            .get(&id)
            .is_some_and(|prepared| prepared.revision == revision)
        {
            return Ok(());
        }

        let mesh = self
            .asset_manager()?
            .load_mesh_asset(id)
            .map_err(|error| GraphicsError::Asset(error.to_string()))?;
        let local_bounds = mesh
            .bounds()
            .map_err(|error| GraphicsError::Asset(error.to_string()))?;
        let primitive = mesh
            .to_model_primitive()
            .map_err(|error| GraphicsError::Asset(error.to_string()))?;
        let deformation = PreparedGeometryDeformation::from_mesh_asset(&mesh);
        let mesh_sdf = mesh_sdf_seed_from_primitives(std::slice::from_ref(&primitive));
        let asset = Arc::<MeshAsset>::new(mesh);
        let resource = Arc::new(GpuMeshResource::from_asset(device, primitive));
        self.meshes.insert(
            id,
            PreparedMesh {
                revision,
                local_bounds,
                deformation,
                mesh_sdf,
                asset,
                resource,
            },
        );
        Ok(())
    }
}
