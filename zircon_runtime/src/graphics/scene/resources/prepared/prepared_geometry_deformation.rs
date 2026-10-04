use crate::asset::{
    MeshAsset, MeshAttributeValues, ModelPrimitiveAsset, MESH_ATTRIBUTE_JOINT_WEIGHT,
    MESH_ATTRIBUTE_POSITION,
};
use crate::core::framework::render::RenderMeshBounds;
use crate::core::math::Vec3;

#[derive(Clone, Debug, Default)]
pub(in crate::graphics::scene::resources) struct PreparedGeometryDeformation {
    morph_target_delta_bounds: Vec<RenderMeshBounds>,
    has_skinning: bool,
}

impl PreparedGeometryDeformation {
    pub(in crate::graphics::scene::resources) fn from_mesh_asset(asset: &MeshAsset) -> Self {
        let mut deformation = Self::default();
        deformation.include_mesh_asset(asset);
        deformation
    }

    /// 汇总蒙皮与形变边界；显式 skin 优先，否则须同时存在非零权重和有效属性，避免为探测权重构造整份 primitive。
    pub(in crate::graphics::scene::resources) fn include_mesh_asset(&mut self, asset: &MeshAsset) {
        self.has_skinning |= asset.skin.is_some();
        if !self.has_skinning {
            if let Some(MeshAttributeValues::Float32x4(joint_weights)) =
                asset.attributes.get(MESH_ATTRIBUTE_JOINT_WEIGHT)
            {
                if joint_weights
                    .iter()
                    .any(|weights| weights.iter().any(|weight| weight.abs() > f32::EPSILON))
                    && asset.validate().is_ok()
                {
                    self.has_skinning = true;
                }
            }
        }
        for (target_index, target) in asset.morph_targets.iter().enumerate() {
            let Some(MeshAttributeValues::Float32x3(position_deltas)) =
                target.attributes.get(MESH_ATTRIBUTE_POSITION)
            else {
                continue;
            };
            if position_deltas.is_empty() {
                continue;
            }
            let bounds = RenderMeshBounds::from_positions(position_deltas.iter().copied());
            if let Some(existing) = self.morph_target_delta_bounds.get_mut(target_index) {
                *existing = union_bounds(*existing, bounds);
            } else {
                self.morph_target_delta_bounds.resize(
                    target_index,
                    RenderMeshBounds::from_min_max([0.0; 3], [0.0; 3]),
                );
                self.morph_target_delta_bounds.push(bounds);
            }
        }
    }

    pub(in crate::graphics::scene::resources) fn include_primitive(
        &mut self,
        primitive: &ModelPrimitiveAsset,
    ) {
        self.has_skinning |= primitive.uses_skinning_channels();
    }

    pub(in crate::graphics::scene::resources) fn has_skinning(&self) -> bool {
        self.has_skinning
    }

    pub(in crate::graphics::scene::resources) fn local_bounds_for_morph_weights(
        &self,
        base_bounds: RenderMeshBounds,
        morph_weights: &[f32],
    ) -> RenderMeshBounds {
        let mut min = Vec3::from_array(base_bounds.min);
        let mut max = Vec3::from_array(base_bounds.max);
        for (target_index, delta_bounds) in self.morph_target_delta_bounds.iter().enumerate() {
            let weight = morph_weights.get(target_index).copied().unwrap_or_default();
            if !weight.is_finite() || weight.abs() <= f32::EPSILON {
                continue;
            }
            // 负 morph 权重会交换区间端点，须重新取 min/max 后累加，才能把形变范围保守地并入基础边界。
            let delta_min = Vec3::from_array(delta_bounds.min) * weight;
            let delta_max = Vec3::from_array(delta_bounds.max) * weight;
            min += delta_min.min(delta_max);
            max += delta_min.max(delta_max);
        }
        RenderMeshBounds::from_min_max(min.to_array(), max.to_array())
    }
}

fn union_bounds(left: RenderMeshBounds, right: RenderMeshBounds) -> RenderMeshBounds {
    RenderMeshBounds::from_min_max(
        Vec3::from_array(left.min)
            .min(Vec3::from_array(right.min))
            .to_array(),
        Vec3::from_array(left.max)
            .max(Vec3::from_array(right.max))
            .to_array(),
    )
}

#[cfg(test)]
#[path = "tests/prepared_geometry_deformation.rs"]
mod tests;
