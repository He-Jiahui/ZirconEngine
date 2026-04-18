use zircon_framework::render::{
    RenderExtractContext, RenderExtractProducer, RenderFrameExtract, RenderWorldSnapshotHandle,
};

use crate::{level_system::LevelSystem, World};

impl World {
    pub fn to_render_frame_extract(&self) -> RenderFrameExtract {
        RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(0),
            self.to_render_snapshot(),
        )
    }
}

impl RenderExtractProducer for World {
    fn build_render_frame_extract(&self, context: &RenderExtractContext) -> RenderFrameExtract {
        RenderFrameExtract::from_snapshot(
            context.world,
            self.build_viewport_render_packet(&context.request),
        )
    }
}

impl RenderExtractProducer for LevelSystem {
    fn build_render_frame_extract(&self, context: &RenderExtractContext) -> RenderFrameExtract {
        self.with_world(|world| world.build_render_frame_extract(context))
    }
}
