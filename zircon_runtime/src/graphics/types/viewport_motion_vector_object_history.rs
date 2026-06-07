use std::collections::BTreeMap;

use crate::core::framework::render::RenderMeshSnapshot;
use crate::core::framework::scene::{EntityId, Mobility};
use crate::core::math::Transform;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ViewportMotionVectorObjectHistory {
    transforms: BTreeMap<EntityId, Transform>,
}

impl ViewportMotionVectorObjectHistory {
    pub(crate) fn from_meshes(meshes: &[RenderMeshSnapshot]) -> Self {
        let transforms = meshes
            .iter()
            .filter(|mesh| mesh.mobility == Mobility::Dynamic)
            .map(|mesh| (mesh.node_id, mesh.transform))
            .collect();
        Self { transforms }
    }

    pub(crate) fn transform(&self, entity: EntityId) -> Option<&Transform> {
        self.transforms.get(&entity)
    }

    pub(crate) fn matched_transform_count(&self, previous: Option<&Self>) -> usize {
        let Some(previous) = previous else {
            return 0;
        };
        self.transforms
            .keys()
            .filter(|entity| previous.transforms.contains_key(entity))
            .count()
    }

    pub(crate) fn missing_transform_count(&self, previous: Option<&Self>) -> usize {
        self.len()
            .saturating_sub(self.matched_transform_count(previous))
    }

    pub(crate) fn len(&self) -> usize {
        self.transforms.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.transforms.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::RenderMeshSnapshot;
    use crate::core::framework::scene::Mobility;
    use crate::core::math::{Transform, Vec3, Vec4};
    use crate::core::resource::{ResourceHandle, ResourceId};

    use super::ViewportMotionVectorObjectHistory;

    #[test]
    fn object_motion_history_keeps_only_dynamic_mesh_transforms() {
        let dynamic_transform = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let static_transform = Transform::from_translation(Vec3::new(4.0, 5.0, 6.0));

        let history = ViewportMotionVectorObjectHistory::from_meshes(&[
            test_mesh(1, Mobility::Dynamic, dynamic_transform),
            test_mesh(2, Mobility::Static, static_transform),
        ]);

        assert_eq!(history.len(), 1);
        assert_eq!(history.transform(1), Some(&dynamic_transform));
        assert_eq!(history.transform(2), None);
    }

    #[test]
    fn object_motion_history_counts_current_matches_against_previous_dynamic_history() {
        let previous_transform = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let current_transform = Transform::from_translation(Vec3::new(4.0, 5.0, 6.0));
        let previous = ViewportMotionVectorObjectHistory::from_meshes(&[
            test_mesh(1, Mobility::Dynamic, previous_transform),
            test_mesh(2, Mobility::Dynamic, previous_transform),
        ]);
        let current = ViewportMotionVectorObjectHistory::from_meshes(&[
            test_mesh(2, Mobility::Dynamic, current_transform),
            test_mesh(3, Mobility::Dynamic, current_transform),
            test_mesh(4, Mobility::Static, current_transform),
        ]);

        assert_eq!(current.len(), 2);
        assert_eq!(current.matched_transform_count(Some(&previous)), 1);
        assert_eq!(current.missing_transform_count(Some(&previous)), 1);
        assert_eq!(current.matched_transform_count(None), 0);
        assert_eq!(current.missing_transform_count(None), 2);
    }

    fn test_mesh(node_id: u64, mobility: Mobility, transform: Transform) -> RenderMeshSnapshot {
        RenderMeshSnapshot {
            node_id,
            transform,
            model: ResourceHandle::new(ResourceId::from_stable_label("tests/model")),
            mesh: None,
            material: ResourceHandle::new(ResourceId::from_stable_label("tests/material")),
            morph_weights: Vec::new(),
            tint: Vec4::ONE,
            mobility,
            render_layer_mask: 1,
        }
    }
}
