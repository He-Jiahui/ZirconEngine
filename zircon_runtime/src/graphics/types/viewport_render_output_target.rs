use crate::core::framework::render::{RenderCameraTarget, RenderCameraTargetKind};
use crate::core::math::UVec2;
use crate::core::resource::{ResourceHandle, TextureMarker};

mod graph_import;
mod writeback;

pub(crate) use graph_import::{ViewportTextureGraphImportPlan, ViewportTextureGraphImportStatus};
pub(crate) use writeback::{ViewportTextureWritebackPlan, ViewportTextureWritebackStatus};

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
}

fn format_label_matches(actual: &str, expected: &str) -> bool {
    actual.trim().eq_ignore_ascii_case(expected)
}

#[cfg(test)]
mod tests;
