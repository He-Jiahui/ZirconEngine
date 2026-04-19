use crate::core::framework::render::RenderFrameExtract;

use super::super::declarations::VisibilityContext;

impl From<&RenderFrameExtract> for VisibilityContext {
    fn from(value: &RenderFrameExtract) -> Self {
        VisibilityContext::from_extract(value)
    }
}
