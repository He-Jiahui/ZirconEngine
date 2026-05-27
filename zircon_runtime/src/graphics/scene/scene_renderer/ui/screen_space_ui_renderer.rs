use super::text::{ScreenSpaceUiTextPrepareReport, ScreenSpaceUiTextSystem};

pub(crate) struct ScreenSpaceUiRenderer {
    pub(super) pipeline: wgpu::RenderPipeline,
    pub(super) text_system: ScreenSpaceUiTextSystem,
    pub(super) last_text_prepare_report: ScreenSpaceUiTextPrepareReport,
}
