use super::super::RenderCameraTargetKind;
use crate::core::math::UVec2;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderCameraTargetResolutionReport {
    pub target_kind: RenderCameraTargetKind,
    pub primary_target_size: UVec2,
    pub resolved_target_size: UVec2,
    pub effective_view_size: UVec2,
    pub effective_render_size: UVec2,
}

impl RenderCameraTargetResolutionReport {
    pub const fn new(
        target_kind: RenderCameraTargetKind,
        primary_target_size: UVec2,
        resolved_target_size: UVec2,
        effective_view_size: UVec2,
        effective_render_size: UVec2,
    ) -> Self {
        Self {
            target_kind,
            primary_target_size,
            resolved_target_size,
            effective_view_size,
            effective_render_size,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RenderCameraTargetWritebackStatus {
    #[default]
    NotRequested,
    PendingTargetDescriptor,
    ReadyForCopy,
    ReadyForConversion,
    SuppressedByCameraStack,
    SkippedDirectImport,
    Copied,
    Converted,
    BlockedFormatMismatch,
}

impl RenderCameraTargetWritebackStatus {
    pub const fn label(self) -> &'static str {
        match self {
            Self::NotRequested => "not_requested",
            Self::PendingTargetDescriptor => "pending_target_descriptor",
            Self::ReadyForCopy => "ready_for_copy",
            Self::ReadyForConversion => "ready_for_conversion",
            Self::SuppressedByCameraStack => "suppressed_by_camera_stack",
            Self::SkippedDirectImport => "skipped_direct_import",
            Self::Copied => "copied",
            Self::Converted => "converted",
            Self::BlockedFormatMismatch => "blocked_format_mismatch",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderCameraTargetWritebackReport {
    pub target_kind: RenderCameraTargetKind,
    pub status: RenderCameraTargetWritebackStatus,
    pub debug_marker_emitted: bool,
    pub conversion_debug_marker_emitted: bool,
    pub target_size: UVec2,
    pub copied_count: usize,
    pub converted_count: usize,
}

impl RenderCameraTargetWritebackReport {
    pub const fn new(
        target_kind: RenderCameraTargetKind,
        status: RenderCameraTargetWritebackStatus,
        debug_marker_emitted: bool,
        conversion_debug_marker_emitted: bool,
        target_size: UVec2,
        copied_count: usize,
        converted_count: usize,
    ) -> Self {
        Self {
            target_kind,
            status,
            debug_marker_emitted,
            conversion_debug_marker_emitted,
            target_size,
            copied_count,
            converted_count,
        }
    }

    pub fn not_requested(target_kind: RenderCameraTargetKind) -> Self {
        Self::new(
            target_kind,
            RenderCameraTargetWritebackStatus::NotRequested,
            false,
            false,
            UVec2::new(0, 0),
            0,
            0,
        )
    }

    pub const fn pending_target_descriptor(target_size: UVec2) -> Self {
        Self::new(
            RenderCameraTargetKind::Texture,
            RenderCameraTargetWritebackStatus::PendingTargetDescriptor,
            false,
            false,
            target_size,
            0,
            0,
        )
    }

    pub const fn ready_for_copy(target_size: UVec2) -> Self {
        Self::new(
            RenderCameraTargetKind::Texture,
            RenderCameraTargetWritebackStatus::ReadyForCopy,
            false,
            false,
            target_size,
            0,
            0,
        )
    }

    pub const fn ready_for_conversion(target_size: UVec2) -> Self {
        Self::new(
            RenderCameraTargetKind::Texture,
            RenderCameraTargetWritebackStatus::ReadyForConversion,
            false,
            false,
            target_size,
            0,
            0,
        )
    }

    pub const fn suppressed_by_camera_stack(target_size: UVec2) -> Self {
        Self::new(
            RenderCameraTargetKind::Texture,
            RenderCameraTargetWritebackStatus::SuppressedByCameraStack,
            false,
            false,
            target_size,
            0,
            0,
        )
    }

    pub const fn skipped_direct_import(target_size: UVec2) -> Self {
        Self::new(
            RenderCameraTargetKind::Texture,
            RenderCameraTargetWritebackStatus::SkippedDirectImport,
            false,
            false,
            target_size,
            0,
            0,
        )
    }

    pub const fn copied(target_size: UVec2) -> Self {
        Self::new(
            RenderCameraTargetKind::Texture,
            RenderCameraTargetWritebackStatus::Copied,
            true,
            false,
            target_size,
            1,
            0,
        )
    }

    pub const fn converted(target_size: UVec2) -> Self {
        Self::new(
            RenderCameraTargetKind::Texture,
            RenderCameraTargetWritebackStatus::Converted,
            false,
            true,
            target_size,
            0,
            1,
        )
    }

    pub const fn blocked_format_mismatch(target_size: UVec2) -> Self {
        Self::new(
            RenderCameraTargetKind::Texture,
            RenderCameraTargetWritebackStatus::BlockedFormatMismatch,
            false,
            false,
            target_size,
            0,
            0,
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RenderCameraTargetGraphImportStatus {
    #[default]
    NotRequested,
    PendingTargetDescriptor,
    ReadyForDirectImport,
    DirectImported,
    RequiresConversionWriteback,
    SuppressedByCameraStack,
    BlockedFormatMismatch,
}

impl RenderCameraTargetGraphImportStatus {
    pub const fn label(self) -> &'static str {
        match self {
            Self::NotRequested => "not_requested",
            Self::PendingTargetDescriptor => "pending_target_descriptor",
            Self::ReadyForDirectImport => "ready_for_direct_import",
            Self::DirectImported => "direct_imported",
            Self::RequiresConversionWriteback => "requires_conversion_writeback",
            Self::SuppressedByCameraStack => "suppressed_by_camera_stack",
            Self::BlockedFormatMismatch => "blocked_format_mismatch",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderCameraTargetGraphImportReport {
    pub target_kind: RenderCameraTargetKind,
    pub status: RenderCameraTargetGraphImportStatus,
    pub target_size: UVec2,
    pub direct_import_count: usize,
    pub conversion_writeback_count: usize,
    pub blocked_count: usize,
}

impl RenderCameraTargetGraphImportReport {
    pub const fn new(
        target_kind: RenderCameraTargetKind,
        status: RenderCameraTargetGraphImportStatus,
        target_size: UVec2,
        direct_import_count: usize,
        conversion_writeback_count: usize,
        blocked_count: usize,
    ) -> Self {
        Self {
            target_kind,
            status,
            target_size,
            direct_import_count,
            conversion_writeback_count,
            blocked_count,
        }
    }

    pub fn not_requested(target_kind: RenderCameraTargetKind) -> Self {
        Self::new(
            target_kind,
            RenderCameraTargetGraphImportStatus::NotRequested,
            UVec2::new(0, 0),
            0,
            0,
            0,
        )
    }

    pub const fn pending_target_descriptor(target_size: UVec2) -> Self {
        Self::new(
            RenderCameraTargetKind::Texture,
            RenderCameraTargetGraphImportStatus::PendingTargetDescriptor,
            target_size,
            0,
            0,
            0,
        )
    }

    pub const fn ready_for_direct_import(target_size: UVec2) -> Self {
        Self::new(
            RenderCameraTargetKind::Texture,
            RenderCameraTargetGraphImportStatus::ReadyForDirectImport,
            target_size,
            0,
            0,
            0,
        )
    }

    pub const fn direct_imported(target_size: UVec2) -> Self {
        Self::new(
            RenderCameraTargetKind::Texture,
            RenderCameraTargetGraphImportStatus::DirectImported,
            target_size,
            1,
            0,
            0,
        )
    }

    pub const fn requires_conversion_writeback(target_size: UVec2) -> Self {
        Self::new(
            RenderCameraTargetKind::Texture,
            RenderCameraTargetGraphImportStatus::RequiresConversionWriteback,
            target_size,
            0,
            1,
            0,
        )
    }

    pub const fn suppressed_by_camera_stack(target_size: UVec2) -> Self {
        Self::new(
            RenderCameraTargetKind::Texture,
            RenderCameraTargetGraphImportStatus::SuppressedByCameraStack,
            target_size,
            0,
            0,
            0,
        )
    }

    pub const fn blocked_format_mismatch(target_size: UVec2) -> Self {
        Self::new(
            RenderCameraTargetKind::Texture,
            RenderCameraTargetGraphImportStatus::BlockedFormatMismatch,
            target_size,
            0,
            0,
            1,
        )
    }
}
