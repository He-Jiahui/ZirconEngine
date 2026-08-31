use std::sync::Arc;

use crate::core::resource::{ModelMarker, ResourceHandle};

use crate::graphics::types::GraphicsError;

use super::super::prepared::{mesh_sdf_seed_from_primitives, PreparedModel};
use super::super::GpuModelResource;
use super::model_geometry_resolution::{
    model_dependencies_are_current, model_geometry_revision, resolve_model_geometry,
};
use super::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn ensure_model(
        &mut self,
        device: &wgpu::Device,
        handle: ResourceHandle<ModelMarker>,
    ) -> Result<(), GraphicsError> {
        let id = handle.id();
        let source_revision = self.resource_revision(id)?;
        let asset_manager = self.asset_manager()?;
        if self.models.get(&id).is_some_and(|prepared| {
            prepared.source_revision == source_revision
                && model_dependencies_are_current(
                    asset_manager.as_ref(),
                    &prepared.mesh_dependency_states,
                )
        }) {
            return Ok(());
        }
        let model = self
            .load_model_asset(id)
            .ok_or_else(|| GraphicsError::Asset(format!("failed to load model asset {id:?}")))?;
        let resolved = resolve_model_geometry(asset_manager.as_ref(), model.as_ref());
        let revision = model_geometry_revision(id, source_revision, &resolved.dependency_states);
        let local_bounds = resolved.local_bounds;
        let mesh_sdf = mesh_sdf_seed_from_primitives(&resolved.primitives);
        let resource = Arc::new(GpuModelResource::from_primitives(
            device,
            id,
            resolved.primitives,
        ));
        self.models.insert(
            id,
            PreparedModel {
                revision,
                source_revision,
                mesh_dependency_states: resolved.dependency_states,
                local_bounds,
                deformation: resolved.deformation,
                mesh_sdf,
                asset: model,
                resource,
            },
        );
        Ok(())
    }
}
