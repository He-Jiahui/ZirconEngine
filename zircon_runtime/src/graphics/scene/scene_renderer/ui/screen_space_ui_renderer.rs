use super::text::ScreenSpaceUiTextSystem;

pub(crate) struct ScreenSpaceUiRenderer {
    pub(super) pipeline: wgpu::RenderPipeline,
    pub(super) text_system: ScreenSpaceUiTextSystem,
}
