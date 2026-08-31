use super::{RenderExtractContext, RenderFrameExtract};

pub trait RenderExtractProducer {
    fn build_render_frame_extract(&self, context: &RenderExtractContext) -> RenderFrameExtract;
}
