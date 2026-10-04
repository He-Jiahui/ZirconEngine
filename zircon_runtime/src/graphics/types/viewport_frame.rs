//! 完成后的离屏视口帧包含 CPU RGBA 数据与对应的捕获回执。
use crate::core::framework::render::RenderCaptureReport;

/// 已完成读回的视口帧；rgba 按 width × height × 4 组织，generation 与捕获回执对齐。
#[derive(Clone, Debug)]
pub struct ViewportFrame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub generation: u64,
    pub capture_report: RenderCaptureReport,
}
