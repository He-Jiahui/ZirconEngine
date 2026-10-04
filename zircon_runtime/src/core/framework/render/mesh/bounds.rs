use serde::{Deserialize, Serialize};

use crate::core::math::{Mat4, Transform, Vec3};

/// 资产、场景和可见性共用的局部/世界包围数据；min/max 是权威值，center/radius 是派生值。
/// 不可信输入须先校验有限值和轴序，再交给场景重建派生值。
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RenderMeshBounds {
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub center: [f32; 3],
    pub radius: f32,
}

impl RenderMeshBounds {
    pub fn from_min_max(min: [f32; 3], max: [f32; 3]) -> Self {
        let center = [
            (min[0] + max[0]) * 0.5,
            (min[1] + max[1]) * 0.5,
            (min[2] + max[2]) * 0.5,
        ];
        Self {
            min,
            max,
            center,
            radius: max_distance_from_center([min, max], center),
        }
    }

    // BUG: [CR-RENDER-MESH-0001] 含 NaN 的顶点可通过网格校验；min/max 忽略 NaN 后报告看似有效但不完整的包围盒。
    /// 为导入网格、模型和变形几何建立保守包围盒；空顶点集返回零盒，调用方负责判定其语义。
    pub fn from_positions(positions: impl IntoIterator<Item = [f32; 3]>) -> Self {
        let mut iter = positions.into_iter();
        let Some(first) = iter.next() else {
            return Self::default();
        };

        let mut min = first;
        let mut max = first;
        for position in iter {
            for axis in 0..3 {
                min[axis] = min[axis].min(position[axis]);
                max[axis] = max[axis].max(position[axis]);
            }
        }

        Self::from_min_max(min, max)
    }

    pub fn transformed(self, transform: Transform) -> Self {
        self.transformed_by_affine(transform.matrix())
    }

    /// 场景提交时将局部 AABB 投影到世界空间；包含旋转和剪切的仿射变换也必须保持保守包围。
    pub(crate) fn transformed_by_affine(self, world_from_local: Mat4) -> Self {
        let local_min = Vec3::from_array(self.min);
        let local_max = Vec3::from_array(self.max);
        let local_center = (local_min + local_max) * 0.5;
        let local_half_extent = (local_max - local_min).abs() * 0.5;
        let world_center = world_from_local.transform_point3(local_center);
        let world_half_extent = world_from_local
            .transform_vector3(Vec3::new(local_half_extent.x, 0.0, 0.0))
            .abs()
            + world_from_local
                .transform_vector3(Vec3::new(0.0, local_half_extent.y, 0.0))
                .abs()
            + world_from_local
                .transform_vector3(Vec3::new(0.0, 0.0, local_half_extent.z))
                .abs();
        Self::from_min_max(
            (world_center - world_half_extent).to_array(),
            (world_center + world_half_extent).to_array(),
        )
    }
}

fn max_distance_from_center(points: [[f32; 3]; 2], center: [f32; 3]) -> f32 {
    points
        .into_iter()
        .map(|point| {
            let dx = point[0] - center[0];
            let dy = point[1] - center[1];
            let dz = point[2] - center[2];
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .fold(0.0, f32::max)
}

#[cfg(test)]
#[path = "tests/bounds.rs"]
mod tests;
