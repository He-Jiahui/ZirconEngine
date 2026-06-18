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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapturedFrame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub generation: u64,
    pub capture_report: RenderCaptureReport,
    pub graph_dump: Option<String>,
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
        Self {
            width,
            height,
            rgba,
            generation,
            capture_report,
            graph_dump,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captured_frame_new_defaults_to_primary_framework_offscreen_source() {
        let frame = CapturedFrame::new(16, 8, vec![0; 16 * 8 * 4], 3);

        assert_eq!(
            frame.capture_report.target_kind,
            RenderCameraTargetKind::PrimarySurface
        );
        assert_eq!(
            frame.capture_report.source,
            RenderCaptureSource::FrameworkOffscreen
        );
        assert_eq!(frame.capture_report.output_size, UVec2::new(16, 8));
        assert_eq!(frame.graph_dump, None);
    }

    #[test]
    fn texture_capture_report_distinguishes_direct_import_and_conversion_sources() {
        let size = UVec2::new(72, 40);

        let direct = RenderCaptureReport::texture_from_reports(
            size,
            RenderCameraTargetGraphImportReport::direct_imported(size),
            RenderCameraTargetWritebackReport::skipped_direct_import(size),
        );
        let converted = RenderCaptureReport::texture_from_reports(
            size,
            RenderCameraTargetGraphImportReport::requires_conversion_writeback(size),
            RenderCameraTargetWritebackReport::converted(size),
        );

        assert_eq!(direct.source, RenderCaptureSource::TextureDirectGraphImport);
        assert_eq!(
            direct.graph_import_status,
            RenderCameraTargetGraphImportStatus::DirectImported
        );
        assert_eq!(
            converted.source,
            RenderCaptureSource::TextureWritebackConversion
        );
        assert_eq!(
            converted.writeback_status,
            RenderCameraTargetWritebackStatus::Converted
        );
    }
}
