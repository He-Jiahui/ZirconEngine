use crate::core::framework::render::RenderCaptureReport;

#[derive(Clone, Debug)]
pub struct ViewportFrame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub generation: u64,
    pub capture_report: RenderCaptureReport,
}
