use crate::asset::{
    MESH_ATTRIBUTE_JOINT_WEIGHT, MESH_ATTRIBUTE_POSITION, MeshAsset, MeshAttributeValues,
    ModelPrimitiveAsset,
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

    pub(in crate::graphics::scene::resources) fn include_mesh_asset(&mut self, asset: &MeshAsset) {
        self.has_skinning |= asset.skin.is_some();
        if !self.has_skinning
            && let Some(MeshAttributeValues::Float32x4(joint_weights)) =
                asset.attributes.get(MESH_ATTRIBUTE_JOINT_WEIGHT)
            && joint_weights
                .iter()
                .any(|weights| weights.iter().any(|weight| weight.abs() > f32::EPSILON))
            && asset.validate().is_ok()
        {
            self.has_skinning = true;
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
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::asset::{AssetUri, MESH_ATTRIBUTE_JOINT_WEIGHT, MESH_ATTRIBUTE_POSITION};
    use crate::core::framework::render::RenderMeshTopology;

    #[test]
    fn morph_bounds_cover_positive_and_negative_weighted_delta_extrema() {
        let deformation = PreparedGeometryDeformation {
            morph_target_delta_bounds: vec![
                RenderMeshBounds::from_min_max([-2.0, -4.0, -6.0], [4.0, 6.0, 8.0]),
                RenderMeshBounds::from_min_max([-3.0, -5.0, -7.0], [2.0, 4.0, 6.0]),
            ],
            has_skinning: false,
        };

        let bounds = deformation.local_bounds_for_morph_weights(
            RenderMeshBounds::from_min_max([10.0, 20.0, 30.0], [12.0, 22.0, 32.0]),
            &[0.5, -0.25],
        );

        assert_eq!(bounds.min, [8.5, 17.0, 25.5]);
        assert_eq!(bounds.max, [14.75, 26.25, 37.75]);
    }

    #[test]
    fn mesh_skinning_detection_does_not_build_a_model_primitive() {
        let source = include_str!("prepared_geometry_deformation.rs")
            .split_once("#[cfg(test)]")
            .expect("production source and tests must remain separated")
            .0;

        assert!(!source.contains("to_model_primitive"));
    }

    #[test]
    fn mesh_skinning_detection_reads_joint_weight_attributes_directly() {
        let mut attributes = BTreeMap::new();
        attributes.insert(
            MESH_ATTRIBUTE_POSITION.to_string(),
            MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]),
        );
        attributes.insert(
            MESH_ATTRIBUTE_JOINT_WEIGHT.to_string(),
            MeshAttributeValues::Float32x4(vec![
                [0.0, 0.0, 0.0, 1.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
            ]),
        );
        let mesh = MeshAsset {
            uri: AssetUri::parse("res://meshes/skinned.zmesh").expect("valid test URI"),
            topology: RenderMeshTopology::TriangleList,
            attributes,
            indices: None,
            asset_usage: Default::default(),
            morph_targets: Vec::new(),
            skin: None,
            mesh_sdf: None,
            virtual_geometry: None,
        };

        assert!(PreparedGeometryDeformation::from_mesh_asset(&mesh).has_skinning());
    }

    #[test]
    fn invalid_mesh_attributes_do_not_enable_skinning() {
        let mut attributes = BTreeMap::new();
        attributes.insert(
            MESH_ATTRIBUTE_POSITION.to_string(),
            MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 0.0]]),
        );
        attributes.insert(
            MESH_ATTRIBUTE_JOINT_WEIGHT.to_string(),
            MeshAttributeValues::Float32x4(vec![[0.0, 0.0, 0.0, 1.0], [0.0, 0.0, 0.0, 1.0]]),
        );
        let mesh = MeshAsset {
            uri: AssetUri::parse("res://meshes/invalid-skinned.zmesh").expect("valid test URI"),
            topology: RenderMeshTopology::TriangleList,
            attributes,
            indices: None,
            asset_usage: Default::default(),
            morph_targets: Vec::new(),
            skin: None,
            mesh_sdf: None,
            virtual_geometry: None,
        };

        assert!(!PreparedGeometryDeformation::from_mesh_asset(&mesh).has_skinning());
    }
}
