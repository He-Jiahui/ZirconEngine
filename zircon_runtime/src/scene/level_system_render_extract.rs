use crate::core::framework::render::{
    RenderExtractContext, RenderExtractProducer, RenderFrameExtract, RenderSkeletalPoseExtract,
};

use crate::scene::LevelSystem;

impl RenderExtractProducer for LevelSystem {
    fn build_render_frame_extract(&self, context: &RenderExtractContext) -> RenderFrameExtract {
        let frame_state = self.frame_state_snapshot();
        let candidate_entities = frame_state
            .animation_poses()
            .keys()
            .copied()
            .collect::<Vec<_>>();
        let (mut extract, skeletons) = self.with_world_mut(|world| {
            let extract = world.build_prepared_render_frame_extract(context);
            if frame_state.world_generation() != world.world_generation()
                || candidate_entities.is_empty()
            {
                return (extract, Vec::new());
            }

            let skeletons = candidate_entities
                .iter()
                .filter_map(|entity| {
                    world
                        .find_node(*entity)
                        .filter(|node| node.mesh.is_some())
                        .and_then(|_| world.animation_skeleton(*entity))
                        .map(|skeleton| (*entity, skeleton.skeleton.id()))
                })
                .collect::<Vec<_>>();
            (extract, skeletons)
        });

        extract.animation_poses = skeletons
            .into_iter()
            .filter_map(|(entity, skeleton)| {
                frame_state
                    .animation_poses()
                    .get(&entity)
                    .map(|pose| RenderSkeletalPoseExtract {
                        entity,
                        skeleton,
                        pose: pose.clone(),
                    })
            })
            .collect();
        extract
    }
}
