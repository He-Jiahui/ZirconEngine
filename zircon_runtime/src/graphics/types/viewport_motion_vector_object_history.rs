use std::collections::BTreeMap;

use crate::core::framework::animation::AnimationPoseOutput;
use crate::core::framework::render::{RenderMeshSnapshot, RenderSkeletalPoseExtract};
use crate::core::framework::scene::{EntityId, Mobility};
use crate::core::math::Transform;
use crate::core::resource::ResourceId;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ViewportMotionVectorObjectHistory {
    transforms: BTreeMap<EntityId, Transform>,
    skinned_poses: BTreeMap<EntityId, ViewportMotionVectorSkinnedPoseHistory>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ViewportMotionVectorSkinnedPoseHistory {
    skeleton: ResourceId,
    pose: AnimationPoseOutput,
    morph_weights: Vec<f32>,
}

impl ViewportMotionVectorSkinnedPoseHistory {
    pub(crate) fn skeleton(&self) -> ResourceId {
        self.skeleton
    }

    pub(crate) fn pose(&self) -> &AnimationPoseOutput {
        &self.pose
    }

    pub(crate) fn morph_weights(&self) -> &[f32] {
        &self.morph_weights
    }
}

impl ViewportMotionVectorObjectHistory {
    pub(crate) fn from_meshes(meshes: &[RenderMeshSnapshot]) -> Self {
        Self::from_meshes_and_animation_poses(meshes, &[])
    }

    pub(crate) fn from_meshes_and_animation_poses(
        meshes: &[RenderMeshSnapshot],
        animation_poses: &[RenderSkeletalPoseExtract],
    ) -> Self {
        let dynamic_meshes = meshes
            .iter()
            .filter(|mesh| mesh.mobility == Mobility::Dynamic)
            .collect::<Vec<_>>();
        let transforms = dynamic_meshes
            .iter()
            .map(|mesh| (mesh.node_id, mesh.transform))
            .collect::<BTreeMap<_, _>>();
        let morph_weights = dynamic_meshes
            .iter()
            .map(|mesh| (mesh.node_id, mesh.morph_weights.clone()))
            .collect::<BTreeMap<_, _>>();
        let skinned_poses = animation_poses
            .iter()
            .filter(|pose| transforms.contains_key(&pose.entity))
            .map(|pose| {
                (
                    pose.entity,
                    ViewportMotionVectorSkinnedPoseHistory {
                        skeleton: pose.skeleton,
                        pose: pose.pose.clone(),
                        morph_weights: morph_weights.get(&pose.entity).cloned().unwrap_or_default(),
                    },
                )
            })
            .collect();
        Self {
            transforms,
            skinned_poses,
        }
    }

    pub(crate) fn transform(&self, entity: EntityId) -> Option<&Transform> {
        self.transforms.get(&entity)
    }

    pub(crate) fn skinned_pose(
        &self,
        entity: EntityId,
    ) -> Option<&ViewportMotionVectorSkinnedPoseHistory> {
        self.skinned_poses.get(&entity)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn skinned_pose_count(&self) -> usize {
        self.skinned_poses.len()
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
    use crate::core::framework::animation::{AnimationPoseOutput, AnimationPoseSource};
    use crate::core::framework::render::{RenderMeshSnapshot, RenderSkeletalPoseExtract};
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
    fn object_motion_history_keeps_dynamic_skinned_pose_sideband() {
        let dynamic_transform = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let static_transform = Transform::from_translation(Vec3::new(4.0, 5.0, 6.0));
        let skeleton = ResourceId::from_stable_label("tests/skeleton");
        let pose = AnimationPoseOutput {
            source: AnimationPoseSource::Graph,
            active_state: Some("Walk".to_string()),
            bones: Vec::new(),
        };

        let history = ViewportMotionVectorObjectHistory::from_meshes_and_animation_poses(
            &[
                test_mesh_with_morph_weights(
                    1,
                    Mobility::Dynamic,
                    dynamic_transform,
                    vec![0.0, 0.25],
                ),
                test_mesh(2, Mobility::Static, static_transform),
            ],
            &[
                RenderSkeletalPoseExtract {
                    entity: 1,
                    skeleton,
                    pose: pose.clone(),
                },
                RenderSkeletalPoseExtract {
                    entity: 2,
                    skeleton,
                    pose: pose.clone(),
                },
            ],
        );

        assert_eq!(history.skinned_pose_count(), 1);
        let skinned_pose = history.skinned_pose(1).expect("dynamic skinned pose");
        assert_eq!(skinned_pose.skeleton(), skeleton);
        assert_eq!(skinned_pose.pose(), &pose);
        assert_eq!(skinned_pose.morph_weights(), &[0.0, 0.25]);
        assert!(history.skinned_pose(2).is_none());
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
        test_mesh_with_morph_weights(node_id, mobility, transform, Vec::new())
    }

    fn test_mesh_with_morph_weights(
        node_id: u64,
        mobility: Mobility,
        transform: Transform,
        morph_weights: Vec<f32>,
    ) -> RenderMeshSnapshot {
        RenderMeshSnapshot {
            node_id,
            stable_instance_key: node_id << 16,
            transform_revision: 0,
            transform,
            model: ResourceHandle::new(ResourceId::from_stable_label("tests/model")),
            mesh: None,
            material: ResourceHandle::new(ResourceId::from_stable_label("tests/material")),
            mesh_lod: None,
            morph_weights,
            tint: Vec4::ONE,
            mobility,
            static_state: Default::default(),
            render_layer_mask: 1,
        }
    }
}
