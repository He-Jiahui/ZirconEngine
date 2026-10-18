use std::collections::HashSet;
use std::sync::Arc;

use crate::core::framework::render::{
    RenderComponentChangeArtifact, RenderComponentChangeKind, RenderComponentMeshPayload,
    RenderComponentMeshPrimitiveBinding, RenderComponentProjectionMode, RenderComponentSnapshot,
    RenderComponentValue, RenderLayerSet, RendererCommon, RENDER_MESH_STABLE_KEY_PRIMITIVE_BITS,
};
use crate::core::framework::scene::{EntityId, Mobility};
use crate::core::math::Mat4;

use super::super::{
    RenderScene, RenderSceneDelta, RenderSceneMeshBinding, RenderSceneMeshLod,
    RenderSceneMeshSource, RenderSceneMeshSourceLevel, RenderScenePrimitive,
    RenderScenePrimitiveDescriptor, RenderScenePrimitiveLocalBounds, RenderScenePrimitiveRevisions,
};
use super::{
    RenderSceneComponentProjectionError, RenderSceneGeometryResolver, RenderSceneRequiredComponent,
};

pub(super) fn build_delta(
    scene: &RenderScene,
    artifact: &RenderComponentChangeArtifact,
    resolver: &mut impl RenderSceneGeometryResolver,
) -> Result<RenderSceneDelta, RenderSceneComponentProjectionError> {
    let mut upserts = Vec::with_capacity(artifact.upserts().len());
    for patch in artifact.upserts() {
        upserts.push(project_primitive(scene, patch, false, resolver)?);
    }
    let removals = if matches!(artifact.mode(), RenderComponentProjectionMode::Full(_)) {
        full_reprojection_removals(scene, &upserts)
    } else {
        artifact
            .removals()
            .iter()
            .copied()
            .map(stable_instance_key)
            .collect::<Result<Vec<_>, _>>()?
    };
    Ok(RenderSceneDelta::new(upserts, removals))
}

pub(super) fn build_delta_with_geometry_replay(
    scene: &RenderScene,
    artifact: &RenderComponentChangeArtifact,
    geometry_targets: &[u64],
    resolver: &mut impl RenderSceneGeometryResolver,
) -> Result<RenderSceneDelta, RenderSceneComponentProjectionError> {
    let mut remaining_targets = geometry_targets.iter().copied().collect::<HashSet<_>>();
    let mut upserts = Vec::with_capacity(
        artifact
            .upserts()
            .len()
            .saturating_add(remaining_targets.len()),
    );
    for patch in artifact.upserts() {
        let key = stable_instance_key(patch.entity())?;
        let force_geometry = remaining_targets.remove(&key);
        upserts.push(project_primitive(scene, patch, force_geometry, resolver)?);
    }
    let removals = if matches!(artifact.mode(), RenderComponentProjectionMode::Full(_)) {
        full_reprojection_removals(scene, &upserts)
    } else {
        artifact
            .removals()
            .iter()
            .copied()
            .map(stable_instance_key)
            .collect::<Result<Vec<_>, _>>()?
    };
    let removal_keys = removals.iter().copied().collect::<HashSet<_>>();
    remaining_targets.retain(|key| !removal_keys.contains(key));
    let read = scene.read();
    let mut remaining_targets = remaining_targets.into_iter().collect::<Vec<_>>();
    remaining_targets.sort_unstable();
    for key in remaining_targets {
        let Some(previous) = read
            .handle_for_stable_key(key)
            .and_then(|handle| read.get(handle))
        else {
            continue;
        };
        upserts.push(reproject_geometry(previous, resolver)?);
    }
    Ok(RenderSceneDelta::new(upserts, removals))
}

/// Dormant sources retain only component metadata. They own no geometry, payload or lease.
#[derive(Clone)]
pub(super) struct DormantGeometryPrimitive {
    pub(super) descriptor: RenderScenePrimitiveDescriptor,
    pub(super) revisions: RenderScenePrimitiveRevisions,
}

impl DormantGeometryPrimitive {
    pub(super) fn from_primitive(primitive: &RenderScenePrimitive) -> Self {
        Self {
            descriptor: primitive.descriptor().clone(),
            revisions: primitive.revisions(),
        }
    }

    pub(super) fn key(&self) -> u64 {
        self.descriptor.stable_instance_key
    }
}

pub(super) struct GeometryRetirementDelta {
    pub(super) delta: RenderSceneDelta,
    pub(super) parked: Vec<DormantGeometryPrimitive>,
    pub(super) awakened: Vec<u64>,
    pub(super) forgotten: Vec<u64>,
    pub(super) preserve_dependencies: HashSet<u64>,
}

/// A retirement is an explicit Resource-authority tombstone, never a resolver failure.
/// Missing/Pending keep their existing error behavior on every route without tombstones.
pub(super) fn build_delta_with_geometry_retirements(
    scene: &RenderScene,
    dormant: &std::collections::HashMap<u64, DormantGeometryPrimitive>,
    artifact: Option<&RenderComponentChangeArtifact>,
    targets: &[u64],
    retired: &HashSet<crate::core::resource::UntypedResourceHandle>,
    resolver: &mut impl RenderSceneGeometryResolver,
) -> Result<GeometryRetirementDelta, RenderSceneComponentProjectionError> {
    if dormant.is_empty() && retired.is_empty() {
        let delta = match artifact {
            Some(artifact) if targets.is_empty() => build_delta(scene, artifact, resolver)?,
            Some(artifact) => build_delta_with_geometry_replay(scene, artifact, targets, resolver)?,
            None => {
                let read = scene.read();
                let upserts = targets
                    .iter()
                    .filter_map(|key| {
                        read.handle_for_stable_key(*key)
                            .and_then(|handle| read.get(handle))
                    })
                    .map(|previous| reproject_geometry(previous, resolver))
                    .collect::<Result<Vec<_>, _>>()?;
                RenderSceneDelta::new(upserts, Vec::new())
            }
        };
        return Ok(GeometryRetirementDelta {
            delta,
            parked: Vec::new(),
            awakened: Vec::new(),
            forgotten: Vec::new(),
            preserve_dependencies: HashSet::new(),
        });
    }
    let read = scene.read();
    let mut remaining = targets.iter().copied().collect::<HashSet<_>>();
    let mut incoming = HashSet::new();
    let mut upserts = Vec::new();
    let mut removals = std::collections::BTreeSet::new();
    let mut parked = Vec::new();
    let mut awakened = Vec::new();
    let mut forgotten = Vec::new();
    let mut preserve_dependencies = HashSet::new();
    if let Some(artifact) = artifact {
        for patch in artifact.upserts() {
            let key = stable_instance_key(patch.entity())?;
            incoming.insert(key);
            let force_geometry = remaining.remove(&key);
            let live = read
                .handle_for_stable_key(key)
                .and_then(|handle| read.get(handle));
            let prior = live
                .map(DormantGeometryPrimitive::from_primitive)
                .or_else(|| dormant.get(&key).cloned());
            let metadata = metadata_for_patch(prior.as_ref(), patch)?;
            if source_is_retired(&metadata.descriptor.mesh_source, retired) {
                if prior.as_ref().is_some_and(|prior| {
                    prior
                        .descriptor
                        .mesh_source
                        .geometry_eq(&metadata.descriptor.mesh_source)
                }) {
                    preserve_dependencies.insert(key);
                }
                if live.is_some() {
                    removals.insert(key);
                }
                parked.push(metadata);
            } else {
                let projected = if let Some(live) = live {
                    project_existing(live.clone(), patch, force_geometry, resolver)?
                } else if prior.is_some() {
                    resolve_dormant(metadata, resolver)?
                } else {
                    project_new(patch, resolver)?
                };
                awakened.push(key);
                upserts.push(projected);
            }
        }
        if matches!(artifact.mode(), RenderComponentProjectionMode::Full(_)) {
            forgotten.extend(
                read.iter()
                    .map(|(_, primitive)| primitive.stable_instance_key())
                    .chain(dormant.keys().copied())
                    .filter(|key| !incoming.contains(key)),
            );
        } else {
            forgotten.extend(
                artifact
                    .removals()
                    .iter()
                    .copied()
                    .map(stable_instance_key)
                    .collect::<Result<Vec<_>, _>>()?,
            );
        }
    }
    forgotten.sort_unstable();
    forgotten.dedup();
    for key in &forgotten {
        remaining.remove(key);
        if read.handle_for_stable_key(*key).is_some() {
            removals.insert(*key);
        }
    }
    let mut remaining = remaining.into_iter().collect::<Vec<_>>();
    remaining.sort_unstable();
    for key in remaining {
        let live = read
            .handle_for_stable_key(key)
            .and_then(|handle| read.get(handle));
        let Some(metadata) = live
            .map(DormantGeometryPrimitive::from_primitive)
            .or_else(|| dormant.get(&key).cloned())
        else {
            continue;
        };
        if source_is_retired(&metadata.descriptor.mesh_source, retired) {
            preserve_dependencies.insert(key);
            if live.is_some() {
                removals.insert(key);
            }
            parked.push(metadata);
        } else {
            awakened.push(key);
            upserts.push(resolve_dormant(metadata, resolver)?);
        }
    }
    Ok(GeometryRetirementDelta {
        delta: RenderSceneDelta::new(upserts, removals.into_iter().collect()),
        parked,
        awakened,
        forgotten,
        preserve_dependencies,
    })
}

pub(super) fn source_is_retired(
    source: &RenderSceneMeshSource,
    retired: &HashSet<crate::core::resource::UntypedResourceHandle>,
) -> bool {
    use crate::core::resource::{ResourceKind, UntypedResourceHandle};
    std::iter::once(source.base())
        .chain(source.lods().iter().map(|lod| &lod.source))
        .any(|level| {
            retired.contains(&match level.mesh {
                Some(mesh) => UntypedResourceHandle::new(mesh.id(), ResourceKind::Mesh),
                None => UntypedResourceHandle::new(level.model.id(), ResourceKind::Model),
            }) || level.primitives.iter().any(|binding| {
                retired.contains(&UntypedResourceHandle::new(
                    binding.mesh.id(),
                    ResourceKind::Mesh,
                ))
            })
        })
}

fn resolve_dormant(
    dormant: DormantGeometryPrimitive,
    resolver: &mut impl RenderSceneGeometryResolver,
) -> Result<RenderScenePrimitive, RenderSceneComponentProjectionError> {
    let entity = dormant.descriptor.node_id;
    let mut revisions = dormant.revisions;
    let bounds = resolver
        .resolve_geometry(
            entity,
            &dormant.descriptor.mesh_source,
            &dormant.descriptor.morph_weights,
        )
        .map_err(|issue| RenderSceneComponentProjectionError::GeometryResolution { entity, issue })?
        .apply_to(&mut revisions);
    RenderScenePrimitive::new(dormant.descriptor, bounds, revisions).map_err(Into::into)
}

fn metadata_for_patch(
    previous: Option<&DormantGeometryPrimitive>,
    patch: &RenderComponentSnapshot,
) -> Result<DormantGeometryPrimitive, RenderSceneComponentProjectionError> {
    let entity = patch.entity();
    if let Some(previous) = previous {
        let mut metadata = previous.clone();
        apply_transform_patch(
            &mut metadata.descriptor,
            &mut metadata.revisions,
            patch.world_matrix(),
            entity,
        )?;
        apply_active_patch(
            &mut metadata.descriptor,
            patch.active_in_hierarchy(),
            entity,
        )?;
        apply_layer_patch(&mut metadata.descriptor, patch.render_layer_mask(), entity)?;
        apply_mobility_patch(&mut metadata.descriptor, patch.mobility(), entity)?;
        match patch.mesh_renderer() {
            RenderComponentValue::Unchanged if patch.kind() == RenderComponentChangeKind::Added => {
                return Err(missing_component(
                    entity,
                    RenderSceneRequiredComponent::MeshRenderer,
                ))
            }
            RenderComponentValue::Unchanged => {}
            RenderComponentValue::Removed => {
                return Err(
                    RenderSceneComponentProjectionError::RemovedMeshRendererInUpsert { entity },
                )
            }
            RenderComponentValue::Present(mesh) => {
                let source = mesh_source(entity, mesh)?;
                apply_mesh_renderer(&mut metadata.descriptor, source, mesh);
            }
        }
        return Ok(metadata);
    }
    if patch.kind() == RenderComponentChangeKind::Updated {
        return Err(RenderSceneComponentProjectionError::MissingPrimitive { entity });
    }
    let mesh = required_present(
        patch.mesh_renderer(),
        entity,
        RenderSceneRequiredComponent::MeshRenderer,
    )?;
    let matrix = *required_present(
        patch.world_matrix(),
        entity,
        RenderSceneRequiredComponent::WorldMatrix,
    )?;
    let active = required_present(
        patch.active_in_hierarchy(),
        entity,
        RenderSceneRequiredComponent::ActiveInHierarchy,
    )?;
    let layer = required_present(
        patch.render_layer_mask(),
        entity,
        RenderSceneRequiredComponent::RenderLayerMask,
    )?;
    let mobility = *required_present(
        patch.mobility(),
        entity,
        RenderSceneRequiredComponent::Mobility,
    )?;
    let source = mesh_source(entity, mesh)?;
    Ok(DormantGeometryPrimitive {
        descriptor: descriptor(
            entity,
            stable_instance_key(entity)?,
            matrix,
            active,
            layer,
            mobility,
            source,
            mesh,
        ),
        revisions: RenderScenePrimitiveRevisions::new(0, 0, 0, 0, 0),
    })
}

pub(super) fn reproject_geometry(
    previous: &RenderScenePrimitive,
    resolver: &mut impl RenderSceneGeometryResolver,
) -> Result<RenderScenePrimitive, RenderSceneComponentProjectionError> {
    let entity = previous.descriptor().node_id;
    let mut revisions = previous.revisions();
    let local_bounds = resolver
        .resolve_geometry(
            entity,
            &previous.descriptor().mesh_source,
            &previous.descriptor().morph_weights,
        )
        .map_err(|issue| RenderSceneComponentProjectionError::GeometryResolution { entity, issue })?
        .apply_to(&mut revisions);
    RenderScenePrimitive::new(previous.descriptor().clone(), local_bounds, revisions)
        .map_err(Into::into)
}

fn project_primitive(
    scene: &RenderScene,
    patch: &RenderComponentSnapshot,
    force_geometry: bool,
    resolver: &mut impl RenderSceneGeometryResolver,
) -> Result<RenderScenePrimitive, RenderSceneComponentProjectionError> {
    let entity = patch.entity();
    let key = stable_instance_key(entity)?;
    let previous = {
        let read = scene.read();
        read.handle_for_stable_key(key)
            .and_then(|handle| read.get(handle).cloned())
    };
    if patch.kind() == RenderComponentChangeKind::Updated && previous.is_none() {
        return Err(RenderSceneComponentProjectionError::MissingPrimitive { entity });
    }

    match previous {
        Some(previous) => project_existing(previous, patch, force_geometry, resolver),
        None => project_new(patch, resolver),
    }
}

fn project_new(
    patch: &RenderComponentSnapshot,
    resolver: &mut impl RenderSceneGeometryResolver,
) -> Result<RenderScenePrimitive, RenderSceneComponentProjectionError> {
    let entity = patch.entity();
    let mesh = required_present(
        patch.mesh_renderer(),
        entity,
        RenderSceneRequiredComponent::MeshRenderer,
    )?;
    let world_from_local = *required_present(
        patch.world_matrix(),
        entity,
        RenderSceneRequiredComponent::WorldMatrix,
    )?;
    let active = required_present(
        patch.active_in_hierarchy(),
        entity,
        RenderSceneRequiredComponent::ActiveInHierarchy,
    )?;
    let layer = required_present(
        patch.render_layer_mask(),
        entity,
        RenderSceneRequiredComponent::RenderLayerMask,
    )?;
    let mobility = *required_present(
        patch.mobility(),
        entity,
        RenderSceneRequiredComponent::Mobility,
    )?;
    let mesh_source = mesh_source(entity, mesh)?;
    let mut revisions = RenderScenePrimitiveRevisions::new(0, 0, 0, 0, 0);
    let local_bounds = resolver
        .resolve_geometry(entity, &mesh_source, mesh.morph_weights())
        .map_err(|issue| RenderSceneComponentProjectionError::GeometryResolution { entity, issue })?
        .apply_to(&mut revisions);
    RenderScenePrimitive::new(
        descriptor(
            entity,
            stable_instance_key(entity)?,
            world_from_local,
            active,
            layer,
            mobility,
            mesh_source,
            mesh,
        ),
        local_bounds,
        revisions,
    )
    .map_err(Into::into)
}

fn project_existing(
    previous: RenderScenePrimitive,
    patch: &RenderComponentSnapshot,
    force_geometry: bool,
    resolver: &mut impl RenderSceneGeometryResolver,
) -> Result<RenderScenePrimitive, RenderSceneComponentProjectionError> {
    let entity = patch.entity();
    let mut descriptor = previous.descriptor().clone();
    let mut local_bounds = previous.local_bounds_source().clone();
    let mut revisions = previous.revisions();

    apply_transform_patch(
        &mut descriptor,
        &mut revisions,
        patch.world_matrix(),
        entity,
    )?;
    apply_active_patch(&mut descriptor, patch.active_in_hierarchy(), entity)?;
    apply_layer_patch(&mut descriptor, patch.render_layer_mask(), entity)?;
    apply_mobility_patch(&mut descriptor, patch.mobility(), entity)?;

    match patch.mesh_renderer() {
        RenderComponentValue::Unchanged => {
            if patch.kind() == RenderComponentChangeKind::Added {
                return Err(missing_component(
                    entity,
                    RenderSceneRequiredComponent::MeshRenderer,
                ));
            }
        }
        RenderComponentValue::Removed => {
            return Err(
                RenderSceneComponentProjectionError::RemovedMeshRendererInUpsert { entity },
            );
        }
        RenderComponentValue::Present(mesh) => {
            let source = mesh_source(entity, mesh)?;
            let geometry_changed = force_geometry
                || patch.kind() == RenderComponentChangeKind::Added
                || !source.geometry_eq(&descriptor.mesh_source)
                || mesh.morph_weights() != descriptor.morph_weights.as_ref();
            apply_mesh_renderer(&mut descriptor, source, mesh);
            if geometry_changed {
                local_bounds = resolver
                    .resolve_geometry(entity, &descriptor.mesh_source, mesh.morph_weights())
                    .map_err(
                        |issue| RenderSceneComponentProjectionError::GeometryResolution {
                            entity,
                            issue,
                        },
                    )?
                    .apply_to(&mut revisions);
            }
        }
    }

    if force_geometry && matches!(patch.mesh_renderer(), RenderComponentValue::Unchanged) {
        local_bounds = resolver
            .resolve_geometry(entity, &descriptor.mesh_source, &descriptor.morph_weights)
            .map_err(
                |issue| RenderSceneComponentProjectionError::GeometryResolution { entity, issue },
            )?
            .apply_to(&mut revisions);
    }

    RenderScenePrimitive::new(descriptor, local_bounds, revisions).map_err(Into::into)
}

fn descriptor(
    entity: EntityId,
    stable_instance_key: u64,
    world_from_local: Mat4,
    active: &bool,
    layer: &u32,
    mobility: Mobility,
    mesh_source: RenderSceneMeshSource,
    mesh: &RenderComponentMeshPayload,
) -> RenderScenePrimitiveDescriptor {
    let is_static = mobility == Mobility::Static;
    RenderScenePrimitiveDescriptor {
        node_id: entity,
        stable_instance_key,
        world_from_local,
        mesh_source,
        morph_weights: Arc::from(mesh.morph_weights()),
        skeletal_pose: None,
        tint: mesh.tint(),
        material_property_overrides: mesh.material_property_overrides().clone(),
        material_alpha_mode: mesh.material_alpha_mode(),
        render_queue: mesh.render_queue(),
        material_queue: mesh.material_queue(),
        order_in_layer: mesh.order_in_layer(),
        depth_bias: mesh.depth_bias(),
        mobility,
        transform_static: is_static,
        common: RendererCommon {
            enabled: *active,
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(*layer),
            is_static,
            ..RendererCommon::default()
        },
    }
}

fn apply_mesh_renderer(
    descriptor: &mut RenderScenePrimitiveDescriptor,
    source: RenderSceneMeshSource,
    mesh: &RenderComponentMeshPayload,
) {
    descriptor.mesh_source = source;
    descriptor.morph_weights = Arc::from(mesh.morph_weights());
    descriptor.tint = mesh.tint();
    descriptor.material_property_overrides = mesh.material_property_overrides().clone();
    descriptor.material_alpha_mode = mesh.material_alpha_mode();
    descriptor.render_queue = mesh.render_queue();
    descriptor.material_queue = mesh.material_queue();
    descriptor.order_in_layer = mesh.order_in_layer();
    descriptor.depth_bias = mesh.depth_bias();
}

fn mesh_source(
    entity: EntityId,
    mesh: &RenderComponentMeshPayload,
) -> Result<RenderSceneMeshSource, RenderSceneComponentProjectionError> {
    let base = source_level(
        mesh.model(),
        mesh.mesh(),
        mesh.material(),
        mesh.primitives(),
    );
    let lods = mesh
        .lods()
        .iter()
        .map(|lod| {
            RenderSceneMeshLod::new(
                lod.min_distance(),
                source_level(lod.model(), lod.mesh(), lod.material(), lod.primitives()),
            )
        })
        .collect::<Vec<_>>();
    let mut source = RenderSceneMeshSource::new(base, lods);
    source
        .canonicalize_lods()
        .map_err(|issue| RenderSceneComponentProjectionError::InvalidLodSource { entity, issue })?;
    Ok(source)
}

fn source_level(
    model: crate::core::resource::ResourceHandle<crate::core::resource::ModelMarker>,
    mesh: Option<crate::core::resource::ResourceHandle<crate::core::resource::MeshMarker>>,
    material: crate::core::resource::ResourceHandle<crate::core::resource::MaterialMarker>,
    primitives: &[RenderComponentMeshPrimitiveBinding],
) -> RenderSceneMeshSourceLevel {
    RenderSceneMeshSourceLevel::new(
        model,
        mesh,
        material,
        primitives
            .iter()
            .map(|primitive| RenderSceneMeshBinding {
                mesh: primitive.mesh(),
                material: primitive.material(),
            })
            .collect::<Vec<_>>(),
    )
}

fn apply_transform_patch(
    descriptor: &mut RenderScenePrimitiveDescriptor,
    revisions: &mut RenderScenePrimitiveRevisions,
    value: &RenderComponentValue<Mat4>,
    entity: EntityId,
) -> Result<(), RenderSceneComponentProjectionError> {
    match value {
        RenderComponentValue::Unchanged => Ok(()),
        RenderComponentValue::Present(matrix) => {
            if descriptor.world_from_local != *matrix {
                descriptor.world_from_local = *matrix;
                revisions.transform = revisions.transform.saturating_add(1);
            }
            Ok(())
        }
        RenderComponentValue::Removed => Err(missing_component(
            entity,
            RenderSceneRequiredComponent::WorldMatrix,
        )),
    }
}

fn apply_active_patch(
    descriptor: &mut RenderScenePrimitiveDescriptor,
    value: &RenderComponentValue<bool>,
    entity: EntityId,
) -> Result<(), RenderSceneComponentProjectionError> {
    match value {
        RenderComponentValue::Unchanged => Ok(()),
        RenderComponentValue::Present(active) => {
            descriptor.common.enabled = *active;
            Ok(())
        }
        RenderComponentValue::Removed => Err(missing_component(
            entity,
            RenderSceneRequiredComponent::ActiveInHierarchy,
        )),
    }
}

fn apply_layer_patch(
    descriptor: &mut RenderScenePrimitiveDescriptor,
    value: &RenderComponentValue<u32>,
    entity: EntityId,
) -> Result<(), RenderSceneComponentProjectionError> {
    match value {
        RenderComponentValue::Unchanged => Ok(()),
        RenderComponentValue::Present(layer) => {
            descriptor.common.layer_mask = RenderLayerSet::from_scene_schema_v1_mask(*layer);
            Ok(())
        }
        RenderComponentValue::Removed => Err(missing_component(
            entity,
            RenderSceneRequiredComponent::RenderLayerMask,
        )),
    }
}

fn apply_mobility_patch(
    descriptor: &mut RenderScenePrimitiveDescriptor,
    value: &RenderComponentValue<Mobility>,
    entity: EntityId,
) -> Result<(), RenderSceneComponentProjectionError> {
    match value {
        RenderComponentValue::Unchanged => Ok(()),
        RenderComponentValue::Present(mobility) => {
            descriptor.mobility = *mobility;
            let is_static = *mobility == Mobility::Static;
            descriptor.transform_static = is_static;
            descriptor.common.is_static = is_static;
            Ok(())
        }
        RenderComponentValue::Removed => Err(missing_component(
            entity,
            RenderSceneRequiredComponent::Mobility,
        )),
    }
}

fn required_present<'value, T>(
    value: &'value RenderComponentValue<T>,
    entity: EntityId,
    component: RenderSceneRequiredComponent,
) -> Result<&'value T, RenderSceneComponentProjectionError> {
    match value {
        RenderComponentValue::Present(value) => Ok(value),
        RenderComponentValue::Unchanged | RenderComponentValue::Removed => {
            Err(missing_component(entity, component))
        }
    }
}

fn missing_component(
    entity: EntityId,
    component: RenderSceneRequiredComponent,
) -> RenderSceneComponentProjectionError {
    RenderSceneComponentProjectionError::MissingRequiredComponent { entity, component }
}

fn full_reprojection_removals(scene: &RenderScene, upserts: &[RenderScenePrimitive]) -> Vec<u64> {
    let mut live_keys = scene
        .read()
        .iter()
        .map(|(_, primitive)| primitive.stable_instance_key())
        .collect::<Vec<_>>();
    let mut incoming_keys = upserts
        .iter()
        .map(RenderScenePrimitive::stable_instance_key)
        .collect::<Vec<_>>();
    live_keys.sort_unstable();
    incoming_keys.sort_unstable();
    live_keys
        .into_iter()
        .filter(|key| incoming_keys.binary_search(key).is_err())
        .collect()
}

fn stable_instance_key(entity: EntityId) -> Result<u64, RenderSceneComponentProjectionError> {
    if entity > (u64::MAX >> RENDER_MESH_STABLE_KEY_PRIMITIVE_BITS) {
        return Err(RenderSceneComponentProjectionError::EntityExceedsStableKeyCapacity { entity });
    }
    Ok(entity << RENDER_MESH_STABLE_KEY_PRIMITIVE_BITS)
}
