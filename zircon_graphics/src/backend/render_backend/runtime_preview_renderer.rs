use crate::scene::{ResourceStreamer, SceneRendererCore};

use super::{render_backend::RenderBackend, surface_state::SurfaceState};

pub struct RuntimePreviewRenderer {
    pub(crate) backend: RenderBackend,
    pub(crate) surface_state: SurfaceState,
    pub(crate) scene_renderer: SceneRendererCore,
    pub(crate) streamer: ResourceStreamer,
}
