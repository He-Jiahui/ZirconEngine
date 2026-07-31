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
        format: &'static str,
    },
    Headless {
        size: UVec2,
    },
}

impl ViewportRenderOutputTarget {
    pub(crate) fn from_camera_target(
        target: &RenderCameraTarget,
        resolved_size: UVec2,
        texture_format: Option<&'static str>,
    ) -> Self {
        match target {
            RenderCameraTarget::PrimarySurface => Self::PrimarySurface,
            RenderCameraTarget::Texture(handle) => Self::Texture {
                handle: *handle,
                size: resolved_size,
                format: texture_format
                    .expect("texture camera target must carry a resolved texture format"),
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

    pub(crate) fn texture_format(self) -> Option<&'static str> {
        match self {
            Self::Texture { format, .. } => Some(format),
            Self::PrimarySurface | Self::Headless { .. } => None,
        }
    }

    pub(crate) fn writeback_plan(
        self,
        target_format: Option<&str>,
    ) -> ViewportTextureWritebackPlan {
        let Self::Texture {
            handle,
            size,
            format,
        } = self
        else {
            return ViewportTextureWritebackPlan::not_requested(self.kind());
        };
        let Some(target_format) = target_format else {
            return ViewportTextureWritebackPlan::pending_descriptor(handle, size, format);
        };
        if !format_label_matches(target_format, format) {
            return ViewportTextureWritebackPlan::blocked_prepared_format_mismatch(
                handle,
                size,
                target_format,
                format,
            );
        }
        if format_label_matches(format, FRAMEWORK_OUTPUT_FORMAT_LABEL) {
            return ViewportTextureWritebackPlan::ready(handle, size);
        }
        if format_label_matches(format, LINEAR_OUTPUT_FORMAT_LABEL) {
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
        let Self::Texture {
            handle,
            size,
            format,
        } = self
        else {
            return ViewportTextureGraphImportPlan::not_requested(self.kind());
        };
        let Some(target_format) = target_format else {
            return ViewportTextureGraphImportPlan::pending_descriptor(handle, size, format);
        };
        if !format_label_matches(target_format, format) {
            return ViewportTextureGraphImportPlan::blocked_prepared_format_mismatch(
                handle,
                size,
                target_format,
                format,
            );
        }
        if format_label_matches(format, FRAMEWORK_OUTPUT_FORMAT_LABEL) {
            return ViewportTextureGraphImportPlan::ready_for_direct_import(handle, size);
        }
        if format_label_matches(format, LINEAR_OUTPUT_FORMAT_LABEL) {
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
    #[cfg(test)]
    texture: Option<ResourceHandle<TextureMarker>>,
    size: Option<UVec2>,
    #[cfg(test)]
    target_format: Option<String>,
    #[cfg(test)]
    expected_target_format: Option<String>,
    #[cfg(test)]
    source_format: Option<String>,
}

impl ViewportTextureWritebackPlan {
    fn not_requested(target_kind: RenderCameraTargetKind) -> Self {
        Self {
            target_kind,
            status: ViewportTextureWritebackStatus::NotRequested,
            #[cfg(test)]
            texture: None,
            size: None,
            #[cfg(test)]
            target_format: None,
            #[cfg(test)]
            expected_target_format: None,
            #[cfg(test)]
            source_format: None,
        }
    }

    fn pending_descriptor(
        _texture: ResourceHandle<TextureMarker>,
        size: UVec2,
        _target_format: &'static str,
    ) -> Self {
        Self {
            target_kind: RenderCameraTargetKind::Texture,
            status: ViewportTextureWritebackStatus::PendingTargetDescriptor,
            #[cfg(test)]
            texture: Some(_texture),
            size: Some(size),
            #[cfg(test)]
            target_format: Some(_target_format.to_string()),
            #[cfg(test)]
            expected_target_format: Some(_target_format.to_string()),
            #[cfg(test)]
            source_format: Some(FRAMEWORK_OUTPUT_FORMAT_LABEL.to_string()),
        }
    }

    fn ready(_texture: ResourceHandle<TextureMarker>, size: UVec2) -> Self {
        Self {
            target_kind: RenderCameraTargetKind::Texture,
            status: ViewportTextureWritebackStatus::ReadyForSrgbCopy,
            #[cfg(test)]
            texture: Some(_texture),
            size: Some(size),
            #[cfg(test)]
            target_format: Some(FRAMEWORK_OUTPUT_FORMAT_LABEL.to_string()),
            #[cfg(test)]
            expected_target_format: Some(FRAMEWORK_OUTPUT_FORMAT_LABEL.to_string()),
            #[cfg(test)]
            source_format: Some(FRAMEWORK_OUTPUT_FORMAT_LABEL.to_string()),
        }
    }

    fn ready_for_conversion(_texture: ResourceHandle<TextureMarker>, size: UVec2) -> Self {
        Self {
            target_kind: RenderCameraTargetKind::Texture,
            status: ViewportTextureWritebackStatus::ReadyForConversion,
            #[cfg(test)]
            texture: Some(_texture),
            size: Some(size),
            #[cfg(test)]
            target_format: Some(LINEAR_OUTPUT_FORMAT_LABEL.to_string()),
            #[cfg(test)]
            expected_target_format: Some(LINEAR_OUTPUT_FORMAT_LABEL.to_string()),
            #[cfg(test)]
            source_format: Some(FRAMEWORK_OUTPUT_FORMAT_LABEL.to_string()),
        }
    }

    fn blocked_prepared_format_mismatch(
        _texture: ResourceHandle<TextureMarker>,
        size: UVec2,
        _target_format: &str,
        _expected_target_format: &'static str,
    ) -> Self {
        Self {
            target_kind: RenderCameraTargetKind::Texture,
            status: ViewportTextureWritebackStatus::BlockedPreparedFormatMismatch,
            #[cfg(test)]
            texture: Some(_texture),
            size: Some(size),
            #[cfg(test)]
            target_format: Some(_target_format.trim().to_string()),
            #[cfg(test)]
            expected_target_format: Some(_expected_target_format.to_string()),
            #[cfg(test)]
            source_format: Some(FRAMEWORK_OUTPUT_FORMAT_LABEL.to_string()),
        }
    }

    fn blocked_format(
        _texture: ResourceHandle<TextureMarker>,
        size: UVec2,
        _target_format: &str,
        _source_format: &str,
    ) -> Self {
        Self {
            target_kind: RenderCameraTargetKind::Texture,
            status: ViewportTextureWritebackStatus::BlockedFormatMismatch,
            #[cfg(test)]
            texture: Some(_texture),
            size: Some(size),
            #[cfg(test)]
            target_format: Some(_target_format.to_string()),
            #[cfg(test)]
            expected_target_format: Some(_target_format.to_string()),
            #[cfg(test)]
            source_format: Some(_source_format.to_string()),
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
    pub(crate) fn expected_target_format(&self) -> Option<&str> {
        self.expected_target_format.as_deref()
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
    BlockedPreparedFormatMismatch,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ViewportTextureGraphImportPlan {
    target_kind: RenderCameraTargetKind,
    status: ViewportTextureGraphImportStatus,
    #[cfg(test)]
    texture: Option<ResourceHandle<TextureMarker>>,
    size: Option<UVec2>,
    #[cfg(test)]
    target_format: Option<String>,
    #[cfg(test)]
    expected_target_format: Option<String>,
    #[cfg(test)]
    source_format: Option<String>,
}

impl ViewportTextureGraphImportPlan {
    fn not_requested(target_kind: RenderCameraTargetKind) -> Self {
        Self {
            target_kind,
            status: ViewportTextureGraphImportStatus::NotRequested,
            #[cfg(test)]
            texture: None,
            size: None,
            #[cfg(test)]
            target_format: None,
            #[cfg(test)]
            expected_target_format: None,
            #[cfg(test)]
            source_format: None,
        }
    }

    fn pending_descriptor(
        _texture: ResourceHandle<TextureMarker>,
        size: UVec2,
        _target_format: &'static str,
    ) -> Self {
        Self {
            target_kind: RenderCameraTargetKind::Texture,
            status: ViewportTextureGraphImportStatus::PendingTargetDescriptor,
            #[cfg(test)]
            texture: Some(_texture),
            size: Some(size),
            #[cfg(test)]
            target_format: Some(_target_format.to_string()),
            #[cfg(test)]
            expected_target_format: Some(_target_format.to_string()),
            #[cfg(test)]
            source_format: Some(FRAMEWORK_OUTPUT_FORMAT_LABEL.to_string()),
        }
    }

    fn ready_for_direct_import(_texture: ResourceHandle<TextureMarker>, size: UVec2) -> Self {
        Self {
            target_kind: RenderCameraTargetKind::Texture,
            status: ViewportTextureGraphImportStatus::ReadyForDirectImport,
            #[cfg(test)]
            texture: Some(_texture),
            size: Some(size),
            #[cfg(test)]
            target_format: Some(FRAMEWORK_OUTPUT_FORMAT_LABEL.to_string()),
            #[cfg(test)]
            expected_target_format: Some(FRAMEWORK_OUTPUT_FORMAT_LABEL.to_string()),
            #[cfg(test)]
            source_format: Some(FRAMEWORK_OUTPUT_FORMAT_LABEL.to_string()),
        }
    }

    fn requires_conversion_writeback(_texture: ResourceHandle<TextureMarker>, size: UVec2) -> Self {
        Self {
            target_kind: RenderCameraTargetKind::Texture,
            status: ViewportTextureGraphImportStatus::RequiresConversionWriteback,
            #[cfg(test)]
            texture: Some(_texture),
            size: Some(size),
            #[cfg(test)]
            target_format: Some(LINEAR_OUTPUT_FORMAT_LABEL.to_string()),
            #[cfg(test)]
            expected_target_format: Some(LINEAR_OUTPUT_FORMAT_LABEL.to_string()),
            #[cfg(test)]
            source_format: Some(FRAMEWORK_OUTPUT_FORMAT_LABEL.to_string()),
        }
    }

    fn blocked_prepared_format_mismatch(
        _texture: ResourceHandle<TextureMarker>,
        size: UVec2,
        _target_format: &str,
        _expected_target_format: &'static str,
    ) -> Self {
        Self {
            target_kind: RenderCameraTargetKind::Texture,
            status: ViewportTextureGraphImportStatus::BlockedPreparedFormatMismatch,
            #[cfg(test)]
            texture: Some(_texture),
            size: Some(size),
            #[cfg(test)]
            target_format: Some(_target_format.trim().to_string()),
            #[cfg(test)]
            expected_target_format: Some(_expected_target_format.to_string()),
            #[cfg(test)]
            source_format: Some(FRAMEWORK_OUTPUT_FORMAT_LABEL.to_string()),
        }
    }

    fn blocked_format(
        _texture: ResourceHandle<TextureMarker>,
        size: UVec2,
        _target_format: &str,
        _source_format: &str,
    ) -> Self {
        Self {
            target_kind: RenderCameraTargetKind::Texture,
            status: ViewportTextureGraphImportStatus::BlockedFormatMismatch,
            #[cfg(test)]
            texture: Some(_texture),
            size: Some(size),
            #[cfg(test)]
            target_format: Some(_target_format.to_string()),
            #[cfg(test)]
            expected_target_format: Some(_target_format.to_string()),
            #[cfg(test)]
            source_format: Some(_source_format.to_string()),
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
    pub(crate) fn expected_target_format(&self) -> Option<&str> {
        self.expected_target_format.as_deref()
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
    BlockedPreparedFormatMismatch,
}

fn format_label_matches(actual: &str, expected: &str) -> bool {
    actual.trim().eq_ignore_ascii_case(expected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::resource::ResourceId;

    #[test]
    fn output_target_plans_keep_diagnostic_payloads_out_of_production_builds() {
        let source = include_str!("viewport_render_output_target.rs");

        for field in [
            "texture: Option<ResourceHandle<TextureMarker>>",
            "target_format: Option<String>",
            "expected_target_format: Option<String>",
            "source_format: Option<String>",
        ] {
            assert_eq!(
                source
                    .matches(&format!("#[cfg(test)]\n    {field}"))
                    .count(),
                2,
                "both plan types must compile {field} only for tests",
            );
        }
    }

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
            format: FRAMEWORK_OUTPUT_FORMAT_LABEL,
        };

        let plan = target.writeback_plan(None);

        assert_eq!(
            plan.status(),
            ViewportTextureWritebackStatus::PendingTargetDescriptor
        );
        assert_eq!(plan.texture(), Some(texture));
        assert_eq!(plan.size(), Some(UVec2::new(128, 72)));
        assert_eq!(plan.source_format(), Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));
        assert_eq!(plan.target_format(), Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));
        assert_eq!(
            plan.expected_target_format(),
            Some(FRAMEWORK_OUTPUT_FORMAT_LABEL)
        );
    }

    #[test]
    fn output_target_writeback_plan_accepts_matching_srgb_format() {
        let texture = texture_handle("tests/writeback/ready");
        let target = ViewportRenderOutputTarget::Texture {
            handle: texture,
            size: UVec2::new(128, 72),
            format: FRAMEWORK_OUTPUT_FORMAT_LABEL,
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
        assert_eq!(
            plan.expected_target_format(),
            Some(FRAMEWORK_OUTPUT_FORMAT_LABEL)
        );
    }

    #[test]
    fn output_target_writeback_plan_accepts_linear_rgba_target_for_conversion() {
        let texture = texture_handle("tests/writeback/linear");
        let target = ViewportRenderOutputTarget::Texture {
            handle: texture,
            size: UVec2::new(128, 72),
            format: LINEAR_OUTPUT_FORMAT_LABEL,
        };

        let plan = target.writeback_plan(Some("rgba8unorm"));

        assert_eq!(
            plan.status(),
            ViewportTextureWritebackStatus::ReadyForConversion
        );
        assert_eq!(plan.texture(), Some(texture));
        assert_eq!(plan.source_format(), Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));
        assert_eq!(plan.target_format(), Some("rgba8unorm"));
        assert_eq!(
            plan.expected_target_format(),
            Some(LINEAR_OUTPUT_FORMAT_LABEL)
        );
    }

    #[test]
    fn output_target_writeback_plan_blocks_unsupported_target_format() {
        let texture = texture_handle("tests/writeback/unsupported");
        let target = ViewportRenderOutputTarget::Texture {
            handle: texture,
            size: UVec2::new(128, 72),
            format: "rgba16float",
        };

        let plan = target.writeback_plan(Some("rgba16float"));

        assert_eq!(
            plan.status(),
            ViewportTextureWritebackStatus::BlockedFormatMismatch
        );
        assert_eq!(plan.texture(), Some(texture));
        assert_eq!(plan.source_format(), Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));
        assert_eq!(plan.target_format(), Some("rgba16float"));
        assert_eq!(plan.expected_target_format(), Some("rgba16float"));
    }

    #[test]
    fn output_target_writeback_plan_blocks_prepared_format_drift() {
        let texture = texture_handle("tests/writeback/prepared-format-drift");
        let target = ViewportRenderOutputTarget::Texture {
            handle: texture,
            size: UVec2::new(128, 72),
            format: LINEAR_OUTPUT_FORMAT_LABEL,
        };

        let plan = target.writeback_plan(Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));

        assert_eq!(
            plan.status(),
            ViewportTextureWritebackStatus::BlockedPreparedFormatMismatch
        );
        assert_eq!(plan.texture(), Some(texture));
        assert_eq!(plan.source_format(), Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));
        assert_eq!(plan.target_format(), Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));
        assert_eq!(
            plan.expected_target_format(),
            Some(LINEAR_OUTPUT_FORMAT_LABEL)
        );
    }

    #[test]
    fn output_target_graph_import_plan_marks_srgb_texture_ready_for_direct_import() {
        let texture = texture_handle("tests/graph-import/srgb");
        let target = ViewportRenderOutputTarget::Texture {
            handle: texture,
            size: UVec2::new(128, 72),
            format: FRAMEWORK_OUTPUT_FORMAT_LABEL,
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
        assert_eq!(
            plan.expected_target_format(),
            Some(FRAMEWORK_OUTPUT_FORMAT_LABEL)
        );
    }

    #[test]
    fn output_target_graph_import_plan_keeps_linear_texture_on_conversion_writeback_path() {
        let texture = texture_handle("tests/graph-import/linear");
        let target = ViewportRenderOutputTarget::Texture {
            handle: texture,
            size: UVec2::new(128, 72),
            format: LINEAR_OUTPUT_FORMAT_LABEL,
        };

        let plan = target.graph_import_plan(Some("rgba8unorm"));

        assert_eq!(
            plan.status(),
            ViewportTextureGraphImportStatus::RequiresConversionWriteback
        );
        assert_eq!(plan.texture(), Some(texture));
        assert_eq!(plan.source_format(), Some(FRAMEWORK_OUTPUT_FORMAT_LABEL));
        assert_eq!(plan.target_format(), Some("rgba8unorm"));
        assert_eq!(
            plan.expected_target_format(),
            Some(LINEAR_OUTPUT_FORMAT_LABEL)
        );
    }

    #[test]
    fn output_target_graph_import_plan_blocks_unsupported_target_format() {
        let texture = texture_handle("tests/graph-import/unsupported");
        let target = ViewportRenderOutputTarget::Texture {
            handle: texture,
            size: UVec2::new(128, 72),
            format: "rgba16float",
        };

        let plan = target.graph_import_plan(Some("rgba16float"));

        assert_eq!(
            plan.status(),
            ViewportTextureGraphImportStatus::BlockedFormatMismatch
        );
        assert_eq!(plan.texture(), Some(texture));
        assert_eq!(plan.target_format(), Some("rgba16float"));
        assert_eq!(plan.expected_target_format(), Some("rgba16float"));
    }

    #[test]
    fn output_target_graph_import_plan_blocks_prepared_format_drift() {
        let texture = texture_handle("tests/graph-import/prepared-format-drift");
        let target = ViewportRenderOutputTarget::Texture {
            handle: texture,
            size: UVec2::new(128, 72),
            format: FRAMEWORK_OUTPUT_FORMAT_LABEL,
        };

        let plan = target.graph_import_plan(Some(LINEAR_OUTPUT_FORMAT_LABEL));

        assert_eq!(
            plan.status(),
            ViewportTextureGraphImportStatus::BlockedPreparedFormatMismatch
        );
        assert_eq!(plan.texture(), Some(texture));
        assert_eq!(plan.target_format(), Some(LINEAR_OUTPUT_FORMAT_LABEL));
        assert_eq!(
            plan.expected_target_format(),
            Some(FRAMEWORK_OUTPUT_FORMAT_LABEL)
        );
    }

    #[test]
    fn output_target_from_camera_target_retains_resolved_texture_format() {
        let texture = texture_handle("tests/output-target/from-camera-target");
        let target = ViewportRenderOutputTarget::from_camera_target(
            &RenderCameraTarget::Texture(texture),
            UVec2::new(96, 54),
            Some(LINEAR_OUTPUT_FORMAT_LABEL),
        );

        assert_eq!(target.texture_handle(), Some(texture));
        assert_eq!(target.size(), Some(UVec2::new(96, 54)));
        assert_eq!(target.texture_format(), Some(LINEAR_OUTPUT_FORMAT_LABEL));
    }

    fn texture_handle(label: &str) -> ResourceHandle<TextureMarker> {
        ResourceHandle::new(ResourceId::from_stable_label(label))
    }
}
