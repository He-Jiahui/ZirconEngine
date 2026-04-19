use crate::core::framework::render::RenderFrameExtract;

use super::super::declarations::VisibilityContext;

impl VisibilityContext {
    pub fn from_extract(value: &RenderFrameExtract) -> Self {
        Self::from_extract_with_history(value, None)
    }
}
