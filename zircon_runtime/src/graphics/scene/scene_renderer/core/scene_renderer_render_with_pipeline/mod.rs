mod async_viewport_capture;
mod render_frame_with_pipeline;

pub(in crate::graphics::scene::scene_renderer::core) use render_frame_with_pipeline::render_gpu_timing_status;

pub(crate) use async_viewport_capture::{
    AsyncViewportCaptureRequest, ViewportAsyncCaptureSubmission, capture_request_was_admitted,
};
