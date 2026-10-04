use crate::core::math::Vec3;

/// 网格上传时汇总顶点位置，供设备资源与调试线框持有紧凑局部边界。
pub(super) struct MeshBoundsAccumulator {
    min: Vec3,
    max: Vec3,
}

impl Default for MeshBoundsAccumulator {
    fn default() -> Self {
        Self {
            min: Vec3::splat(f32::INFINITY),
            max: Vec3::splat(f32::NEG_INFINITY),
        }
    }
}

impl MeshBoundsAccumulator {
    pub(super) fn include_position(&mut self, position: [f32; 3]) {
        let position = Vec3::from_array(position);
        self.min = self.min.min(position);
        self.max = self.max.max(position);
    }

    // TODO: [CR-GRAPHICS-SCENERES-0003] 确认非有限顶点的资产准入策略；MeshAsset::validate 只校验格式/数量/索引，此处将无效范围变成零边界，需验证剔除与诊断路径。
    pub(super) fn finish(self) -> (Vec3, Vec3) {
        if !self.min.is_finite() || !self.max.is_finite() {
            (Vec3::ZERO, Vec3::ZERO)
        } else {
            (self.min, self.max)
        }
    }
}

#[cfg(test)]
#[path = "tests/mesh_bounds.rs"]
mod tests;
