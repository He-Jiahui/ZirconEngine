//! Frame bundle produced by runtime UI without depending on the graphics implementation crate root.

use crate::core::framework::render::RenderFrameExtract;
use crate::core::math::UVec2;

use crate::ui::surface::UiRenderExtract;

#[derive(Clone, Debug)]
pub struct PublicRuntimeFrame {
    pub extract: RenderFrameExtract,
    pub viewport_size: UVec2,
    pub ui: Option<UiRenderExtract>,
}
