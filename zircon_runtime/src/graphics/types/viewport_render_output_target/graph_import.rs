use crate::core::framework::render::RenderCameraTargetKind;
use crate::core::math::UVec2;
use crate::core::resource::{ResourceHandle, TextureMarker};

use super::{
    FRAMEWORK_OUTPUT_FORMAT_LABEL, LINEAR_OUTPUT_FORMAT_LABEL, ViewportRenderOutputTarget,
    format_label_matches,
};

impl ViewportRenderOutputTarget {
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
