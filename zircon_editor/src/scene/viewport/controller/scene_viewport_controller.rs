use crate::core::settings::{SettingsRegistry, SettingsStore};
use crate::scene::viewport::ViewportInteractionExtractCache;
use crate::scene::viewport::handles::HandleToolRegistry;
use crate::scene::viewport::pointer::ViewportOverlayPointerRouter;

use super::scene_viewport_controller_overlay_providers::ViewportOverlayProviderRegistry;
use super::scene_viewport_state::SceneViewportState;

pub(crate) struct SceneViewportController {
    pub(in crate::scene::viewport::controller) state: SceneViewportState,
    pub(in crate::scene::viewport::controller) handles: HandleToolRegistry,
    pub(in crate::scene::viewport::controller) interaction_extract: ViewportInteractionExtractCache,
    pub(in crate::scene::viewport::controller) pointer_bridge: ViewportOverlayPointerRouter,
    pub(in crate::scene::viewport::controller) settings_registry: SettingsRegistry,
    pub(in crate::scene::viewport::controller) settings_store: Option<SettingsStore>,
    pub(in crate::scene::viewport::controller) overlay_providers: ViewportOverlayProviderRegistry,
}
