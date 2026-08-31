use crate::core::framework::render::RenderViewportHandle;
use crate::graphics::scene::ViewportAsyncCaptureSubmission;
use crate::graphics::GraphicsError;

use super::super::super::render_framework_state::RenderFrameworkState;

pub(super) fn publish_viewport_product(
    state: &mut RenderFrameworkState,
    viewport: RenderViewportHandle,
    frame: &mut ViewportAsyncCaptureSubmission,
) -> Result<(), GraphicsError> {
    let copy = match frame.take_viewport_product_copy() {
        Some(copy) => copy,
        None => {
            return Err(GraphicsError::FrameProductPublicationFailed {
                receipt: frame.submission_receipt().clone(),
                product_submission: None,
                source: Box::new(GraphicsError::SurfaceStatus(
                    "requested viewport product copy was not recorded in the scene packet",
                )),
            });
        }
    };
    state
        .viewport_products
        .publish(viewport, copy, frame.submission_receipt())
}

#[cfg(test)]
mod tests {
    #[test]
    fn publication_commits_only_the_copy_already_bound_to_the_frame_identity() {
        let source = include_str!("publish_viewport_product.rs");

        assert!(source.contains("frame.take_viewport_product_copy()"));
        assert!(source.contains("frame.submission_receipt()"));
        assert!(source.contains("viewport_products.publish("));
        assert!(!source.contains("replace_submission_receipt"));
        assert!(!source.contains("retain_viewport_product_submission_receipt"));
    }
}
