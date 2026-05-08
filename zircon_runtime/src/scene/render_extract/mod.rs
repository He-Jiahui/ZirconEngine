use crate::core::framework::render::{
    RenderExtractContext, RenderExtractProducer, RenderFrameExtract, RenderWorldSnapshotHandle,
};

use crate::scene::world::World;

impl World {
    pub fn to_render_frame_extract(&self) -> RenderFrameExtract {
        let mut world = self.clone();
        world.build_prepared_render_frame_extract(&RenderExtractContext::new(
            RenderWorldSnapshotHandle::new(0),
            Default::default(),
        ))
    }

    pub(crate) fn build_prepared_render_frame_extract(
        &mut self,
        context: &RenderExtractContext,
    ) -> RenderFrameExtract {
        self.build_prepared_render_frame_extract_for_request(context.world, &context.request)
    }
}

impl RenderExtractProducer for World {
    fn build_render_frame_extract(&self, context: &RenderExtractContext) -> RenderFrameExtract {
        let mut world = self.clone();
        world.build_prepared_render_frame_extract(context)
    }
}
