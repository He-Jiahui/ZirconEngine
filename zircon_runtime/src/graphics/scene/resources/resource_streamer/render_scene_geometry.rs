use std::collections::{HashMap, HashSet};

use crate::core::framework::render::{
    RenderComponentChangeArtifact, RenderComponentValue, RenderMeshBounds,
};
use crate::core::resource::ResourceId;
use crate::core::resource::{
    MeshMarker, ModelMarker, ResourceHandle, ResourceKind, UntypedResourceHandle,
};
use crate::graphics::scene::render_scene::{
    RenderSceneGeometryResolveIssue, RenderSceneGeometryResolver, RenderSceneMeshSource,
    RenderSceneMeshSourceLevel, RenderScenePrimitiveLocalBounds, RenderSceneResolvedGeometry,
};
use crate::graphics::types::GraphicsError;
use crate::graphics::RuntimePrepareMeshGeometrySeed;

use super::super::prepared::{PreparedMesh, PreparedModel};
use super::resource_streamer_accessors::geometry_seed_for_prepared;
use super::ResourceStreamer;

pub(super) struct PreparedRenderSceneGeometryResolver<'prepared> {
    models: &'prepared HashMap<ResourceId, PreparedModel>,
    meshes: &'prepared HashMap<ResourceId, PreparedMesh>,
}

impl<'prepared> PreparedRenderSceneGeometryResolver<'prepared> {
    pub(super) const fn new(
        models: &'prepared HashMap<ResourceId, PreparedModel>,
        meshes: &'prepared HashMap<ResourceId, PreparedMesh>,
    ) -> Self {
        Self { models, meshes }
    }
}

impl ResourceStreamer {
    pub(crate) fn ensure_render_scene_projection_geometry(
        &mut self,
        device: &wgpu::Device,
        artifact: &RenderComponentChangeArtifact,
    ) -> Result<(), GraphicsError> {
        let mut ensured_models = HashSet::new();
        let mut ensured_meshes = HashSet::new();
        for patch in artifact.upserts() {
            let RenderComponentValue::Present(mesh) = patch.mesh_renderer() else {
                continue;
            };
            ensure_source_level(
                self,
                device,
                mesh.model(),
                mesh.mesh(),
                mesh.primitives().iter().map(|binding| binding.mesh()),
                &mut ensured_models,
                &mut ensured_meshes,
            )?;
            for lod in mesh.lods() {
                ensure_source_level(
                    self,
                    device,
                    lod.model(),
                    lod.mesh(),
                    lod.primitives().iter().map(|binding| binding.mesh()),
                    &mut ensured_models,
                    &mut ensured_meshes,
                )?;
            }
        }
        Ok(())
    }

    pub(crate) fn ensure_render_scene_projection_geometry_resources(
        &mut self,
        device: &wgpu::Device,
        resources: impl IntoIterator<Item = UntypedResourceHandle>,
    ) -> Result<(), GraphicsError> {
        let mut meshes = HashSet::new();
        let mut models = HashSet::new();
        for resource in resources {
            match resource.kind() {
                ResourceKind::Mesh => {
                    meshes.insert(resource.id());
                }
                ResourceKind::Model => {
                    models.insert(resource.id());
                }
                _ => {}
            }
        }
        for id in models {
            self.ensure_model(device, ResourceHandle::<ModelMarker>::new(id))?;
        }
        for id in meshes {
            self.ensure_mesh(device, ResourceHandle::<MeshMarker>::new(id))?;
        }
        Ok(())
    }

    pub(super) fn render_scene_geometry_resolver(&self) -> PreparedRenderSceneGeometryResolver<'_> {
        PreparedRenderSceneGeometryResolver {
            models: &self.models,
            meshes: &self.meshes,
        }
    }
}

impl RenderSceneGeometryResolver for PreparedRenderSceneGeometryResolver<'_> {
    fn supplemental_geometry_dependencies(
        &self,
        source: &RenderSceneMeshSource,
    ) -> Vec<UntypedResourceHandle> {
        std::iter::once(source.base())
            .chain(source.lods().iter().map(|lod| &lod.source))
            .filter(|level| level.mesh.is_none())
            .filter_map(|level| self.models.get(&level.model.id()))
            .flat_map(|model| model.mesh_dependency_states.iter())
            .filter_map(|dependency| dependency.resource_id())
            .map(|id| UntypedResourceHandle::new(id, ResourceKind::Mesh))
            .collect()
    }

    fn resolve_geometry(
        &mut self,
        _entity: u64,
        source: &RenderSceneMeshSource,
        morph_weights: &[f32],
    ) -> Result<RenderSceneResolvedGeometry, RenderSceneGeometryResolveIssue> {
        let base = self.resolve_level(source.base(), morph_weights)?;
        let lods = source
            .lods()
            .iter()
            .map(|lod| self.resolve_level(&lod.source, morph_weights))
            .collect::<Result<Vec<_>, _>>()?;
        let revisions = resolved_revision_set(std::iter::once(&base).chain(lods.iter()));
        Ok(RenderSceneResolvedGeometry::new(
            RenderScenePrimitiveLocalBounds::new(
                base.local_bounds,
                lods.iter().map(|lod| lod.local_bounds).collect(),
            ),
            revisions.geometry,
            revisions.bounds,
            revisions.deformation,
        ))
    }
}

impl PreparedRenderSceneGeometryResolver<'_> {
    fn resolve_level(
        &self,
        source: &RenderSceneMeshSourceLevel,
        morph_weights: &[f32],
    ) -> Result<ResolvedSourceLevel, RenderSceneGeometryResolveIssue> {
        let primary = match source.mesh {
            Some(mesh) => self.mesh_seed(mesh.id(), morph_weights),
            None => self.model_seed(source.model.id(), morph_weights),
        }
        .ok_or(RenderSceneGeometryResolveIssue::Pending)?;
        let mut local_bounds = primary.local_bounds;
        let mut resource_revisions = Vec::with_capacity(source.primitives.len().saturating_add(1));
        resource_revisions.push(primary.resource_revision);
        for binding in source.primitives.iter() {
            let seed = self
                .mesh_seed(binding.mesh.id(), morph_weights)
                .ok_or(RenderSceneGeometryResolveIssue::Pending)?;
            local_bounds = union_bounds(local_bounds, seed.local_bounds);
            resource_revisions.push(seed.resource_revision);
        }
        Ok(ResolvedSourceLevel {
            local_bounds,
            resource_revisions,
        })
    }

    fn model_seed(
        &self,
        id: ResourceId,
        morph_weights: &[f32],
    ) -> Option<RuntimePrepareMeshGeometrySeed> {
        self.models.get(&id).map(|prepared| {
            geometry_seed_for_prepared(
                prepared.local_bounds,
                prepared.revision,
                &prepared.deformation,
                &prepared.mesh_sdf,
                morph_weights,
            )
        })
    }

    fn mesh_seed(
        &self,
        id: ResourceId,
        morph_weights: &[f32],
    ) -> Option<RuntimePrepareMeshGeometrySeed> {
        self.meshes.get(&id).map(|prepared| {
            geometry_seed_for_prepared(
                prepared.local_bounds,
                prepared.revision,
                &prepared.deformation,
                &prepared.mesh_sdf,
                morph_weights,
            )
        })
    }
}

fn ensure_source_level(
    streamer: &mut ResourceStreamer,
    device: &wgpu::Device,
    model: crate::core::resource::ResourceHandle<crate::core::resource::ModelMarker>,
    mesh: Option<crate::core::resource::ResourceHandle<crate::core::resource::MeshMarker>>,
    primitive_meshes: impl IntoIterator<
        Item = crate::core::resource::ResourceHandle<crate::core::resource::MeshMarker>,
    >,
    ensured_models: &mut HashSet<ResourceId>,
    ensured_meshes: &mut HashSet<ResourceId>,
) -> Result<(), GraphicsError> {
    match mesh {
        Some(mesh) => ensure_mesh_once(streamer, device, mesh, ensured_meshes)?,
        None if ensured_models.insert(model.id()) => streamer.ensure_model(device, model)?,
        None => {}
    }
    for mesh in primitive_meshes {
        ensure_mesh_once(streamer, device, mesh, ensured_meshes)?;
    }
    Ok(())
}

fn ensure_mesh_once(
    streamer: &mut ResourceStreamer,
    device: &wgpu::Device,
    mesh: crate::core::resource::ResourceHandle<crate::core::resource::MeshMarker>,
    ensured_meshes: &mut HashSet<ResourceId>,
) -> Result<(), GraphicsError> {
    if ensured_meshes.insert(mesh.id()) {
        streamer.ensure_mesh(device, mesh)?;
    }
    Ok(())
}

struct ResolvedSourceLevel {
    local_bounds: RenderMeshBounds,
    resource_revisions: Vec<u64>,
}

struct ResolvedRevisionSet {
    geometry: u64,
    bounds: u64,
    deformation: u64,
}

fn resolved_revision_set<'level>(
    levels: impl IntoIterator<Item = &'level ResolvedSourceLevel>,
) -> ResolvedRevisionSet {
    let levels = levels.into_iter().collect::<Vec<_>>();
    ResolvedRevisionSet {
        geometry: revision_digest(b"zircon.render-scene.geometry", &levels, false),
        bounds: revision_digest(b"zircon.render-scene.bounds", &levels, true),
        deformation: revision_digest(b"zircon.render-scene.deformation", &levels, false),
    }
}

fn revision_digest(domain: &[u8], levels: &[&ResolvedSourceLevel], include_bounds: bool) -> u64 {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(&(levels.len() as u64).to_le_bytes());
    for level in levels {
        hasher.update(&(level.resource_revisions.len() as u64).to_le_bytes());
        for revision in &level.resource_revisions {
            hasher.update(&revision.to_le_bytes());
        }
        if include_bounds {
            for component in level
                .local_bounds
                .min
                .into_iter()
                .chain(level.local_bounds.max)
            {
                hasher.update(&component.to_bits().to_le_bytes());
            }
        }
    }
    let mut revision = [0; 8];
    revision.copy_from_slice(&hasher.finalize().as_bytes()[..8]);
    u64::from_le_bytes(revision).max(1)
}

fn union_bounds(left: RenderMeshBounds, right: RenderMeshBounds) -> RenderMeshBounds {
    let mut min = left.min;
    let mut max = left.max;
    for axis in 0..3 {
        min[axis] = min[axis].min(right.min[axis]);
        max[axis] = max[axis].max(right.max[axis]);
    }
    RenderMeshBounds::from_min_max(min, max)
}

#[cfg(test)]
#[path = "tests/render_scene_geometry.rs"]
mod tests;
