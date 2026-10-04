use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use crate::core::framework::render::RenderLayerSet;
use crate::core::math::{Mat4, Real, Vec3};
use crate::core::resource::ResourceId as AssetId;

/// 可序列化的辐照体积输入；transform 是 world-to-volume 变换，采样前仍须由图形层取得 voxels 资源。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IrradianceVolumeData {
    pub volume_id: u64,
    pub transform: Mat4,
    pub voxels: AssetId,
    pub intensity: Real,
    pub affects_lightmapped_meshes: bool,
    pub priority: i32,
    #[serde(default)]
    pub layer_mask: RenderLayerSet,
}

impl IrradianceVolumeData {
    /// Maps a world-space position into the volume's normalized texture domain.
    /// Authored local volume bounds are `[-0.5, 0.5]` on every axis.
    pub fn world_to_uvw(&self, world_position: Vec3) -> Vec3 {
        self.transform.transform_point3(world_position) + Vec3::splat(0.5)
    }

    pub fn contains_world_position(&self, world_position: Vec3) -> bool {
        let uvw = self.world_to_uvw(world_position);
        uvw.is_finite() && uvw.cmpge(Vec3::ZERO).all() && uvw.cmple(Vec3::ONE).all()
    }
}

pub fn select_irradiance_volume<'a>(
    volumes: &'a [IrradianceVolumeData],
    world_position: Vec3,
    render_layers: &RenderLayerSet,
) -> Option<&'a IrradianceVolumeData> {
    volumes
        .iter()
        .filter(|volume| {
            volume.intensity > 0.0
                && volume.layer_mask.intersects(render_layers)
                && volume.contains_world_position(world_position)
        })
        .max_by(|left, right| irradiance_volume_priority_cmp(left, right))
}

/// 在当前相机层和可见物体位置中选出一个候选体积，供帧资源绑定使用。
/// 空位置集合允许按层选择；逐像素边界仍由着色器依据 world-to-volume 变换判断。
pub fn select_irradiance_volume_for_view<'a>(
    volumes: &'a [IrradianceVolumeData],
    render_layers: &RenderLayerSet,
    visible_world_positions: &[Vec3],
) -> Option<&'a IrradianceVolumeData> {
    let mut selected: Option<&IrradianceVolumeData> = None;
    for volume in volumes {
        if !(volume.intensity > 0.0) || !volume.layer_mask.intersects(render_layers) {
            continue;
        }
        if selected.is_some_and(|current| irradiance_volume_priority_cmp(volume, current).is_lt()) {
            continue;
        }
        if !volume.transform.is_finite()
            || !(volume.transform.determinant().abs() > Real::EPSILON)
            || (!visible_world_positions.is_empty()
                && !visible_world_positions
                    .iter()
                    .copied()
                    .any(|position| volume.contains_world_position(position)))
        {
            continue;
        }
        selected = Some(volume);
    }
    selected
}

fn irradiance_volume_priority_cmp(
    left: &IrradianceVolumeData,
    right: &IrradianceVolumeData,
) -> Ordering {
    left.priority
        .cmp(&right.priority)
        .then_with(|| right.volume_id.cmp(&left.volume_id))
}

#[cfg(test)]
#[path = "tests/irradiance_volume.rs"]
mod tests;
