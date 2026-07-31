use zircon_runtime_interface::math::UVec2;

use crate::core::settings::settings_registry_with_defaults;
use crate::scene::viewport::ViewportInteractionExtractCache;
use crate::scene::viewport::handles::HandleToolRegistry;
use crate::scene::viewport::pointer::ViewportOverlayPointerRouter;

use super::{SceneViewportController, scene_viewport_state::SceneViewportState};

impl SceneViewportController {
    pub(crate) fn new(viewport_size: UVec2) -> Self {
        Self {
            state: SceneViewportState::new(viewport_size),
            handles: HandleToolRegistry::default(),
            interaction_extract: ViewportInteractionExtractCache::default(),
            pointer_bridge: ViewportOverlayPointerRouter::new(),
            settings_registry: settings_registry_with_defaults(),
            settings_store: None,
            overlay_providers: Default::default(),
        }
    }
}
