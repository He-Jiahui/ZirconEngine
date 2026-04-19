use crate::core::framework::render::{RenderExtractContext, RenderExtractProducer, RenderFrameExtract};

use crate::scene::LevelSystem;

impl RenderExtractProducer for LevelSystem {
    fn build_render_frame_extract(&self, context: &RenderExtractContext) -> RenderFrameExtract {
        self.with_world(|world| world.build_render_frame_extract(context))
    }
}
