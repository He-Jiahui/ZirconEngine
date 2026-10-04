use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::core::framework::render::{RenderMeshSnapshot, RenderMeshStaticState};
use crate::core::framework::scene::Mobility;
use crate::core::resource::ResourceId;

use super::atlas::ShadowSlotKey;
use super::slot::GpuShadowSlot;

const STATIC_CASTER_REVISION_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const STATIC_CASTER_REVISION_PRIME: u64 = 0x0000_0100_0000_01b3;

/// One caster contribution to a conservative static-shadow revision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ShadowStaticCasterRevisionInput {
    pub(crate) stable_instance_key: u64,
    pub(crate) transform_revision: u64,
    pub(crate) static_state: RenderMeshStaticState,
}

impl ShadowStaticCasterRevisionInput {
    pub(crate) const fn new(
        stable_instance_key: u64,
        transform_revision: u64,
        static_state: RenderMeshStaticState,
    ) -> Self {
        Self {
            stable_instance_key,
            transform_revision,
            static_state,
        }
    }
}

/// Computes an order-independent revision for all static shadow casters.
///
/// A missing authoritative revision or a movable caster returns `None`; callers must treat that
/// as a cache miss. This deliberately biases toward redraws, never stale static depth.
pub(crate) fn static_shadow_caster_revision(
    casters: impl IntoIterator<Item = ShadowStaticCasterRevisionInput>,
) -> Option<u64> {
    let mut casters = casters.into_iter().collect::<Vec<_>>();
    if casters
        .iter()
        .any(|caster| !caster.static_state.has_authoritative_revisions())
    {
        return None;
    }
    casters.sort_unstable_by_key(|caster| caster.stable_instance_key);

    let mut revision = STATIC_CASTER_REVISION_OFFSET_BASIS;
    for caster in casters {
        revision = fnv1a_u64(revision, caster.stable_instance_key);
        revision = fnv1a_u64(revision, caster.transform_revision);
        revision = fnv1a_u64(revision, caster.static_state.geometry_revision);
        revision = fnv1a_u64(revision, caster.static_state.material_revision);
    }
    Some(revision)
}

/// Builds the static-depth revision from the extracted mesh set.
///
/// Dynamic casters are intentionally excluded: the later overlay pass redraws them after a static
/// cache hit. Static casters with uncertain ownership still disable the static cache for the
/// frame, because their omitted depth cannot be recovered by the dynamic overlay.
pub(crate) fn static_shadow_caster_revision_from_meshes(
    meshes: &[RenderMeshSnapshot],
) -> Option<u64> {
    static_shadow_caster_revision(meshes.iter().filter_map(|mesh| {
        (mesh.common.enabled
            && mesh.common.cast_shadows.casts_shadows()
            && mesh.mobility == Mobility::Static)
            .then_some(ShadowStaticCasterRevisionInput::new(
                mesh.stable_instance_key,
                mesh.transform_revision,
                mesh.static_state,
            ))
    }))
}

/// Builds the static-depth revision from resource-manager revisions instead of extract defaults.
///
/// The render extract intentionally keeps resource revisions optional so it can remain broadly
/// reusable. The render path supplies the authoritative ready revisions here; unavailable input
/// fails closed and disables reuse for the frame.
pub(crate) fn static_shadow_caster_revision_from_meshes_with_resource_revisions(
    meshes: &[RenderMeshSnapshot],
    mut resource_revision: impl FnMut(ResourceId) -> Option<u64>,
) -> Option<u64> {
    let mut casters = Vec::new();
    for mesh in meshes {
        if !mesh.common.enabled
            || !mesh.common.cast_shadows.casts_shadows()
            || mesh.mobility != Mobility::Static
        {
            continue;
        }

        let model_resource = mesh.model.id();
        let model_revision = resource_revision(model_resource)?;
        let geometry_revision = if let Some(mesh_resource) = mesh.mesh {
            let mesh_resource = mesh_resource.id();
            let mesh_revision = resource_revision(mesh_resource)?;
            resource_revision_fingerprint([
                (model_resource, model_revision),
                (mesh_resource, mesh_revision),
            ])
        } else {
            resource_revision_fingerprint([(model_resource, model_revision)])
        };
        let material_resource = mesh.material.id();
        let material_revision = resource_revision(material_resource)?;
        let material_revision =
            resource_revision_fingerprint([(material_resource, material_revision)]);
        if casters.is_empty() {
            casters.reserve_exact(meshes.len());
        }
        casters.push(ShadowStaticCasterRevisionInput::new(
            mesh.stable_instance_key,
            mesh.transform_revision,
            RenderMeshStaticState::new(
                mesh.static_state.transform_static,
                geometry_revision,
                material_revision,
            ),
        ));
    }
    static_shadow_caster_revision(casters)
}

fn resource_revision_fingerprint(resources: impl IntoIterator<Item = (ResourceId, u64)>) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for (resource, revision) in resources {
        resource.hash(&mut hasher);
        revision.hash(&mut hasher);
    }
    hasher.finish().max(1)
}

fn fnv1a_u64(mut hash: u64, value: u64) -> u64 {
    for byte in value.to_le_bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(STATIC_CASTER_REVISION_PRIME);
    }
    hash
}

/// Hashes the complete shader-visible light and atlas parameters for one shadow slot.
///
/// This intentionally hashes float bit patterns instead of approximate values. A tiny camera or
/// light change may produce a slightly different depth map, so treating it as a hit would be a
/// correctness bug rather than a useful cache optimization.
pub(crate) fn shadow_light_params_hash(slot: &GpuShadowSlot) -> u64 {
    let mut hash = STATIC_CASTER_REVISION_OFFSET_BASIS;
    for column in slot.view_proj {
        for value in column {
            hash = fnv1a_u64(hash, u64::from(value.to_bits()));
        }
    }
    for value in slot.atlas_scale_bias {
        hash = fnv1a_u64(hash, u64::from(value.to_bits()));
    }
    for value in slot.params {
        hash = fnv1a_u64(hash, u64::from(value.to_bits()));
    }
    hash
}

/// Immutable input used to decide whether one atlas slot can retain its static depth content.
///
/// Callers must make each revision monotonic. An unavailable or uncertain revision is represented
/// by a new value, forcing a redraw instead of risking a stale shadow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ShadowCacheInput {
    pub(crate) slot_key: ShadowSlotKey,
    pub(crate) light_params_hash: u64,
    pub(crate) static_caster_revision: u64,
    pub(crate) atlas_slot_generation: u64,
}

impl ShadowCacheInput {
    pub(crate) const fn new(
        slot_key: ShadowSlotKey,
        light_params_hash: u64,
        static_caster_revision: u64,
        atlas_slot_generation: u64,
    ) -> Self {
        Self {
            slot_key,
            light_params_hash,
            static_caster_revision,
            atlas_slot_generation,
        }
    }
}

/// Cached static-depth identity for one atlas slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ShadowCacheEntry {
    pub(crate) light_params_hash: u64,
    pub(crate) static_caster_revision: u64,
    pub(crate) atlas_slot_generation: u64,
}

impl From<ShadowCacheInput> for ShadowCacheEntry {
    fn from(input: ShadowCacheInput) -> Self {
        Self {
            light_params_hash: input.light_params_hash,
            static_caster_revision: input.static_caster_revision,
            atlas_slot_generation: input.atlas_slot_generation,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShadowCacheInvalidationReason {
    Missing,
    LightParametersChanged,
    StaticCastersChanged,
    AtlasSlotReallocated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShadowCacheDecision {
    ReuseStaticDepth,
    RedrawStaticDepth(ShadowCacheInvalidationReason),
}

/// Per-slot static shadow cache identity table.
///
/// This module deliberately decides only cache validity. The atlas copy and dynamic-caster overlay
/// are separate render passes and must consume a `ReuseStaticDepth` result explicitly; no cached
/// content is assumed to be available merely because a slot has been allocated.
#[derive(Debug, Default)]
pub(crate) struct ShadowCache {
    entries: HashMap<ShadowSlotKey, ShadowCacheEntry>,
}

impl ShadowCache {
    pub(crate) fn evaluate(&self, input: ShadowCacheInput) -> ShadowCacheDecision {
        let Some(entry) = self.entries.get(&input.slot_key) else {
            return ShadowCacheDecision::RedrawStaticDepth(ShadowCacheInvalidationReason::Missing);
        };
        if entry.light_params_hash != input.light_params_hash {
            return ShadowCacheDecision::RedrawStaticDepth(
                ShadowCacheInvalidationReason::LightParametersChanged,
            );
        }
        if entry.static_caster_revision != input.static_caster_revision {
            return ShadowCacheDecision::RedrawStaticDepth(
                ShadowCacheInvalidationReason::StaticCastersChanged,
            );
        }
        if entry.atlas_slot_generation != input.atlas_slot_generation {
            return ShadowCacheDecision::RedrawStaticDepth(
                ShadowCacheInvalidationReason::AtlasSlotReallocated,
            );
        }
        ShadowCacheDecision::ReuseStaticDepth
    }

    /// Records a successfully rendered static-depth result after its atlas write has completed.
    pub(crate) fn commit_static_depth(&mut self, input: ShadowCacheInput) {
        self.entries.insert(input.slot_key, input.into());
    }

    /// Removes slots which are absent from the current atlas allocation.
    pub(crate) fn retain_slots(&mut self, mut slots: impl FnMut(&ShadowSlotKey) -> bool) {
        self.entries.retain(|slot, _| slots(slot));
    }

    pub(crate) fn clear(&mut self) {
        self.entries.clear();
    }

    #[cfg(test)]
    pub(crate) fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
#[path = "shadow_cache/tests/preallocated_static_casters_tests.rs"]
mod preallocated_static_casters_tests;

#[cfg(test)]
#[path = "tests/shadow_cache.rs"]
mod tests;
