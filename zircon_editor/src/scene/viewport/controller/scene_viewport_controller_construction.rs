use std::sync::Arc;

use zircon_runtime_interface::math::UVec2;

use crate::core::settings::{SettingsAuthority, SettingsPersistenceService};
use crate::scene::viewport::handles::HandleToolRegistry;
use crate::scene::viewport::pointer::ViewportOverlayPointerRouter;
use crate::scene::viewport::ViewportInteractionExtractCache;

use super::{scene_viewport_state::SceneViewportState, SceneViewportController};

impl SceneViewportController {
    #[cfg(test)]
    pub(crate) fn new(viewport_size: UVec2) -> Self {
        let settings_authority = Arc::new(SettingsAuthority::with_defaults());
        let settings_persistence = SettingsPersistenceService::new(Arc::clone(&settings_authority));
        Self::with_settings(viewport_size, settings_authority, settings_persistence)
    }

    pub(crate) fn with_settings(
        viewport_size: UVec2,
        settings_authority: Arc<SettingsAuthority>,
        settings_persistence: SettingsPersistenceService,
    ) -> Self {
        Self {
            state: SceneViewportState::new(viewport_size),
            handles: HandleToolRegistry::default(),
            interaction_extract: ViewportInteractionExtractCache::default(),
            pointer_bridge: ViewportOverlayPointerRouter::new(),
            settings_authority,
            settings_persistence,
            settings_store: None,
            settings_persistence_tickets: Default::default(),
            overlay_providers: Default::default(),
        }
    }
}
