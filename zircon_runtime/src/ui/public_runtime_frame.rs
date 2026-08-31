//! Frame bundle produced by runtime UI without depending on the graphics implementation crate root.

use std::sync::Arc;

use crate::core::framework::render::{RenderFrameExtract, UiRenderSubmission};
use crate::core::math::UVec2;

#[derive(Clone, Debug)]
pub(crate) struct PublicRuntimeFrame {
    pub extract: RenderFrameExtract,
    pub viewport_size: UVec2,
    pub ui: Option<Arc<UiRenderSubmission>>,
}
