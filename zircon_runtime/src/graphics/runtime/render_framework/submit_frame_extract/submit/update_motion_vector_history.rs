use crate::graphics::{ViewportMotionVectorObjectHistory, ViewportRenderFrame};

use super::super::super::viewport_record::ViewportRecord;

pub(super) fn update_motion_vector_history_after_success(
    record: &mut ViewportRecord,
    frame: &ViewportRenderFrame,
) {
    record.replace_motion_vector_camera(frame.extract.view.camera.clone());
    record.replace_motion_vector_object_history(
        ViewportMotionVectorObjectHistory::from_meshes_and_animation_poses(
            frame.meshes(),
            &frame.extract.animation_poses,
        ),
    );
}

#[cfg(test)]
mod tests {
    use crate::core::framework::animation::{AnimationPoseOutput, AnimationPoseSource};
    use crate::core::framework::render::{
        RenderFrameExtract, RenderMeshSnapshot, RenderSkeletalPoseExtract,
        RenderViewportDescriptor, RenderWorldSnapshotHandle,
    };
    use crate::core::framework::scene::Mobility;
    use crate::core::math::{Transform, UVec2, Vec3, Vec4};
    use crate::core::resource::{ResourceHandle, ResourceId};
    use crate::graphics::runtime::render_framework::viewport_record::ViewportRecord;
    use crate::graphics::ViewportRenderFrame;
    use crate::scene::world::World;

    use super::update_motion_vector_history_after_success;

    #[test]
    fn successful_submit_records_dynamic_object_history_for_next_frame() {
        let dynamic_transform = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let static_transform = Transform::from_translation(Vec3::new(4.0, 5.0, 6.0));
        let mut extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(7),
            World::new().to_render_snapshot(),
        );
        extract.geometry.meshes = vec![
            test_mesh(42, Mobility::Dynamic, dynamic_transform),
            test_mesh(99, Mobility::Static, static_transform),
        ];
        let skeleton = ResourceId::from_stable_label("tests/skeleton");
        let pose = AnimationPoseOutput {
            source: AnimationPoseSource::Graph,
            active_state: Some("Walk".to_string()),
            bones: Vec::new(),
        };
        extract.animation_poses = vec![
            RenderSkeletalPoseExtract {
                entity: 42,
                skeleton,
                pose: pose.clone(),
            },
            RenderSkeletalPoseExtract {
                entity: 99,
                skeleton,
                pose: pose.clone(),
            },
        ];
        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64));
        let mut record = ViewportRecord::new(RenderViewportDescriptor::new(UVec2::new(64, 64)));

        update_motion_vector_history_after_success(&mut record, &frame);

        let history = record
            .motion_vector_object_history()
            .expect("object history should be recorded after a successful frame");
        assert_eq!(history.len(), 1);
        assert_eq!(history.transform(42), Some(&dynamic_transform));
        assert_eq!(history.transform(99), None);
        assert_eq!(history.skinned_pose_count(), 1);
        let skinned_pose = history.skinned_pose(42).expect("dynamic skinned pose");
        assert_eq!(skinned_pose.skeleton(), skeleton);
        assert_eq!(skinned_pose.pose(), &pose);
        assert!(history.skinned_pose(99).is_none());
        assert_eq!(
            record.motion_vector_camera(),
            Some(&frame.extract.view.camera)
        );
    }

    fn test_mesh(node_id: u64, mobility: Mobility, transform: Transform) -> RenderMeshSnapshot {
        RenderMeshSnapshot {
            node_id,
            stable_instance_key: node_id << 16,
            transform_revision: 0,
            transform,
            model: ResourceHandle::new(ResourceId::from_stable_label("tests/model")),
            mesh: None,
            material: ResourceHandle::new(ResourceId::from_stable_label("tests/material")),
            mesh_lod: None,
            morph_weights: Vec::new(),
            tint: Vec4::ONE,
            mobility,
            static_state: Default::default(),
            render_layer_mask: 1,
        }
    }
}
