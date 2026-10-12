#[path = "../../../overlay/overlay/commit_pending_icon_uploads.rs"]
mod commit_pending_icon_uploads;
#[path = "construct/mod.rs"]
mod construct;
#[path = "../../../overlay/overlay/depth_reconstruction.rs"]
mod depth_reconstruction;
#[path = "pass_order/mod.rs"]
mod pass_order;
#[path = "../../../overlay/overlay/prepare_buffers/mod.rs"]
mod prepare_buffers;
#[path = "record/mod.rs"]
mod record;
#[path = "overlay.rs"]
mod viewport_overlay_renderer;

pub(crate) use viewport_overlay_renderer::ViewportOverlayRenderer;
