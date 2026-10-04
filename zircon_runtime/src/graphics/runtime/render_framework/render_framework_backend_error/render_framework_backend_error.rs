//! 后端错误在框架边界映射为产品级提交错误，保留设备丢失与普通图失败的区别。
use crate::core::framework::render::RenderFrameworkError;

use crate::graphics::GraphicsError;

pub(in crate::graphics::runtime::render_framework) fn render_framework_backend_error(
    error: GraphicsError,
) -> RenderFrameworkError {
    match error {
        GraphicsError::FrameProductPublicationFailed {
            receipt,
            product_submission,
            source,
        } => RenderFrameworkError::FrameProductPublicationFailed {
            receipt,
            product_submission,
            reason: source.to_string(),
        },
        GraphicsError::MissingViewFamilyPhase { phase } => {
            RenderFrameworkError::MissingViewFamilyPhase { phase }
        }
        GraphicsError::MissingFrameGraphResourceBacking { resource } => {
            RenderFrameworkError::MissingFrameGraphResourceBacking { resource }
        }
        GraphicsError::MissingPreparedGpuSceneUpload => {
            RenderFrameworkError::MissingPreparedGpuSceneUpload
        }
        GraphicsError::InvalidBufferUploadRange { label } => {
            RenderFrameworkError::InvalidBufferUploadRange { label }
        }
        GraphicsError::SceneSubmissionCompletion(error) => {
            RenderFrameworkError::SceneSubmissionCompletion(error)
        }
        GraphicsError::FrameProducerRegistrationFailed {
            ticket,
            status,
            source,
        } => RenderFrameworkError::FrameProducerRegistrationFailed {
            ticket,
            status,
            reason: source.to_string(),
        },
        error => RenderFrameworkError::Backend(error.to_string()),
    }
}

#[cfg(test)]
#[path = "tests/render_framework_backend_error.rs"]
mod tests;
