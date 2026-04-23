use crate::core::framework::render::{
    RenderExtractContext, RenderExtractProducer, RenderFrameExtract, RenderSkeletalPoseExtract,
};

use crate::scene::LevelSystem;

impl RenderExtractProducer for LevelSystem {
    fn build_render_frame_extract(&self, context: &RenderExtractContext) -> RenderFrameExtract {
        let cached_poses = self.animation_poses();
        self.with_world(|world| {
            let mut extract = world.build_render_frame_extract(context);
            if cached_poses.is_empty() {
                return extract;
            }

            let mut animation_poses = cached_poses
                .into_iter()
                .filter_map(|(entity, pose)| {
                    world
                        .find_node(entity)
                        .filter(|node| node.mesh.is_some())
                        .and_then(|_| world.animation_skeleton(entity))
                        .map(|skeleton| RenderSkeletalPoseExtract {
                            entity,
                            skeleton: skeleton.skeleton.id(),
                            pose,
                        })
                })
                .collect::<Vec<_>>();
            animation_poses.sort_by_key(|entry| entry.entity);
            extract.animation_poses = animation_poses;
            extract
        })
    }
}
