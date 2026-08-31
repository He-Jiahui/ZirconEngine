use std::sync::Arc;

use crate::core::framework::render::{
    RENDER_MESH_STABLE_KEY_PRIMITIVE_BITS, RenderComponentChangeArtifact,
    RenderComponentChangeKind, RenderComponentMeshPayload, RenderComponentMeshPrimitiveBinding,
    RenderComponentProjectionMode, RenderComponentSnapshot, RenderComponentValue, RenderLayerSet,
    RendererCommon,
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
        upserts.push(project_primitive(scene, patch, resolver)?);
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

fn project_primitive(
    scene: &RenderScene,
    patch: &RenderComponentSnapshot,
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
        Some(previous) => project_existing(previous, patch, resolver),
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
            let geometry_changed = patch.kind() == RenderComponentChangeKind::Added
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
