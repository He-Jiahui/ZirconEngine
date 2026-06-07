use crate::core::framework::render::{RenderCameraTarget, RenderCameraTargetKind};
use crate::core::math::UVec2;
use crate::core::resource::{ResourceHandle, TextureMarker};

pub(crate) const FRAMEWORK_OUTPUT_FORMAT_LABEL: &str = "rgba8unorm_srgb";
pub(crate) const LINEAR_OUTPUT_FORMAT_LABEL: &str = "rgba8unorm";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ViewportRenderOutputTarget {
    #[default]
    PrimarySurface,
    Texture {
        handle: ResourceHandle<TextureMarker>,
        size: UVec2,
    },
    Headless {
        size: UVec2,
    },
}

impl ViewportRenderOutputTarget {
    pub(crate) fn from_camera_target(target: &RenderCameraTarget, resolved_size: UVec2) -> Self {
        match target {
            RenderCameraTarget::PrimarySurface => Self::PrimarySurface,
            RenderCameraTarget::Texture(handle) => Self::Texture {
                handle: *handle,
                size: resolved_size,
            },
            RenderCameraTarget::Headless { .. } => Self::Headless {
                size: resolved_size,
            },
        }
    }

    pub(crate) fn kind(self) -> RenderCameraTargetKind {
        match self {
            Self::PrimarySurface => RenderCameraTargetKind::PrimarySurface,
            Self::Texture { .. } => RenderCameraTargetKind::Texture,
            Self::Headless { .. } => RenderCameraTargetKind::Headless,
        }
    }

    pub(crate) fn texture_handle(self) -> Option<ResourceHandle<TextureMarker>> {
        match self {
            Self::Texture { handle, .. } => Some(handle),
            Self::PrimarySurface | Self::Headless { .. } => None,
        }
    }

    pub(crate) fn size(self) -> Option<UVec2> {
        match self {
            Self::Texture { size, .. } | Self::Headless { size } => Some(size),
            Self::PrimarySurface => None,
        }
    }

    pub(crate) fn writeback_plan(
        self,
        target_format: Option<&str>,
    ) -> ViewportTextureWritebackPlan {
        let Self::Texture { handle, size } = self else {
            return ViewportTextureWritebackPlan::not_requested(self.kind());
        };
        let Some(target_format) = target_format else {
            return ViewportTextureWritebackPlan::pending_descriptor(handle, size);
        };
        if target_format
            .trim()
            .eq_ignore_ascii_case(FRAMEWORK_OUTPUT_FORMAT_LABEL)
        {
            return ViewportTextureWritebackPlan::ready(handle, size);
        }
        if target_format
            .trim()
            .eq_ignore_ascii_case(LINEAR_OUTPUT_FORMAT_LABEL)
        {
            return ViewportTextureWritebackPlan::ready_for_conversion(handle, size);
        }
        ViewportTextureWritebackPlan::blocked_format(
            handle,
            size,
            target_format,
            FRAMEWORK_OUTPUT_FORMAT_LABEL,
        )
    }

    pub(crate) fn graph_import_plan(
        self,
        target_format: Option<&str>,
    ) -> ViewportTextureGraphImportPlan {
        let Self::Texture { handle, size } = self else {
            return ViewportTextureGraphImportPlan::not_requested(self.kind());
        };
        let Some(target_format) = target_format else {
            return ViewportTextureGraphImportPlan::pending_descriptor(handle, size);
        };
        if target_format
            .trim()
            .eq_ignore_ascii_case(FRAMEWORK_OUTPUT_FORMAT_LABEL)
        {
            return ViewportTextureGraphImportPlan::ready_for_direct_import(handle, size);
        }
        if target_format
            .trim()
            .eq_ignore_ascii_case(LINEAR_OUTPUT_FORMAT_LABEL)
        {
            return ViewportTextureGraphImportPlan::requires_conversion_writeback(handle, size);
        }
        ViewportTextureGraphImportPlan::blocked_format(
            handle,
            size,
            target_format,
            FRAMEWORK_OUTPUT_FORMAT_LABEL,
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ViewportTextureWritebackPlan {
    target_kind: RenderCameraTargetKind,
    status: ViewportTextureWritebackStatus,
    texture: Option<ResourceHandle<TextureMarker>>,
    size: Option<UVec2>,
    target_format: Option<String>,
    source_format: Option<String>,
}

impl ViewportTextureWritebackPlan {
    fn not_requested(target_kind: RenderCameraTargetKind) -> Self {
        Self {
            target_kind,
            status: ViewportTextureWritebackStatus::NotRequested,
            texture: None,
            size: None,
            target_format: None,
            source_format: None,
        }
    }

    fn pending_descriptor(texture: ResourceHandle<TextureMarker>, size: UVec2) -> Self {
        Self {
            target_kind: RenderCameraTargetKind::Texture,
            status: ViewportTextureWritebackStatus::PendingTargetDescriptor,
            texture: Some(texture),
            size: Some(size),
            target_format: None,
            source_format: Some(FRAMEWORK_OUTPUT_FORMAT_LABEL.to_string()),
        }
    }

    fn ready(texture: ResourceHandle<TextureMarker>, size: UVec2) -> Self {
        Self {
            target_kind: RenderCameraTargetKind::Texture,
            status: ViewportTextureWritebackStatus::ReadyForSrgbCopy,
            texture: Some(texture),
            size: Some(size),
            target_format: Some(FRAMEWORK_OUTPUT_FORMAT_LABEL.to_string()),
            source_format: Some(FRAMEWORK_OUTPUT_FORMAT_LABEL.to_string()),
        }
    }

    fn ready_for_conversion(texture: ResourceHandle<TextureMarker>, size: UVec2) -> Self {
        Self {
            target_kind: RenderCameraTargetKind::Texture,
            status: ViewportTextureWritebackStatus::ReadyForConversion,
            texture: Some(texture),
            size: Some(size),
            target_format: Some(LINEAR_OUTPUT_FORMAT_LABEL.to_string()),
            source_format: Some(FRAMEWORK_OUTPUT_FORMAT_LABEL.to_string()),
        }
    }

    fn blocked_format(
        texture: ResourceHandle<TextureMarker>,
        size: UVec2,
        target_format: &str,
        source_format: &str,
    ) -> Self {
        Self {
            target_kind: RenderCameraTargetKind::Texture,
            status: ViewportTextureWritebackStatus::BlockedFormatMismatch,
            texture: Some(texture),
            size: Some(size),
            target_format: Some(target_format.to_string()),
            source_format: Some(source_format.to_string()),
        }
    }

    pub(crate) fn target_kind(&self) -> RenderCameraTargetKind {
        self.target_kind
    }

    pub(crate) fn status(&self) -> ViewportTextureWritebackStatus {
        self.status
    }

    #[cfg(test)]
    pub(crate) fn texture(&self) -> Option<ResourceHandle<TextureMarker>> {
        self.texture
    }

    pub(crate) fn size(&self) -> Option<UVec2> {
        self.size
    }

    #[cfg(test)]
    pub(crate) fn target_format(&self) -> Option<&str> {
        self.target_format.as_deref()
    }

    #[cfg(test)]
    pub(crate) fn source_format(&self) -> Option<&str> {
        self.source_format.as_deref()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ViewportTextureWritebackStatus {
    #[default]
    NotRequested,
    PendingTargetDescriptor,
    ReadyForSrgbCopy,
    ReadyForConversion,
    BlockedFormatMismatch,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ViewportTextureGraphImportPlan {
    target_kind: RenderCameraTargetKind,
    status: ViewportTextureGraphImportStatus,
    texture: Option<ResourceHandle<TextureMarker>>,
    size: Option<UVec2>,
    target_format: Option<String>,
    source_format: Option<String>,
}

impl ViewportTextureGraphImportPlan {
    fn not_requested(target_kind: RenderCameraTargetKind) -> Self {
        Self {
            target_kind,
            status: ViewportTextureGraphImportStatus::NotRequested,
            texture: None,
            size: None,
            target_format: None,
            source_format: None,
        }
    }

    fn pending_descriptor(texture: ResourceHandle<TextureMarker>, size: UVec2) -> Self {
        Self {
            target_kind: RenderCameraTargetKind::Texture,
            status: ViewportTextureGraphImportStatus::PendingTargetDescriptor,
            texture: Some(texture),
            size: Some(size),
            target_format: None,
            source_format: Some(FRAMEWORK_OUTPUT_FORMAT_LABEL.to_string()),
        }
    }

    fn ready_for_direct_import(texture: ResourceHandle<TextureMarker>, size: UVec2) -> Self {
        Self {
            target_kind: RenderCameraTargetKind::Texture,
            status: ViewportTextureGraphImportStatus::ReadyForDirectImport,
            texture: Some(texture),
            size: Some(size),
            target_format: Some(FRAMEWORK_OUTPUT_FORMAT_LABEL.to_string()),
            source_format: Some(FRAMEWORK_OUTPUT_FORMAT_LABEL.to_string()),
        }
    }

    fn requires_conversion_writeback(texture: ResourceHandle<TextureMarker>, size: UVec2) -> Self {
        Self {
            target_kind: RenderCameraTargetKind::Texture,
            status: ViewportTextureGraphImportStatus::RequiresConversionWriteback,
            texture: Some(texture),
            size: Some(size),
            target_format: Some(LINEAR_OUTPUT_FORMAT_LABEL.to_string()),
            source_format: Some(FRAMEWORK_OUTPUT_FORMAT_LABEL.to_string()),
        }
    }

    fn blocked_format(
        texture: ResourceHandle<TextureMarker>,
        size: UVec2,
        target_format: &str,
        source_format: &str,
    ) -> Self {
        Self {
            target_kind: RenderCameraTargetKind::Texture,
            status: ViewportTextureGraphImportStatus::BlockedFormatMismatch,
            texture: Some(texture),
            size: Some(size),
            target_format: Some(target_format.to_string()),
            source_format: Some(source_format.to_string()),
        }
    }

    pub(crate) fn target_kind(&self) -> RenderCameraTargetKind {
        self.target_kind
    }

    pub(crate) fn status(&self) -> ViewportTextureGraphImportStatus {
        self.status
    }

    #[cfg(test)]
    pub(crate) fn texture(&self) -> Option<ResourceHandle<TextureMarker>> {
        self.texture
    }

    pub(crate) fn size(&self) -> Option<UVec2> {
        self.size
    }

    #[cfg(test)]
    pub(crate) fn target_format(&self) -> Option<&str> {
        self.target_format.as_deref()
    }

    #[cfg(test)]
    pub(crate) fn source_format(&self) -> Option<&str> {
        self.source_format.as_deref()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ViewportTextureGraphImportStatus {
    #[default]
    NotRequested,
    PendingTargetDescriptor,
    ReadyForDirectImport,
    RequiresConversionWriteback,
    BlockedFormatMismatch,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::resource::ResourceId;

    #[test]
    fn output_target_writeback_plan_ignores_non_texture_targets() {
        let headless = ViewportRenderOutputTarget::Headless {
            size: UVec2::new(64, 32),
        };

        let plan = headless.writeback_plan(Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));

        assert_eq!(plan.target_kind(), RenderCameraTargetKind::Headless);
        assert_eq!(plan.status(), ViewportTextureWritebackStatus::NotRequested);
        assert_eq!(plan.texture(), None);
        assert_eq!(plan.size(), None);
    }

    #[test]
    fn output_target_writeback_plan_waits_for_target_descriptor() {
        let texture = texture_handle("tests/writeback/pending");
        let target = ViewportRenderOutputTarget::Texture {
            handle: texture,
            size: UVec2::new(128, 72),
        };

        let plan = target.writeback_plan(None);

        assert_eq!(
            plan.status(),
            ViewportTextureWritebackStatus::PendingTargetDescriptor
        );
        assert_eq!(plan.texture(), Some(texture));
        assert_eq!(plan.size(), Some(UVec2::new(128, 72)));
        assert_eq!(plan.source_format(), Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));
        assert_eq!(plan.target_format(), None);
    }

    #[test]
    fn output_target_writeback_plan_accepts_matching_srgb_format() {
        let texture = texture_handle("tests/writeback/ready");
        let target = ViewportRenderOutputTarget::Texture {
            handle: texture,
            size: UVec2::new(128, 72),
        };

        let plan = target.writeback_plan(Some(" RGBA8UNORM_SRGB "));

        assert_eq!(
            plan.status(),
            ViewportTextureWritebackStatus::ReadyForSrgbCopy
        );
        assert_eq!(plan.texture(), Some(texture));
        assert_eq!(plan.size(), Some(UVec2::new(128, 72)));
        assert_eq!(plan.source_format(), Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));
        assert_eq!(plan.target_format(), Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));
    }

    #[test]
    fn output_target_writeback_plan_accepts_linear_rgba_target_for_conversion() {
        let texture = texture_handle("tests/writeback/linear");
        let target = ViewportRenderOutputTarget::Texture {
            handle: texture,
            size: UVec2::new(128, 72),
        };

        let plan = target.writeback_plan(Some("rgba8unorm"));

        assert_eq!(
            plan.status(),
            ViewportTextureWritebackStatus::ReadyForConversion
        );
        assert_eq!(plan.texture(), Some(texture));
        assert_eq!(plan.source_format(), Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));
        assert_eq!(plan.target_format(), Some("rgba8unorm"));
    }

    #[test]
    fn output_target_writeback_plan_blocks_unsupported_target_format() {
        let texture = texture_handle("tests/writeback/unsupported");
        let target = ViewportRenderOutputTarget::Texture {
            handle: texture,
            size: UVec2::new(128, 72),
        };

        let plan = target.writeback_plan(Some("rgba16float"));

        assert_eq!(
            plan.status(),
            ViewportTextureWritebackStatus::BlockedFormatMismatch
        );
        assert_eq!(plan.texture(), Some(texture));
        assert_eq!(plan.source_format(), Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));
        assert_eq!(plan.target_format(), Some("rgba16float"));
    }

    #[test]
    fn output_target_graph_import_plan_marks_srgb_texture_ready_for_direct_import() {
        let texture = texture_handle("tests/graph-import/srgb");
        let target = ViewportRenderOutputTarget::Texture {
            handle: texture,
            size: UVec2::new(128, 72),
        };

        let plan = target.graph_import_plan(Some("rgba8unorm_srgb"));

        assert_eq!(
            plan.status(),
            ViewportTextureGraphImportStatus::ReadyForDirectImport
        );
        assert_eq!(plan.texture(), Some(texture));
        assert_eq!(plan.size(), Some(UVec2::new(128, 72)));
        assert_eq!(plan.source_format(), Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));
        assert_eq!(plan.target_format(), Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));
    }

    #[test]
    fn output_target_graph_import_plan_keeps_linear_texture_on_conversion_writeback_path() {
        let texture = texture_handle("tests/graph-import/linear");
        let target = ViewportRenderOutputTarget::Texture {
            handle: texture,
            size: UVec2::new(128, 72),
        };

        let plan = target.graph_import_plan(Some("rgba8unorm"));

        assert_eq!(
            plan.status(),
            ViewportTextureGraphImportStatus::RequiresConversionWriteback
        );
        assert_eq!(plan.texture(), Some(texture));
        assert_eq!(plan.source_format(), Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));
        assert_eq!(plan.target_format(), Some("rgba8unorm"));
    }

    #[test]
    fn output_target_graph_import_plan_blocks_unsupported_target_format() {
        let texture = texture_handle("tests/graph-import/unsupported");
        let target = ViewportRenderOutputTarget::Texture {
            handle: texture,
            size: UVec2::new(128, 72),
        };

        let plan = target.graph_import_plan(Some("rgba16float"));

        assert_eq!(
            plan.status(),
            ViewportTextureGraphImportStatus::BlockedFormatMismatch
        );
        assert_eq!(plan.texture(), Some(texture));
        assert_eq!(plan.target_format(), Some("rgba16float"));
    }

    fn texture_handle(label: &str) -> ResourceHandle<TextureMarker> {
        ResourceHandle::new(ResourceId::from_stable_label(label))
    }
}
