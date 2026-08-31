use std::cmp::Ordering;

use crate::core::resource::{ResourceId, ResourceKind, UntypedResourceHandle};

use super::{
    RenderSceneAddedPrimitive, RenderScenePrimitive, RenderScenePrimitiveDirtyFlags,
    RenderSceneRemovedPrimitive, RenderSceneUpdatedPrimitive,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RenderSceneResourceReferenceDelta {
    resource: UntypedResourceHandle,
    acquired_count: usize,
    released_count: usize,
}

impl RenderSceneResourceReferenceDelta {
    pub(crate) const fn acquire(resource: UntypedResourceHandle, count: usize) -> Self {
        Self {
            resource,
            acquired_count: count,
            released_count: 0,
        }
    }

    pub(crate) const fn release(resource: UntypedResourceHandle, count: usize) -> Self {
        Self {
            resource,
            acquired_count: 0,
            released_count: count,
        }
    }

    pub(crate) const fn resource(self) -> UntypedResourceHandle {
        self.resource
    }

    pub(crate) const fn acquired_count(self) -> usize {
        self.acquired_count
    }

    pub(crate) const fn released_count(self) -> usize {
        self.released_count
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RenderSceneResourceReferenceDeltaStats {
    projected_primitive_payload_count: usize,
    unique_dependency_key_visit_count: usize,
    gross_observation_count: usize,
    net_delta_count: usize,
}

impl RenderSceneResourceReferenceDeltaStats {
    pub(crate) const fn projected_primitive_payload_count(self) -> usize {
        self.projected_primitive_payload_count
    }

    pub(crate) const fn unique_dependency_key_visit_count(self) -> usize {
        self.unique_dependency_key_visit_count
    }

    pub(crate) const fn gross_observation_count(self) -> usize {
        self.gross_observation_count
    }

    pub(crate) const fn net_delta_count(self) -> usize {
        self.net_delta_count
    }

    fn record_projection(&mut self, unique_dependency_count: usize) {
        self.projected_primitive_payload_count += 1;
        self.unique_dependency_key_visit_count += unique_dependency_count;
    }
}

pub(super) struct RenderSceneResourceReferenceDeltaBuild {
    pub(super) deltas: Vec<RenderSceneResourceReferenceDelta>,
    pub(super) stats: RenderSceneResourceReferenceDeltaStats,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum RenderSceneResourceDependencyKind {
    Model,
    Mesh,
    Material,
    AnimationSkeleton,
}

impl RenderSceneResourceDependencyKind {
    const fn resource_kind(self) -> ResourceKind {
        match self {
            Self::Model => ResourceKind::Model,
            Self::Mesh => ResourceKind::Mesh,
            Self::Material => ResourceKind::Material,
            Self::AnimationSkeleton => ResourceKind::AnimationSkeleton,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct RenderSceneResourceDependencyKey {
    kind: RenderSceneResourceDependencyKind,
    id: ResourceId,
}

impl RenderSceneResourceDependencyKey {
    const fn new(kind: RenderSceneResourceDependencyKind, id: ResourceId) -> Self {
        Self { kind, id }
    }

    fn untyped(self) -> UntypedResourceHandle {
        UntypedResourceHandle::new(self.id, self.kind.resource_kind())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RenderSceneResourceReferenceObservation {
    resource: RenderSceneResourceDependencyKey,
    acquired: bool,
}

pub(super) fn build_resource_reference_deltas(
    removals: &[RenderSceneRemovedPrimitive],
    updates: &[RenderSceneUpdatedPrimitive],
    additions: &[RenderSceneAddedPrimitive],
) -> RenderSceneResourceReferenceDeltaBuild {
    let mut observations = Vec::<RenderSceneResourceReferenceObservation>::new();
    let mut previous_dependencies = Vec::new();
    let mut current_dependencies = Vec::new();
    let mut stats = RenderSceneResourceReferenceDeltaStats::default();
    for removal in removals {
        collect_resource_dependencies(removal.primitive(), &mut current_dependencies);
        stats.record_projection(current_dependencies.len());
        record_dependencies(&mut observations, &current_dependencies, false);
    }
    for update in updates {
        if !resource_dependencies_may_have_changed(update.dirty()) {
            continue;
        }
        collect_resource_dependencies(update.previous_primitive(), &mut previous_dependencies);
        collect_resource_dependencies(update.primitive(), &mut current_dependencies);
        stats.record_projection(previous_dependencies.len());
        stats.record_projection(current_dependencies.len());
        record_dependency_difference(
            &mut observations,
            &previous_dependencies,
            &current_dependencies,
        );
    }
    for addition in additions {
        collect_resource_dependencies(addition.primitive(), &mut current_dependencies);
        stats.record_projection(current_dependencies.len());
        record_dependencies(&mut observations, &current_dependencies, true);
    }

    stats.gross_observation_count = observations.len();
    observations.sort_unstable_by_key(|observation| observation.resource);
    let deltas = collapse_observations(observations);
    stats.net_delta_count = deltas.len();
    RenderSceneResourceReferenceDeltaBuild { deltas, stats }
}

fn resource_dependencies_may_have_changed(dirty: RenderScenePrimitiveDirtyFlags) -> bool {
    dirty.contains(RenderScenePrimitiveDirtyFlags::GEOMETRY)
        || dirty.contains(RenderScenePrimitiveDirtyFlags::MATERIAL)
        || dirty.contains(RenderScenePrimitiveDirtyFlags::DEFORMATION)
}

fn collect_resource_dependencies(
    primitive: &RenderScenePrimitive,
    dependencies: &mut Vec<RenderSceneResourceDependencyKey>,
) {
    dependencies.clear();
    let descriptor = primitive.descriptor();
    include_source_level(dependencies, descriptor.mesh_source.base());
    for lod in descriptor.mesh_source.lods() {
        include_source_level(dependencies, &lod.source);
    }
    for (_, material) in descriptor.common.material_overrides.slots() {
        dependencies.push(RenderSceneResourceDependencyKey::new(
            RenderSceneResourceDependencyKind::Material,
            material.id(),
        ));
    }
    if let Some(pose) = descriptor.skeletal_pose.as_ref() {
        dependencies.push(RenderSceneResourceDependencyKey::new(
            RenderSceneResourceDependencyKind::AnimationSkeleton,
            *pose.skeleton(),
        ));
    }
    dependencies.sort_unstable();
    dependencies.dedup();
}

fn include_source_level(
    dependencies: &mut Vec<RenderSceneResourceDependencyKey>,
    level: &super::RenderSceneMeshSourceLevel,
) {
    dependencies.push(RenderSceneResourceDependencyKey::new(
        RenderSceneResourceDependencyKind::Model,
        level.model.id(),
    ));
    if let Some(mesh) = level.mesh {
        dependencies.push(RenderSceneResourceDependencyKey::new(
            RenderSceneResourceDependencyKind::Mesh,
            mesh.id(),
        ));
    }
    dependencies.push(RenderSceneResourceDependencyKey::new(
        RenderSceneResourceDependencyKind::Material,
        level.material.id(),
    ));
    for primitive in level.primitives.iter() {
        dependencies.push(RenderSceneResourceDependencyKey::new(
            RenderSceneResourceDependencyKind::Mesh,
            primitive.mesh.id(),
        ));
        dependencies.push(RenderSceneResourceDependencyKey::new(
            RenderSceneResourceDependencyKind::Material,
            primitive.material.id(),
        ));
    }
}

fn record_dependencies(
    observations: &mut Vec<RenderSceneResourceReferenceObservation>,
    dependencies: &[RenderSceneResourceDependencyKey],
    acquired: bool,
) {
    observations.extend(
        dependencies
            .iter()
            .copied()
            .map(|resource| RenderSceneResourceReferenceObservation { resource, acquired }),
    );
}

fn record_dependency_difference(
    observations: &mut Vec<RenderSceneResourceReferenceObservation>,
    previous: &[RenderSceneResourceDependencyKey],
    current: &[RenderSceneResourceDependencyKey],
) {
    let mut previous_index = 0;
    let mut current_index = 0;
    while previous_index < previous.len() && current_index < current.len() {
        match previous[previous_index].cmp(&current[current_index]) {
            Ordering::Less => {
                record_dependencies(
                    observations,
                    &previous[previous_index..=previous_index],
                    false,
                );
                previous_index += 1;
            }
            Ordering::Greater => {
                record_dependencies(observations, &current[current_index..=current_index], true);
                current_index += 1;
            }
            Ordering::Equal => {
                previous_index += 1;
                current_index += 1;
            }
        }
    }
    record_dependencies(observations, &previous[previous_index..], false);
    record_dependencies(observations, &current[current_index..], true);
}

fn collapse_observations(
    observations: Vec<RenderSceneResourceReferenceObservation>,
) -> Vec<RenderSceneResourceReferenceDelta> {
    let mut deltas = Vec::with_capacity(observations.len());
    let mut index = 0;
    while index < observations.len() {
        let resource = observations[index].resource;
        let mut acquired_count = 0;
        let mut released_count = 0;
        while index < observations.len() && observations[index].resource == resource {
            if observations[index].acquired {
                acquired_count += 1;
            } else {
                released_count += 1;
            }
            index += 1;
        }
        match acquired_count.cmp(&released_count) {
            Ordering::Greater => deltas.push(RenderSceneResourceReferenceDelta::acquire(
                resource.untyped(),
                acquired_count - released_count,
            )),
            Ordering::Less => deltas.push(RenderSceneResourceReferenceDelta::release(
                resource.untyped(),
                released_count - acquired_count,
            )),
            Ordering::Equal => {}
        }
    }
    deltas
}
