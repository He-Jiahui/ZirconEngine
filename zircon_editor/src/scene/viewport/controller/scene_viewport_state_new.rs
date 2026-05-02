use crate::scene::viewport::SceneViewportSettings;
use crate::scene::viewport::ViewportState;
use zircon_runtime_interface::math::{UVec2, Vec3};

use super::{scene_viewport_state::SceneViewportState, viewport_hover_state::ViewportHoverState};

impl SceneViewportState {
    pub(in crate::scene::viewport::controller) fn new(viewport_size: UVec2) -> Self {
        Self {
            settings: SceneViewportSettings::default(),
            selected: None,
            viewport: ViewportState::new(viewport_size),
            camera: None,
            orbit_target: Vec3::ZERO,
            hover: ViewportHoverState::default(),
            drag: None,
        }
    }
}
