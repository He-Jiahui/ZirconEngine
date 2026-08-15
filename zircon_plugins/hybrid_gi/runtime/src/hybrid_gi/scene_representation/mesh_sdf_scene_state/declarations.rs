use zircon_runtime::core::framework::render::{RenderMeshBounds, RenderMeshSnapshot};
use zircon_runtime::core::framework::scene::Mobility;
use zircon_runtime::core::resource::ResourceId;

use super::super::{HybridGiGlobalSdfClipmapBounds, HybridGiMeshSdfAssetState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::hybrid_gi) struct HybridGiMeshSdfMaterialFlags {
    pub(in crate::hybrid_gi) casts_shadows: bool,
    pub(in crate::hybrid_gi) emissive: bool,
}

impl Default for HybridGiMeshSdfMaterialFlags {
    fn default() -> Self {
        Self {
            casts_shadows: true,
            emissive: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::hybrid_gi) struct HybridGiMeshSdfObjectFlags {
    pub(in crate::hybrid_gi) visible: bool,
    pub(in crate::hybrid_gi) movable: bool,
    pub(in crate::hybrid_gi) casts_shadow: bool,
    pub(in crate::hybrid_gi) emissive: bool,
    pub(in crate::hybrid_gi) indirect_while_hidden: bool,
}

impl HybridGiMeshSdfObjectFlags {
    fn participates_in_world_trace(self) -> bool {
        self.visible || self.casts_shadow || self.emissive || self.indirect_while_hidden
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::hybrid_gi) struct HybridGiMeshSdfObject {
    pub(super) stable_instance_key: u64,
    pub(super) geometry_asset_id: ResourceId,
    pub(super) resource_revision: u64,
    pub(super) scene_geometry_revision: u64,
    pub(super) shape_revision: u64,
    pub(super) transform_revision: u64,
    pub(super) bounds: RenderMeshBounds,
    pub(super) world_to_local: [[f32; 4]; 4],
    pub(super) distance_scale: f32,
    pub(super) flags: HybridGiMeshSdfObjectFlags,
    pub(super) asset_state: HybridGiMeshSdfAssetState,
    pub(super) influenced_clipmap_ids: Vec<u32>,
}

impl HybridGiMeshSdfObject {
    pub(in crate::hybrid_gi) fn from_sources(
        mesh: &RenderMeshSnapshot,
        local_bounds: RenderMeshBounds,
        resource_revision: u64,
        shape_revision: u64,
        asset_state: HybridGiMeshSdfAssetState,
        material: HybridGiMeshSdfMaterialFlags,
        clipmaps: &[HybridGiGlobalSdfClipmapBounds],
    ) -> Self {
        let visible = mesh.common.enabled && mesh.common.cast_shadows.renders_in_main_view();
        let casts_shadow = mesh.common.enabled
            && mesh.common.cast_shadows.casts_shadows()
            && material.casts_shadows;
        let flags = HybridGiMeshSdfObjectFlags {
            visible,
            movable: mesh.mobility == Mobility::Dynamic,
            casts_shadow,
            emissive: mesh.common.enabled && material.emissive,
            indirect_while_hidden: mesh.common.enabled && !visible && casts_shadow,
        };
        let world_from_local = mesh.transform.matrix();
        let inverse = world_from_local.inverse();
        let distance_scale = [
            world_from_local.x_axis.truncate().length(),
            world_from_local.y_axis.truncate().length(),
            world_from_local.z_axis.truncate().length(),
        ]
        .into_iter()
        .fold(f32::INFINITY, f32::min);
        let transform_valid =
            inverse.is_finite() && distance_scale.is_finite() && distance_scale > f32::EPSILON;
        let mut bounds = local_bounds.transformed(mesh.transform);
        if asset_state.uses_unbounded_skinning_fallback() {
            bounds = clipmaps
                .iter()
                .copied()
                .map(HybridGiGlobalSdfClipmapBounds::world_bounds)
                .reduce(union_bounds)
                .unwrap_or(bounds);
        }
        let mut influenced_clipmap_ids = if flags.participates_in_world_trace() {
            clipmaps
                .iter()
                .copied()
                .filter(|clipmap| clipmap.intersects(bounds))
                .map(HybridGiGlobalSdfClipmapBounds::clipmap_id)
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        influenced_clipmap_ids.sort_unstable();
        influenced_clipmap_ids.dedup();

        Self {
            stable_instance_key: mesh.stable_instance_key,
            geometry_asset_id: mesh.mesh.map_or(mesh.model.id(), |handle| handle.id()),
            resource_revision,
            scene_geometry_revision: mesh.static_state.geometry_revision,
            shape_revision,
            transform_revision: mesh.transform_revision,
            bounds,
            world_to_local: if transform_valid {
                inverse.to_cols_array_2d()
            } else {
                zircon_runtime::core::math::Mat4::IDENTITY.to_cols_array_2d()
            },
            distance_scale: if transform_valid { distance_scale } else { 0.0 },
            flags,
            asset_state,
            influenced_clipmap_ids,
        }
    }

    pub(in crate::hybrid_gi) fn stable_instance_key(&self) -> u64 {
        self.stable_instance_key
    }

    pub(in crate::hybrid_gi) fn bounds(&self) -> RenderMeshBounds {
        self.bounds
    }

    pub(in crate::hybrid_gi) fn flags(&self) -> HybridGiMeshSdfObjectFlags {
        self.flags
    }

    pub(in crate::hybrid_gi) fn participates_in_global_sdf(&self) -> bool {
        self.flags.participates_in_world_trace()
    }

    pub(in crate::hybrid_gi) fn world_to_local(&self) -> [[f32; 4]; 4] {
        self.world_to_local
    }

    pub(in crate::hybrid_gi) fn distance_scale(&self) -> f32 {
        self.distance_scale
    }

    pub(in crate::hybrid_gi) fn asset_state(&self) -> &HybridGiMeshSdfAssetState {
        &self.asset_state
    }

    pub(in crate::hybrid_gi) fn influenced_clipmap_ids(&self) -> &[u32] {
        &self.influenced_clipmap_ids
    }
}

fn union_bounds(left: RenderMeshBounds, right: RenderMeshBounds) -> RenderMeshBounds {
    let left_min = zircon_runtime::core::math::Vec3::from_array(left.min);
    let left_max = zircon_runtime::core::math::Vec3::from_array(left.max);
    let right_min = zircon_runtime::core::math::Vec3::from_array(right.min);
    let right_max = zircon_runtime::core::math::Vec3::from_array(right.max);
    RenderMeshBounds::from_min_max(
        left_min.min(right_min).to_array(),
        left_max.max(right_max).to_array(),
    )
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(in crate::hybrid_gi) struct HybridGiMeshSdfSceneState {
    pub(super) objects: Vec<HybridGiMeshSdfObject>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(in crate::hybrid_gi) struct HybridGiMeshSdfSyncReport {
    pub(super) dirty_regions: Vec<RenderMeshBounds>,
}

impl HybridGiMeshSdfSyncReport {
    pub(in crate::hybrid_gi) fn dirty_regions(&self) -> &[RenderMeshBounds] {
        &self.dirty_regions
    }
}
