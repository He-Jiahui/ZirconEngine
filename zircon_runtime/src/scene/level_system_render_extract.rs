use crate::core::framework::render::{
    RenderExtractContext, RenderExtractProducer, RenderFrameExtract, RenderSkeletalPoseExtract,
};

use crate::scene::LevelSystem;

impl RenderExtractProducer for LevelSystem {
    fn build_render_frame_extract(&self, context: &RenderExtractContext) -> RenderFrameExtract {
        let frame_state = self.frame_state_snapshot();
        let (mut extract, animation_poses) = self.with_world_mut(|world| {
            let extract = world.build_prepared_render_frame_extract(context);
            if frame_state.world_generation() != world.world_generation()
                || frame_state.animation_poses().is_empty()
            {
                return (extract, Vec::new());
            }

            let animation_poses = frame_state
                .animation_poses()
                .iter()
                .filter_map(|(&entity, pose)| {
                    world
                        .find_node(entity)
                        .filter(|node| node.mesh.is_some())
                        .and_then(|_| world.animation_skeleton(entity))
                        .map(|skeleton| RenderSkeletalPoseExtract {
                            entity,
                            skeleton: skeleton.skeleton.id(),
                            pose: pose.clone(),
                        })
                })
                .collect();
            (extract, animation_poses)
        });

        extract.animation_poses = animation_poses.into();
        extract
    }
}
