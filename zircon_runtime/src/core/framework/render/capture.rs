use std::sync::Arc;

use crate::core::math::UVec2;

use super::{
    RenderCameraTargetGraphImportReport, RenderCameraTargetGraphImportStatus,
    RenderCameraTargetKind, RenderCameraTargetWritebackReport, RenderCameraTargetWritebackStatus,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RenderCaptureSource {
    #[default]
    None,
    FrameworkOffscreen,
    TextureDirectGraphImport,
    TextureWritebackConversion,
    TextureWritebackCopy,
}

impl RenderCaptureSource {
    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::FrameworkOffscreen => "framework_offscreen",
            Self::TextureDirectGraphImport => "texture_direct_graph_import",
            Self::TextureWritebackConversion => "texture_writeback_conversion",
            Self::TextureWritebackCopy => "texture_writeback_copy",
        }
    }
}

/// 记录捕获像素来自离屏、直接导入还是纹理回写，供诊断和导出解释结果来源。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderCaptureReport {
    pub target_kind: RenderCameraTargetKind,
    pub source: RenderCaptureSource,
    pub output_size: UVec2,
    pub graph_import_status: RenderCameraTargetGraphImportStatus,
    pub writeback_status: RenderCameraTargetWritebackStatus,
}

impl RenderCaptureReport {
    pub const fn new(
        target_kind: RenderCameraTargetKind,
        source: RenderCaptureSource,
        output_size: UVec2,
        graph_import_status: RenderCameraTargetGraphImportStatus,
        writeback_status: RenderCameraTargetWritebackStatus,
    ) -> Self {
        Self {
            target_kind,
            source,
            output_size,
            graph_import_status,
            writeback_status,
        }
    }

    pub fn not_captured(target_kind: RenderCameraTargetKind) -> Self {
        Self::new(
            target_kind,
            RenderCaptureSource::None,
            UVec2::new(0, 0),
            RenderCameraTargetGraphImportStatus::NotRequested,
            RenderCameraTargetWritebackStatus::NotRequested,
        )
    }

    pub fn framework_offscreen(target_kind: RenderCameraTargetKind, output_size: UVec2) -> Self {
        Self::new(
            target_kind,
            RenderCaptureSource::FrameworkOffscreen,
            output_size,
            RenderCameraTargetGraphImportStatus::NotRequested,
            RenderCameraTargetWritebackStatus::NotRequested,
        )
    }

    pub fn texture_from_reports(
        output_size: UVec2,
        graph_import: RenderCameraTargetGraphImportReport,
        writeback: RenderCameraTargetWritebackReport,
    ) -> Self {
        let source = match (graph_import.status, writeback.status) {
            (
                RenderCameraTargetGraphImportStatus::DirectImported,
                RenderCameraTargetWritebackStatus::SkippedDirectImport,
            ) => RenderCaptureSource::TextureDirectGraphImport,
            (_, RenderCameraTargetWritebackStatus::Converted) => {
                RenderCaptureSource::TextureWritebackConversion
            }
            (_, RenderCameraTargetWritebackStatus::Copied) => {
                RenderCaptureSource::TextureWritebackCopy
            }
            _ => RenderCaptureSource::FrameworkOffscreen,
        };
        Self::new(
            RenderCameraTargetKind::Texture,
            source,
            output_size,
            graph_import.status,
            writeback.status,
        )
    }
}

/// 已读回的显示编码 RGBA8 帧；图转储和性能报告只随显式捕获附带。
/// 普通 GPU 视口展示使用 RenderViewportProduct，不应依赖每帧读回此数据。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapturedFrame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub generation: u64,
    pub capture_report: RenderCaptureReport,
    pub graph_dump: Option<Arc<str>>,
    pub frame_profile_json: Option<String>,
}

/// A linear, pre-output-transfer scene-color capture from a completed frame.
///
/// This is intentionally a distinct product from [`CapturedFrame`]: its texels
/// remain RGBA16F-derived linear values instead of display-encoded RGBA8 bytes.
#[derive(Clone, Debug, PartialEq)]
pub struct CapturedHdrFrame {
    pub width: u32,
    pub height: u32,
    pub rgba16f: Vec<[f32; 4]>,
    pub generation: u64,
    pub capture_report: RenderCaptureReport,
}

impl CapturedHdrFrame {
    pub fn with_capture_report(
        width: u32,
        height: u32,
        rgba16f: Vec<[f32; 4]>,
        generation: u64,
        capture_report: RenderCaptureReport,
    ) -> Self {
        Self {
            width,
            height,
            rgba16f,
            generation,
            capture_report,
        }
    }
}

impl CapturedFrame {
    pub fn new(width: u32, height: u32, rgba: Vec<u8>, generation: u64) -> Self {
        let output_size = UVec2::new(width, height);
        Self::with_capture_report(
            width,
            height,
            rgba,
            generation,
            RenderCaptureReport::framework_offscreen(
                RenderCameraTargetKind::PrimarySurface,
                output_size,
            ),
        )
    }

    pub fn with_capture_report(
        width: u32,
        height: u32,
        rgba: Vec<u8>,
        generation: u64,
        capture_report: RenderCaptureReport,
    ) -> Self {
        Self::with_capture_report_and_graph_dump(
            width,
            height,
            rgba,
            generation,
            capture_report,
            None,
        )
    }

    pub fn with_capture_report_and_graph_dump(
        width: u32,
        height: u32,
        rgba: Vec<u8>,
        generation: u64,
        capture_report: RenderCaptureReport,
        graph_dump: Option<String>,
    ) -> Self {
        Self::with_capture_report_graph_dump_and_frame_profile_json(
            width,
            height,
            rgba,
            generation,
            capture_report,
            graph_dump,
            None,
        )
    }

    pub fn with_capture_report_graph_dump_and_frame_profile_json(
        width: u32,
        height: u32,
        rgba: Vec<u8>,
        generation: u64,
        capture_report: RenderCaptureReport,
        graph_dump: Option<String>,
        frame_profile_json: Option<String>,
    ) -> Self {
        Self {
            width,
            height,
            rgba,
            generation,
            capture_report,
            graph_dump: graph_dump.map(Arc::from),
            frame_profile_json,
        }
    }
}

#[cfg(test)]
#[path = "tests/capture.rs"]
mod tests;
